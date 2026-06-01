# -------------------- Provision downstream k8s cluster -------------------- #

# Create AmazonEC2 cloud credential
resource "rancher2_cloud_credential" "aws_cloud_credential" {
  name = var.aws_creds_name
  amazonec2_credential_config {
    access_key     = var.aws_access_key
    secret_key     = var.aws_secret_key
    default_region = "us-east-1"
  }
}

# Create AmazonEC2 machine config v2
resource "rancher2_machine_config_v2" "zone_a" {
  generate_name = "zone-a-pool"
  amazonec2_config {
    ami           = "ami-0e2c8caa4b6378d8c"
    region        = "us-east-1"
    security_group = [aws_security_group.downstream_kubernetes_security_group.name]
    subnet_id     = aws_subnet.downstream_kubernetes_subnet_zone_a.id
    vpc_id        = aws_vpc.downstream_kubernetes_vpc.id
    zone          = "a"
    root_size     = "20"
    instance_type = "t3a.small"
    volume_type = "gp3"
    iam_instance_profile = "k8s-ebs-csi"
  }
}

resource "rancher2_machine_config_v2" "zone_b" {
  generate_name = "zone-b-pool"
  amazonec2_config {
    ami           = "ami-0e2c8caa4b6378d8c"
    region        = "us-east-1"
    security_group = [aws_security_group.downstream_kubernetes_security_group.name]
    subnet_id     = aws_subnet.downstream_kubernetes_subnet_zone_b.id
    vpc_id        = aws_vpc.downstream_kubernetes_vpc.id
    zone          = "b"
    root_size     = "20"
    instance_type = "t3a.small"
    volume_type = "gp3"
    iam_instance_profile = "k8s-ebs-csi"

  }
}

resource "rancher2_machine_config_v2" "zone_c" {
  generate_name = "zone-c-pool"
  amazonec2_config {
    ami           = "ami-0e2c8caa4b6378d8c"
    region        = "us-east-1"
    security_group = [aws_security_group.downstream_kubernetes_security_group.name]
    subnet_id     = aws_subnet.downstream_kubernetes_subnet_zone_c.id
    vpc_id        = aws_vpc.downstream_kubernetes_vpc.id
    zone          = "c"
    root_size     = "20"
    instance_type = "t3a.small"
    volume_type = "gp3"
    iam_instance_profile = "k8s-ebs-csi"
  }
}

resource "rancher2_machine_config_v2" "zone_d" {
  generate_name = "zone-d-pool"
  amazonec2_config {
    ami           = "ami-0e2c8caa4b6378d8c"
    region        = "us-east-1"
    security_group = [aws_security_group.downstream_kubernetes_security_group.name]
    subnet_id     = aws_subnet.downstream_kubernetes_subnet_zone_d.id
    vpc_id        = aws_vpc.downstream_kubernetes_vpc.id
    zone          = "d"
    root_size     = "20"
    instance_type = "t3a.small"
    volume_type = "gp3"
    iam_instance_profile = "k8s-ebs-csi"
  }
}

resource "rancher2_cluster_v2" "downstream_kubernetes_cluster" {
  name                  = "downstream-k8s-cluster"
  kubernetes_version    = "v1.31.2+k3s1"
  enable_network_policy = false
  rke_config {
    machine_global_config = <<EOF
kube-apiserver-arg:
  - allow-privileged=true
EOF
    machine_pools {
      name                         = "zone-a-pool"
      cloud_credential_secret_name = rancher2_cloud_credential.aws_cloud_credential.id
      control_plane_role           = true
      etcd_role                    = true
      worker_role                  = true
      quantity                     = 1
      machine_config {
        kind = rancher2_machine_config_v2.zone_a.kind
        name = rancher2_machine_config_v2.zone_a.name
      }
    }

    machine_pools {
      name                         = "zone-b-pool"
      cloud_credential_secret_name = rancher2_cloud_credential.aws_cloud_credential.id
      control_plane_role           = true
      etcd_role                    = true
      worker_role                  = true
      quantity                     = 1
      machine_config {
        kind = rancher2_machine_config_v2.zone_b.kind
        name = rancher2_machine_config_v2.zone_b.name
      }
    }

    machine_pools {
      name                         = "zone-c-pool"
      cloud_credential_secret_name = rancher2_cloud_credential.aws_cloud_credential.id
      control_plane_role           = true
      etcd_role                    = true
      worker_role                  = true
      quantity                     = 1
      machine_config {
        kind = rancher2_machine_config_v2.zone_c.kind
        name = rancher2_machine_config_v2.zone_c.name
      }
    }

    machine_pools {
      name                         = "zone-d-pool"
      cloud_credential_secret_name = rancher2_cloud_credential.aws_cloud_credential.id
      control_plane_role           = true
      etcd_role                    = true
      worker_role                  = true
      quantity                     = 1
      machine_config {
        kind = rancher2_machine_config_v2.zone_d.kind
        name = rancher2_machine_config_v2.zone_d.name
      }
    }

  }
}