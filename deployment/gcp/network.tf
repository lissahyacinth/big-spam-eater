module "vpc" {
  source  = "terraform-google-modules/network/google"
  version = "~> 9.1"

  project_id   = var.project_id
  network_name = local.network_name
  mtu          = "1460"
  routing_mode = "REGIONAL"
  subnets = [{
    subnet_name   = local.subnet_name
    subnet_ip     = "10.0.1.0/24"
    subnet_region = "europe-west2"
  }]
}

resource "google_compute_firewall" "firewall-ssh" {
  name    = "${var.env}-firewall-ssh"
  network = module.vpc.network_name

  allow {
    protocol = "icmp"
  }

  allow {
    protocol = "tcp"
    ports    = ["22"]
  }

  source_ranges = var.local_ips
}