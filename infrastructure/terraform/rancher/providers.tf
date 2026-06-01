terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.75"
    }

    rancher2 = {
      source = "rancher/rancher2"
      version = "6.0.0"
    }
  }

  required_version = ">= 1.2.0"
}


# Configure the Rancher2 provider to bootstrap and admin
# Provider config for bootstrap
provider "rancher2" {
  alias = "bootstrap"

  api_url   = module.rancher_host_server.rancher_api_url
  insecure = true
  bootstrap = true
}

# Create a new rancher2_bootstrap using bootstrap provider config
resource "rancher2_bootstrap" "admin" {
  provider = rancher2.bootstrap

  password = var.rancher2_admin_password
  telemetry = var.rancher2_enable_telemetry

  depends_on = [module.rancher_host_server.rancher_ec2_server_ip]


}

# Provider config for admin
provider "rancher2" {
  api_url = rancher2_bootstrap.admin.url
  token_key = rancher2_bootstrap.admin.token
  insecure = var.rancher2_insecure
}

module "rancher_host_server" {
  source = "./.."

  # aws_access_key = var.aws_access_key
  # aws_secret_key = var.aws_secret_key
  #
  # rancher2_admin_password = var.rancher2_admin_password
  # rancher2_api_url        = module.rancher_host_server.rancher_api_url
}

