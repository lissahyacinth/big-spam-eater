resource "google_service_account" "compute_account" {
  account_id   = "ds-compute-sa"
  display_name = "Data Science Compute SA"
  description  = "Service Account for DS Compute"
}

