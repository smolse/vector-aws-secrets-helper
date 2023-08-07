//! This module contains the secrets loader implementation for AWS SSM Parameter Store.

use crate::vector::{FetchedSecret, FetchedSecrets, SecretsToFetch};
use crate::LoadSecrets;
use async_trait::async_trait;
use aws_sdk_ssm::error::SdkError::ServiceError;
use aws_sdk_ssm::Client;

/// A trait for fetching a single secret from AWS SSM Parameter Store.
#[async_trait]
pub trait SsmFetchSecret {
    async fn fetch_secret(&self, name: String, with_decryption: bool) -> FetchedSecret;
}

/// Implement the SsmFetchSecret trait for the AWS SDK SSM Parameter Store client.
#[async_trait]
impl SsmFetchSecret for Client {
    async fn fetch_secret(&self, name: String, with_decryption: bool) -> FetchedSecret {
        match self
            .get_parameter()
            .name(name)
            .with_decryption(with_decryption)
            .send()
            .await
        {
            Ok(response) => match response.parameter {
                Some(parameter) => match parameter.value {
                    Some(value) => FetchedSecret {
                        value: Some(value),
                        error: None,
                    },
                    None => FetchedSecret {
                        value: None,
                        error: Some(String::from("parameter value not found")),
                    },
                },
                None => FetchedSecret {
                    value: None,
                    error: Some(String::from("parameter not found")),
                },
            },
            Err(error) => match error {
                ServiceError(error) => FetchedSecret {
                    value: None,
                    error: Some(format!("service error: {}", error.into_err())),
                },
                _ => FetchedSecret {
                    value: None,
                    error: Some(error.to_string()),
                },
            },
        }
    }
}

/// A struct for loading secrets from AWS SSM Parameter Store.
pub struct SsmSecretsLoader {
    client: Box<dyn SsmFetchSecret + Send + Sync>,
    with_decryption: bool,
}

/// Implement the SsmSecretsLoader constructor.
impl SsmSecretsLoader {
    pub fn new(client: impl SsmFetchSecret + Send + Sync + 'static, with_decryption: bool) -> Self {
        Self {
            client: Box::new(client),
            with_decryption,
        }
    }
}

/// Implement the LoadSecrets trait for SsmSecretsLoader.
#[async_trait]
impl LoadSecrets for SsmSecretsLoader {
    async fn load(&self, secrets: SecretsToFetch) -> FetchedSecrets {
        let create_task = |secret_name: String| {
            let secret_to_fetch = secret_name.clone();
            let task = async {
                self.client
                    .fetch_secret(secret_to_fetch, self.with_decryption)
                    .await
            };
            (secret_name, task)
        };

        // Run tasks concurrently.
        let (secret_names, tasks): (Vec<_>, Vec<_>) =
            secrets.secrets.into_iter().map(create_task).unzip();
        let results: Vec<_> = futures::future::join_all(tasks).await;

        // Create a FetchedSecrets struct from the results.
        let mut fetched_secrets = FetchedSecrets::default();
        secret_names
            .into_iter()
            .zip(results)
            .for_each(|(secret_name, result)| {
                fetched_secrets.0.insert(secret_name, result);
            });

        fetched_secrets
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn ssm_secrets_loader_loads_secrets() {
        struct MockSsmFetchSecret {}

        #[async_trait]
        impl SsmFetchSecret for MockSsmFetchSecret {
            async fn fetch_secret(&self, name: String, _with_decryption: bool) -> FetchedSecret {
                match name.as_str() {
                    "test.secret_1" => FetchedSecret {
                        value: Some("qwerty".to_string()),
                        error: None,
                    },
                    "test.secret_2" => FetchedSecret {
                        value: None,
                        error: Some("failed to fetch".to_string()),
                    },
                    _ => unreachable!(),
                }
            }
        }

        let secrets_to_fetch = SecretsToFetch {
            version: String::from("1.0"),
            secrets: vec![String::from("test.secret_1"), String::from("test.secret_2")],
        };

        let secrets_loader = SsmSecretsLoader::new(MockSsmFetchSecret {}, true);
        let fetched_secrets = secrets_loader.load(secrets_to_fetch).await;

        assert_eq!(
            fetched_secrets,
            FetchedSecrets(
                [
                    (
                        "test.secret_1".to_string(),
                        FetchedSecret {
                            value: Some("qwerty".to_string()),
                            error: None,
                        }
                    ),
                    (
                        "test.secret_2".to_string(),
                        FetchedSecret {
                            value: None,
                            error: Some("failed to fetch".to_string()),
                        }
                    )
                ]
                .iter()
                .cloned()
                .collect()
            )
        );
    }
}
