/**
 * Copyright 2018 Google LLC
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

locals {
  instance_name = format("%s-%s", var.instance_name, substr(md5(module.gce-container.container.image), 0, 8))
}

module "gce-container" {
  source  = "terraform-google-modules/container-vm/google"
  version = "~> 3.0"

  container = {
    image = var.image
    env = [
      {
        name  = "DISCORD_TOKEN"
        value = google_secret_manager_secret_version.token_version.secret_data
    }]
    volumeMounts = []
  }

  volumes = []

  restart_policy = "Always"
}

resource "google_compute_instance" "vm" {
  project      = var.project_id
  name         = local.instance_name
  machine_type = var.machine_type
  zone         = var.zone

  boot_disk {
    initialize_params {
      image = module.gce-container.source_image
    }
  }

  network_interface {
    network    = module.vpc.network_id
    subnetwork = local.subnet_name
    access_config {}
  }

  metadata = {
    gce-container-declaration = module.gce-container.metadata_value
    google-logging-enabled    = "true"
    google-monitoring-enabled = "true"
  }

  lifecycle {
    ignore_changes = [metadata]
  }

  labels = merge(
    {
      container-vm = module.gce-container.vm_container_label
    },
    var.labels
  )

  service_account {
    email = google_service_account.compute_account.email
    scopes = [
      "https://www.googleapis.com/auth/cloud-platform",
    ]
  }
}