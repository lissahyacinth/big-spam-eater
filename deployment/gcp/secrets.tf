resource "google_secret_manager_secret" "discord_token_basic" {
  secret_id = "discord_token"

  replication {
    user_managed {
      replicas {
        location = var.region
      }
    }
  }
}

resource "google_secret_manager_secret_version" "token_version" {
  secret      = google_secret_manager_secret.discord_token_basic.id
  secret_data = var.token
}

resource "google_secret_manager_secret" "openai_token_basic" {
  secret_id = "openai_token"

  replication {
    user_managed {
      replicas {
        location = var.region
      }
    }
  }
}

resource "google_secret_manager_secret_version" "openai_token_version" {
  secret      = google_secret_manager_secret.openai_token_basic.id
  secret_data = var.token
}