output "s3_website_url" {
  value = "https://${aws_cloudfront_distribution.s3_distribution.domain_name}"
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

output "iot_endpoint_url" {
  description = "The AWS IoT Core data endpoint URL"
  value       = data.aws_iot_endpoint.iot_endpoint.endpoint_address
}

output "iot_client_id" {
  value = aws_iot_thing.raspberry_pi.name
}
