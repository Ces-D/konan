output "s3_website_url" {
  value = "http://${aws_s3_bucket.website_bucket.bucket}.s3-website-${var.region}.amazonaws.com"
}

output "pi_policy" {
  value = aws_iot_policy.pi_policy.arn
}
