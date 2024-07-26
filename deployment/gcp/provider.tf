terraform {
  required_version = "~> 1.3.0"

  required_providers {
    google = {
      source  = "hashicorp/google"
      version = "~> 5.21.0"
    }
  }
}

provider "google" {
  project = var.project_id
  region  = var.region
  zone    = var.zone
  default_labels = {
    "managed_by" : "terraform"
  }
}
