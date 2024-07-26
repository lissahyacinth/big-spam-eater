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

variable "project_id" {
  description = "The project ID to deploy resources into"
  type        = string
}

variable "image" {
  default     = "ghcr.io/lissahyacinth/big-spam-eater:main"
  description = "Full Docker Image Name"
  type        = string
}

variable "machine_type" {
  default     = "e2-micro"
  description = "GCP Machine Type"
  type        = string
}

variable "instance_name" {
  default     = "spam-eater"
  description = "The desired name to assign to the deployed instance"
  type        = string
}

variable "region" {
  description = "The GCP region to deploy instances into"
  type        = string
}

variable "zone" {
  description = "The GCP zone to deploy instances into"
  type        = string
}

variable "labels" {
  description = "Labels for deployed instance"
  type        = map(any)
  default     = {}
}

variable "env" {
  default     = "ds-default"
  description = "Environment Name"
  type        = string
}

variable "token" {
  description = "Discord Token"
  type        = string
}

variable "openai_token" {
  description = "OpenAI Token"
  type        = string
}

variable "local_ips" {
  default     = []
  description = "Local IPs that can access the machine for debugging"
  type        = list(string)
}