output "rancher_admin_password" {
  value = rancher2_bootstrap.admin.current_password
  sensitive = true
}