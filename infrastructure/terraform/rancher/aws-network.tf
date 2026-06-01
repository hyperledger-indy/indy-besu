# -------------------- Network -------------------- #

resource "aws_vpc" "downstream_kubernetes_vpc" {
  cidr_block           = "10.0.0.0/16"
  enable_dns_hostnames = true
  tags = {
    Name = "k8s-vpc"
  }
}

resource "aws_internet_gateway" "downstream_kubernetes_gateway" {
  vpc_id = aws_vpc.downstream_kubernetes_vpc.id

  tags = {
    Name = "k8s-gateway"
  }
}

resource "aws_subnet" "downstream_kubernetes_subnet_zone_a" {
  vpc_id = aws_vpc.downstream_kubernetes_vpc.id

  availability_zone = "us-east-1a"
  cidr_block        = "10.0.0.0/24"

  tags = {
    Name = "k8s-subnet-zone-a"
  }
}

resource "aws_subnet" "downstream_kubernetes_subnet_zone_b" {
  vpc_id = aws_vpc.downstream_kubernetes_vpc.id

  availability_zone = "us-east-1b"
  cidr_block        = "10.0.1.0/24"

  tags = {
    Name = "k8s-subnet-zone-b"
  }
}

resource "aws_subnet" "downstream_kubernetes_subnet_zone_c" {
  vpc_id = aws_vpc.downstream_kubernetes_vpc.id

  availability_zone = "us-east-1c"
  cidr_block        = "10.0.2.0/24"

  tags = {
    Name = "k8s-subnet-zone-c"
  }
}

resource "aws_subnet" "downstream_kubernetes_subnet_zone_d" {
  vpc_id = aws_vpc.downstream_kubernetes_vpc.id

  availability_zone = "us-east-1d"
  cidr_block        = "10.0.3.0/24"

  tags = {
    Name = "k8s-subnet-zone-d"
  }
}

resource "aws_route_table" "downstream_kubernetes_route_table" {
  vpc_id = aws_vpc.downstream_kubernetes_vpc.id

  route {
    cidr_block = "0.0.0.0/0"
    gateway_id = aws_internet_gateway.downstream_kubernetes_gateway.id
  }

  tags = {
    Name = "k8s-route-table"
  }
}

resource "aws_route_table_association" "downstream_kubernetes_route_table_association_zone_a" {
  subnet_id      = aws_subnet.downstream_kubernetes_subnet_zone_a.id
  route_table_id = aws_route_table.downstream_kubernetes_route_table.id
}

resource "aws_route_table_association" "downstream_kubernetes_route_table_association_zone_b" {
  subnet_id      = aws_subnet.downstream_kubernetes_subnet_zone_b.id
  route_table_id = aws_route_table.downstream_kubernetes_route_table.id
}

resource "aws_route_table_association" "downstream_kubernetes_route_table_association_zone_c" {
  subnet_id      = aws_subnet.downstream_kubernetes_subnet_zone_c.id
  route_table_id = aws_route_table.downstream_kubernetes_route_table.id
}

resource "aws_route_table_association" "downstream_kubernetes_route_table_association_zone_d" {
  subnet_id      = aws_subnet.downstream_kubernetes_subnet_zone_d.id
  route_table_id = aws_route_table.downstream_kubernetes_route_table.id
}

resource "aws_security_group" "downstream_kubernetes_security_group" {
  name        = "k8s-security-group"
  description = "Downstream k8s security group"
  vpc_id      = aws_vpc.downstream_kubernetes_vpc.id

  ingress {
    from_port   = "0"
    to_port     = "0"
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  egress {
    from_port   = "0"
    to_port     = "0"
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

}