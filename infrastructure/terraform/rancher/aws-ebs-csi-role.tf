# resource "aws_iam_role" "ebs_csi" {
#   name               = "ebs-csi"
#   assume_role_policy = data.aws_iam_policy_document.ebs_csi_irsa.json
# }
#
# resource "aws_iam_role_policy_attachment" "AmazonEBSCSIDriverPolicy" {
#   policy_arn = "arn:aws:iam::aws:policy/service-role/AmazonEBSCSIDriverPolicy"
#   role       = aws_iam_role.ebs_csi.name
# }