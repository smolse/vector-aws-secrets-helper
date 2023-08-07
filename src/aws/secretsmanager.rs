//! This module contains the secrets loader implementation for AWS Secrets Manager.

use crate::vector::{FetchedSecret, FetchedSecrets, SecretsToFetch};
use crate::LoadSecrets;
use async_trait::async_trait;
use aws_sdk_secretsmanager::error::SdkError::ServiceError;
use aws_sdk_secretsmanager::Client;

/// A trait for fetching a single secret from AWS Secrets Manager.
#[async_trait]
pub trait SecretsManagerFetchSecret {
    async fn fetch_secret(&self, name: String) -> FetchedSecret;
}

/// Implement the SecretsManagerGetSecret trait for the AWS SDK Secrets Manager client.
#[async_trait]
impl SecretsManagerFetchSecret for Client {
    async fn fetch_secret(&self, name: String) -> FetchedSecret {
        match self.get_secret_value().secret_id(name).send().await {
            Ok(response) => match response.secret_string {
                Some(secret) => FetchedSecret {
                    value: Some(secret),
                    error: None,
                },
                None => FetchedSecret {
                    value: None,
                    error: Some(String::from("secret not found")),
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

/// A struct for loading secrets from AWS Secrets Manager.
pub struct SecretsManagerSecretsLoader {
    client: Box<dyn SecretsManagerFetchSecret + Send + Sync>,
}

/// Implement the SecretsManagerSecretsLoader constructor.
impl SecretsManagerSecretsLoader {
    pub fn new(client: impl SecretsManagerFetchSecret + Send + Sync + 'static) -> Self {
        Self {
            client: Box::new(client),
        }
    }
}

/// Implement the LoadSecrets trait for SecretsManagerSecretsLoader.
#[async_trait]
impl LoadSecrets for SecretsManagerSecretsLoader {
    async fn load(&self, secrets: SecretsToFetch) -> FetchedSecrets {
        let create_task = |secret_name: String| {
            let secret_to_fetch = secret_name.clone();
            let task = async { self.client.fetch_secret(secret_to_fetch).await };
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
        struct MockSecretsManagerFetchSecret {}

        #[async_trait]
        impl SecretsManagerFetchSecret for MockSecretsManagerFetchSecret {
            async fn fetch_secret(&self, name: String) -> FetchedSecret {
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

        let secrets_loader = SecretsManagerSecretsLoader::new(MockSecretsManagerFetchSecret {});
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
