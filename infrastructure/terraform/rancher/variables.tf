# rancher bootstrap
# variable "rancher2_access_key" {}
# variable "rancher2_secret_key" {}
# variable "rancher2_api_url" {}
variable "rancher2_admin_password" {}
variable "rancher2_enable_telemetry" { default = false }
variable "rancher2_insecure" { default = false }

# aws cloud credentials
variable "aws_creds_name" { default = "aws-creds" }
variable "aws_access_key" {}
variable "aws_secret_key" {}

