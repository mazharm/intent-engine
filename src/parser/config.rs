//! Configuration file parsing (intent.toml)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Path to the configuration file
pub const CONFIG_FILE: &str = "intent.toml";

/// Project configuration from intent.toml
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IntentConfig {
    #[serde(default)]
    pub project: ProjectConfig,

    #[serde(default)]
    pub generation: GenerationConfig,

    #[serde(default)]
    pub runtime: RuntimeConfig,

    #[serde(default)]
    pub environments: EnvironmentsConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectConfig {
    #[serde(default)]
    pub name: String,

    #[serde(default)]
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationConfig {
    #[serde(default = "default_rust_edition")]
    pub rust_edition: String,
}

fn default_rust_edition() -> String {
    "2021".to_string()
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            rust_edition: default_rust_edition(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    #[serde(default = "default_http_client")]
    pub http_client: String,

    #[serde(default = "default_db_client")]
    pub db_client: String,

    #[serde(default = "default_event_client")]
    pub event_client: String,
}

fn default_http_client() -> String {
    "reqwest".to_string()
}

fn default_db_client() -> String {
    "sqlx".to_string()
}

fn default_event_client() -> String {
    "kafka".to_string()
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            http_client: default_http_client(),
            db_client: default_db_client(),
            event_client: default_event_client(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EnvironmentsConfig {
    #[serde(default)]
    pub default: String,

    #[serde(flatten)]
    pub environments: HashMap<String, HashMap<String, String>>,
}

impl IntentConfig {
    /// Load configuration from the default path
    pub fn load() -> anyhow::Result<Self> {
        Self::load_from_path(CONFIG_FILE)
    }

    /// Load configuration from a specific path
    pub fn load_from_path(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = path.as_ref();

        if !path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(path)?;
        let config: IntentConfig = toml::from_str(&content)?;

        Ok(config)
    }

    /// Get an environment variable value for a given environment
    pub fn get_env_value(&self, env: &str, key: &str) -> Option<&String> {
        self.environments
            .environments
            .get(env)
            .and_then(|e| e.get(key))
    }

    /// Get the default environment name
    pub fn default_env(&self) -> &str {
        if self.environments.default.is_empty() {
            "dev"
        } else {
            &self.environments.default
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = IntentConfig::default();
        assert_eq!(config.generation.rust_edition, "2021");
        assert_eq!(config.runtime.http_client, "reqwest");
        assert_eq!(config.runtime.db_client, "sqlx");
    }

    #[test]
    fn test_parse_config() {
        let toml = r#"
[project]
name = "test-service"
version = "1.0.0"

[generation]
rust_edition = "2021"

[runtime]
http_client = "reqwest"
db_client = "sqlx"
event_client = "kafka"

[environments]
default = "dev"

[environments.dev]
"Payments.base_url" = "http://localhost:8080"
"#;

        let config: IntentConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.project.name, "test-service");
        assert_eq!(config.environments.default, "dev");
    }
}
