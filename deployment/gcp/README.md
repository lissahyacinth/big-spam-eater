# Spam Bot GCP Stack

A stack for running a Spam Bot within GCP. 

## Warnings
This is pretty simplistic - we're capturing the token within a variable and so it'll be available within the state. Keeping it primarily in remote state is a benefit, but it's still not optimal. Expanding this would KMS encrypt the token so that only the container at the other end could decrypt it, but having Kotel handle KMS decryption wasn't trivial.

## Documentation

<!-- BEGIN_TF_DOCS -->
## Requirements

| Name | Version |
|------|---------|
| <a name="requirement_terraform"></a> [terraform](#requirement\_terraform) | ~> 1.3.0 |
| <a name="requirement_google"></a> [google](#requirement\_google) | ~> 5.21.0 |

## Providers

| Name | Version |
|------|---------|
| <a name="provider_google"></a> [google](#provider\_google) | ~> 5.21.0 |
| <a name="provider_local"></a> [local](#provider\_local) | n/a |
| <a name="provider_random"></a> [random](#provider\_random) | n/a |

## Modules

| Name | Source | Version |
|------|--------|---------|
| <a name="module_gce-container"></a> [gce-container](#module\_gce-container) | terraform-google-modules/container-vm/google | ~> 3.0 |
| <a name="module_vpc"></a> [vpc](#module\_vpc) | terraform-google-modules/network/google | ~> 9.1 |

## Resources

| Name | Type |
|------|------|
| [google_compute_firewall.firewall-ssh](https://registry.terraform.io/providers/hashicorp/google/latest/docs/resources/compute_firewall) | resource |
| [google_compute_instance.vm](https://registry.terraform.io/providers/hashicorp/google/latest/docs/resources/compute_instance) | resource |
| [google_secret_manager_secret.discord_token_basic](https://registry.terraform.io/providers/hashicorp/google/latest/docs/resources/secret_manager_secret) | resource |
| [google_secret_manager_secret.openai_token_basic](https://registry.terraform.io/providers/hashicorp/google/latest/docs/resources/secret_manager_secret) | resource |
| [google_secret_manager_secret_version.openai_token_version](https://registry.terraform.io/providers/hashicorp/google/latest/docs/resources/secret_manager_secret_version) | resource |
| [google_secret_manager_secret_version.token_version](https://registry.terraform.io/providers/hashicorp/google/latest/docs/resources/secret_manager_secret_version) | resource |
| [google_service_account.compute_account](https://registry.terraform.io/providers/hashicorp/google/latest/docs/resources/service_account) | resource |
| [google_storage_bucket.default](https://registry.terraform.io/providers/hashicorp/google/latest/docs/resources/storage_bucket) | resource |
| [local_file.default](https://registry.terraform.io/providers/hashicorp/local/latest/docs/resources/file) | resource |
| [random_id.default](https://registry.terraform.io/providers/hashicorp/random/latest/docs/resources/id) | resource |

## Inputs

| Name | Description | Type | Default | Required |
|------|-------------|------|---------|:--------:|
| <a name="input_env"></a> [env](#input\_env) | Environment Name | `string` | `"ds-default"` | no |
| <a name="input_image"></a> [image](#input\_image) | Full Docker Image Name | `string` | `"ghcr.io/lissahyacinth/big-spam-eater:main"` | no |
| <a name="input_instance_name"></a> [instance\_name](#input\_instance\_name) | The desired name to assign to the deployed instance | `string` | `"spam-eater"` | no |
| <a name="input_labels"></a> [labels](#input\_labels) | Labels for deployed instance | `map(any)` | `{}` | no |
| <a name="input_local_ips"></a> [local\_ips](#input\_local\_ips) | Local IPs that can access the machine for debugging | `list(string)` | `[]` | no |
| <a name="input_machine_type"></a> [machine\_type](#input\_machine\_type) | GCP Machine Type | `string` | `"e2-micro"` | no |
| <a name="input_openai_token"></a> [openai\_token](#input\_openai\_token) | OpenAI Token | `string` | n/a | yes |
| <a name="input_project_id"></a> [project\_id](#input\_project\_id) | The project ID to deploy resources into | `string` | n/a | yes |
| <a name="input_region"></a> [region](#input\_region) | The GCP region to deploy instances into | `string` | n/a | yes |
| <a name="input_token"></a> [token](#input\_token) | Discord Token | `string` | n/a | yes |
| <a name="input_zone"></a> [zone](#input\_zone) | The GCP zone to deploy instances into | `string` | n/a | yes |

## Outputs

| Name | Description |
|------|-------------|
| <a name="output_instance_name"></a> [instance\_name](#output\_instance\_name) | The deployed instance name |
| <a name="output_ipv4"></a> [ipv4](#output\_ipv4) | The public IP address of the deployed instance |
| <a name="output_vm_container_label"></a> [vm\_container\_label](#output\_vm\_container\_label) | The instance label containing container configuration |
| <a name="output_volumes"></a> [volumes](#output\_volumes) | The volume metadata provided to the module |
<!-- END_TF_DOCS -->