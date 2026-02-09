output "s3_website_url" {
  value = "http://${aws_s3_bucket.website_bucket.bucket}.s3-website-${var.region}.amazonaws.com"
}

output "pi_policy" {
  value = aws_iot_policy.pi_policy.arn
}

output "api_gatway_routes" {
  description = "The URLs for the Lambda invocation routes"
  value = {
    for key in var.lambda_handlers :
    key => "${aws_apigatewayv2_api.http_api.api_endpoint}/${key}"
  }
}
