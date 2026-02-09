terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "6.31.0"
    }
  }
}

provider "aws" {
  region = var.region
  default_tags {
    tags = {
      Project   = var.project_name
      ManagedBy = "Terraform"
    }
  }
}

locals {
  pi_topics = [
    for handler in var.lambda_handlers :
    "command/${aws_iot_thing.raspberry_pi.name}/${handler}"
  ]
  mime_types = {
    ".html"  = "text/html"
    ".css"   = "text/css"
    ".js"    = "application/javascript"
    ".json"  = "application/json"
    ".png"   = "image/png"
    ".jpg"   = "image/jpeg"
    ".jpeg"  = "image/jpeg"
    ".gif"   = "image/gif"
    ".svg"   = "image/svg+xml"
    ".ico"   = "image/x-icon"
    ".woff"  = "font/woff"
    ".woff2" = "font/woff2"
  }
  # Resolve the site directory robustly whether var.website_location is
  # relative to the module or an absolute path.
  site_dir     = join("/", [path.module, var.website_location])
  s3_origin_id = "konanS3Origin"
}

data "aws_caller_identity" "current" {}

# --- 1. IoT Core (Raspberry Pi) ---
# Note: generate certs locally on the Pi

data "aws_iot_endpoint" "iot_endpoint" {
  endpoint_type = "iot:Data-ATS"
}

resource "aws_iot_thing" "raspberry_pi" {
  name = "${var.project_name}_pi"
}

resource "aws_iot_thing_principal_attachment" "pi_cert_attachment" {
  thing     = aws_iot_thing.raspberry_pi.name
  principal = var.pi_certificate_arn
}

resource "aws_iot_policy_attachment" "pi_policy_attachment" {
  policy = aws_iot_policy.pi_policy.name
  target = var.pi_certificate_arn
}

resource "aws_iot_policy" "pi_policy" {
  name = "${var.project_name}_pi_policy"
  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action   = ["iot:Connect"]
        Effect   = "Allow"
        Resource = "arn:aws:iot:${var.region}:${data.aws_caller_identity.current.account_id}:client/${aws_iot_thing.raspberry_pi.name}"
      },
      { Action = ["iot:Subscribe"]
        Effect = "Allow",
        Resource = [
          for topic in local.pi_topics :
          "arn:aws:iot:${var.region}:${data.aws_caller_identity.current.account_id}:topicfilter/${topic}"
        ]
      },
      {
        Action = ["iot:Receive"]
        Effect = "Allow"
        Resource = [
          for topic in local.pi_topics :
          "arn:aws:iot:${var.region}:${data.aws_caller_identity.current.account_id}:topic/${topic}"
        ]
      }
    ]
  })
}

# --- 2. Lambda Functions (Business Logic) ---
# IAM Role for Lambdas
resource "aws_iam_role" "lambda_exec" {
  name        = "${var.project_name}_role"
  description = "Execution role for ${var.project_name} Lambdas to publish to IoT and write logs"
  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Action    = "sts:AssumeRole"
      Effect    = "Allow"
      Principal = { Service = "lambda.amazonaws.com" }
    }]
  })
}

resource "aws_iam_role_policy" "lambda_iot" {
  role = aws_iam_role.lambda_exec.id
  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = "iot:Publish"
        Effect = "Allow"
        Resource = [
          for topic in local.pi_topics :
          "arn:aws:iot:${var.region}:${data.aws_caller_identity.current.account_id}:topic/${topic}"
        ]
      },
      {
        Action   = ["logs:CreateLogGroup", "logs:CreateLogStream", "logs:PutLogEvents"]
        Effect   = "Allow"
        Resource = "arn:aws:logs:*:*:*"
      }
    ]
  })
}

resource "aws_lambda_function" "pi_lambdas" {
  for_each      = toset(var.lambda_handlers)
  function_name = "${var.project_name}_${each.value}"
  role          = aws_iam_role.lambda_exec.arn
  handler       = "bootstrap"       # Rust handler
  runtime       = "provided.al2023" # Rust runtime
  description   = "${var.project_name} ${each.value} Lambda; publishes commands to IoT Thing"

  timeout       = 5          # Reduce from default 30s
  memory_size   = 128        # Start small, increase if needed
  architectures = ["x86_64"] # 20% cheaper than x86
  # Reduce cold start costs by disabling init billing for simple functions
  snap_start {
    apply_on = "None" # Only use for Java, not needed for Rust
  }

  filename         = "${path.module}/build/${each.value}.zip"
  source_code_hash = filebase64sha256("${path.module}/build/${each.value}.zip")

  environment {
    variables = {
      IOT_ENDPOINT = data.aws_iot_endpoint.iot_endpoint.endpoint_address
      IOT_TOPIC    = "command/${aws_iot_thing.raspberry_pi.name}/${each.value}"
      RUST_LOG     = "error"
    }
  }
}

# resource "aws_cloudwatch_log_group" "lambda_logs" {
#   for_each          = toset(var.lambda_handlers)
#   name              = "/aws/lambda/${aws_lambda_function.pi_lambdas[each.value].function_name}"
#   retention_in_days = 7 # Reduce from default 'never expire'
# }

# --- 3. HTTP API Gateway ---
resource "aws_apigatewayv2_api" "http_api" {
  name          = "${var.project_name}_api"
  protocol_type = "HTTP"
  description   = "HTTP API for ${var.project_name} to invoke Lambdas"

  cors_configuration {
    allow_origins = ["https://${aws_cloudfront_distribution.s3_distribution.domain_name}"]
    allow_methods = ["POST", "OPTIONS"]
    allow_headers = ["Content-Type"]
    max_age       = 300
  }
}

resource "aws_apigatewayv2_stage" "default" {
  api_id      = aws_apigatewayv2_api.http_api.id
  name        = "$default"
  auto_deploy = true
  description = "Default stage for ${var.project_name} API"
}

resource "aws_apigatewayv2_route" "routes" {
  for_each       = toset(var.lambda_handlers)
  api_id         = aws_apigatewayv2_api.http_api.id
  route_key      = "POST /${each.value}"
  target         = "integrations/${aws_apigatewayv2_integration.lambda_int[each.value].id}"
  operation_name = "${var.project_name}_${each.value}_route"
}

resource "aws_apigatewayv2_integration" "lambda_int" {
  for_each               = toset(var.lambda_handlers)
  api_id                 = aws_apigatewayv2_api.http_api.id
  integration_type       = "AWS_PROXY"
  integration_uri        = aws_lambda_function.pi_lambdas[each.value].invoke_arn
  payload_format_version = "2.0"
  description            = "Proxy integration for ${each.value} Lambda"
}

# Permissions for API Gateway to invoke Lambdas
resource "aws_lambda_permission" "api_gw_invoke_funcs" {
  for_each      = toset(var.lambda_handlers)
  statement_id  = "AllowExecutionFromAPIGateway"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.pi_lambdas[each.value].function_name
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_apigatewayv2_api.http_api.execution_arn}/*/*"
}

# --- 4. Static Site Hosting (S3) ---
resource "aws_s3_bucket" "website_bucket" {
  bucket = "${var.project_name}-svelte-site"
}

resource "aws_s3_bucket_public_access_block" "website_policy" {
  bucket                  = aws_s3_bucket.website_bucket.id
  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

resource "aws_cloudfront_origin_access_control" "default" {
  name                              = "${var.project_name}_s3_oac"
  origin_access_control_origin_type = "s3"
  signing_behavior                  = "always"
  signing_protocol                  = "sigv4"
}

resource "aws_cloudfront_distribution" "s3_distribution" {
  enabled             = true
  is_ipv6_enabled     = true
  default_root_object = "index.html"
  origin {
    domain_name              = aws_s3_bucket.website_bucket.bucket_regional_domain_name
    origin_access_control_id = aws_cloudfront_origin_access_control.default.id
    origin_id                = local.s3_origin_id
  }
  default_cache_behavior {
    viewer_protocol_policy = "redirect-to-https"
    allowed_methods        = ["GET", "HEAD"]
    cached_methods         = ["GET", "HEAD"]
    target_origin_id       = local.s3_origin_id
    min_ttl                = 0
    default_ttl            = 86400  # 24 hours
    max_ttl                = 604800 # 7 days 
    compress               = true
    forwarded_values {
      query_string = false
      cookies {
        forward = "none"
      }
    }
  }
  price_class = "PriceClass_100"
  restrictions {
    geo_restriction {
      restriction_type = "whitelist"
      locations        = ["US", "MX"]
    }
  }
  viewer_certificate {
    cloudfront_default_certificate = true
  }
}

resource "aws_s3_bucket_policy" "svelte_site_policy" {
  bucket     = aws_s3_bucket.website_bucket.id
  depends_on = [aws_s3_bucket_public_access_block.website_policy]
  policy = jsonencode({
    Version = "2012-10-17",
    Statement = [
      {
        Effect    = "Allow",
        Principal = { Service = "cloudfront.amazonaws.com" },
        Action    = "s3:GetObject",
        Resource  = "${aws_s3_bucket.website_bucket.arn}/*"
        Condition = {
          StringEquals = {
            "AWS:SourceARN" = aws_cloudfront_distribution.s3_distribution.arn
          }
        }
      }
    ]
  })
}

resource "aws_s3_object" "svelte_files" {
  for_each     = fileset(local.site_dir, "**")
  bucket       = aws_s3_bucket.website_bucket.id
  key          = each.value
  source       = "${local.site_dir}/${each.value}"
  etag         = filemd5("${local.site_dir}/${each.value}") # triggers update when contents change
  content_type = lookup(local.mime_types, regex("\\.[^.]+$", each.value), "application/octet-stream")
}
