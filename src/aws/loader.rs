//! This module contains a trait that should be implemented by all secret loader implementations.

use crate::vector::{FetchedSecrets, SecretsToFetch};
use async_trait::async_trait;

/// A trait for loading secrets from AWS backends.
#[async_trait]
pub trait LoadSecrets {
    async fn load(&self, secrets: SecretsToFetch) -> FetchedSecrets;
}
