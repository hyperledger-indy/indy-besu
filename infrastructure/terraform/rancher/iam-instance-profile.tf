# # resource "aws_iam_instance_profile" "test_profile" {
# #   name = "ebs-csi-instance-profile"
# #   role = aws_iam_role.role.name
# # }
#
# resource "aws_iam_policy" "ebs_csi_driver_policy" {
#   name        = "aws-ebs-csi-driver-policy"
#   path        = "/"
#   description = "aws EBS CSI Driver Policy"
#
#   policy = file("${path.module}/files/ebs-csi-driver-policy.json")
# }
#
# resource "aws_iam_policy_attachment" "ebs_csi_driver_role_policy_attachment" {
#   name       = "aws-ebs-csi-driver-role-policy-attachment"
#   roles      = [aws_iam_role.ebs_csi_driver_role.name]
#   policy_arn = aws_iam_policy.ebs_csi_driver_policy.arn
# }
#
# resource "aws_iam_role" "ebs_csi_driver_role" {
#   name               = "k8s-ebs-csi"
#   path               = "/"
#   assume_role_policy = data.aws_iam_policy_document.ebs_csi.json
# }
#
# # data "aws_iam_policy_document" "ebs_csi" {
# #   statement {
# #     effect = "Allow"
# #
# #     actions = [
# #       "ec2:CreateSnapshot",
# #       "ec2:AttachVolume",
# #       "ec2:DetachVolume",
# #       "ec2:ModifyVolume",
# #       "ec2:DescribeAvailabilityZones",
# #       "ec2:DescribeInstances",
# #       "ec2:DescribeSnapshots",
# #       "ec2:DescribeTags",
# #       "ec2:DescribeVolumes",
# #       "ec2:DescribeVolumesModifications",
# #       "ec2:EnableFastSnapshotRestores"
# #     ]
# #
# #     resources = ["*"]
# #   }
# #   statement {
# #     effect = "Allow"
# #
# #     actions = [
# #       "ec2:CreateTags"
# #     ]
# #
# #     resources = [
# #       "arn:*:ec2:*:*:volume/*",
# #       "arn:*:ec2:*:*:snapshot/*"
# #     ]
# #   }
# #   statement {
# #     effect = "Allow"
# #
# #     actions = [
# #       "ec2:CreateTags"
# #     ]
# #
# #     resources = [
# #       "arn:*:ec2:*:*:volume/*",
# #       "arn:*:ec2:*:*:snapshot/*"
# #     ]
# #   }
# # }
#
