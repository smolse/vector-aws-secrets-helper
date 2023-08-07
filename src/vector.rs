//! This module contains structs for communicating with Vector.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A struct representing the JSON input from Vector.
#[derive(Debug, Serialize, Deserialize)]
pub struct SecretsToFetch {
    pub version: String,
    pub secrets: Vec<String>,
}

/// A struct representing a single secret value retrieved from the target backend.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FetchedSecret {
    pub value: Option<String>,
    pub error: Option<String>,
}

/// A struct representing the JSON output to Vector.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FetchedSecrets(pub HashMap<String, FetchedSecret>);

/// Implement the Default trait for FetchedSecrets.
impl Default for FetchedSecrets {
    fn default() -> Self {
        Self(HashMap::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secrets_json_string_passed_by_vector_can_be_deserialized() {
        let secrets_json_string = "{\"version\": \"1.0\", \"secrets\": [\"some_secret\"]}";
        let secrets_to_fetch: SecretsToFetch = serde_json::from_str(secrets_json_string).unwrap();
        assert_eq!(secrets_to_fetch.version, "1.0");
        assert_eq!(secrets_to_fetch.secrets, vec!["some_secret"]);
    }

    #[test]
    fn fetched_secrets_success_struct_serialization() {
        let mut fetched_secrets = FetchedSecrets::default();
        fetched_secrets.0.insert(
            "/test/secret_1".to_string(),
            FetchedSecret {
                value: Some("qwerty".to_string()),
                error: None,
            },
        );

        let expected_output = "{\"/test/secret_1\":{\"value\":\"qwerty\",\"error\":null}}";
        let output = serde_json::to_string(&fetched_secrets).unwrap();
        assert_eq!(output, expected_output);
    }

    #[test]
    fn fetched_secrets_failure_struct_serialization() {
        let mut fetched_secrets = FetchedSecrets::default();
        fetched_secrets.0.insert(
            "/test/secret_2".to_string(),
            FetchedSecret {
                value: None,
                error: Some("failed to fetch".to_string()),
            },
        );

        let expected_output = "{\"/test/secret_2\":{\"value\":null,\"error\":\"failed to fetch\"}}";
        let output = serde_json::to_string(&fetched_secrets).unwrap();
        assert_eq!(output, expected_output);
    }
}
