terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.75"
    }
  }

  required_version = ">= 1.2.0"
}

provider "aws" {
  region  = "us-east-1"
}

resource "aws_key_pair" "rancher_ec2_server_key_pair" {
  key_name   = "mirafzal-pc-key-pair"
  public_key = "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAACAQDQK9fOCy2mVnk5v+tsqfUmr+moMwg/OrCrDti1oWZPmQQL3PfOCSDRA2WvScaKgqRqDh4yKhE3uWUS9ZvdT73f5nD5omHPLQbs2IWcMhUNyn4uC8w3nM2G969QSXsyUR6H+gzGkDjNpA+CWoEhNhyIOLZVbJsZbpyerK11xrkGrymCP0m28E69xm5hLoEathk/BYlakLNqqeoJBFafljvFjmRG9HkcADLQW5jmJc9xDHrzu+AXBNa0SPDiB5wB0be2f5Se6WTqW2kBDWcMHwxKW+sJHhe+tXsnlABPFhVJe9eRKuT31Vfu8QbUFpduYx7B+MqWw/i0zz1t1tpVakKNJtzPGfDaXLdIN7j9+v33sfNAsT9byU6k8+vv6ewK/OcmgPGOlgGQrVUlI9wIGQw0/yz6+hkIu14veL37kt2gR7GcIVOIFZ8/gS6jhtsFAw5AWrLMV1FIRB8VggzQ6nTcCwnjLVRr9N6vgWNZQcLaOW5Ovlh051dMMFlZtYdKUllhs9l2dVKW8vTEEipyEFSIpczvU2PXNi4Mxfr39THZfa+eZDsxUan1TCA/b3dvsPbdaRYYsql9XRCmAcuzIHcw4i1vkFXQoaQgz0MTcaGCv9bevNwImA/wgLHH+oQC4O1BTGz8q+kKyHRnZOAwP8TF8MtuPhUms03nNdpHXpHUQQ== sammy@DESKTOP-JJ791QV"
}

resource "aws_instance" "rancher_ec2_server" {
  ami           = "ami-0453ec754f44f9a4a"
  instance_type = "t3a.medium"

  root_block_device {
    volume_size = "20"
  }

  tags = {
    Name = "rancher-ec2-server"
  }

  key_name = aws_key_pair.rancher_ec2_server_key_pair.key_name

  user_data = templatefile(
    "${path.module}/files/userdata_rancher_node.template",
    {}
  )

}

output "rancher_ec2_server_ip" {
  value = aws_instance.rancher_ec2_server.public_ip
}

output "rancher_api_url" {
  value = "https://${aws_instance.rancher_ec2_server.public_ip}.sslip.io"
}

output "rancher_ec2_server_ssh_command" {
  value = "ssh ec2-user@${aws_instance.rancher_ec2_server.public_ip}"
}