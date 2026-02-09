variable "project_name" {
  description = "Project namespace"
  type        = string
  default     = "konan"
}

variable "pi_certificate_arn" {
  type = string
}

variable "region" {
  default = "us-east-1"
}

variable "lambda_handlers" {
  default = ["habits", "message", "outline"]
}

variable "website_location" {
  default = "konan_web/build"
}
