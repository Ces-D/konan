variable "project_name" {
  description = "Project namespace"
  type        = string
  default     = "konan"
}

variable "pi_certificate_arn" {
  type = string
}

variable "region" {
  type    = string
  default = "us-east-1"
}

variable "lambda_handlers" {
  type    = list(string)
  default = ["habits", "message", "outline"]
}

variable "api_gateway_allowed_origins" {
  description = "Allowed callers of api gateway"
  type        = list(string)
  default     = []
}
