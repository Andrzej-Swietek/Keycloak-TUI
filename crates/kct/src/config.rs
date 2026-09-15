use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::{Context as _, Result};
use serde::Deserialize;

/// Saved connection details, in the style of a kubeconfig: several servers, one of them the
/// default, and never a password on disk.
#[derive(Debug, Default, Deserialize)]
pub struct Config {
    pub default: Option<String>,
    #[serde(default)]
    pub contexts: BTreeMap<String, ContextConfig>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ContextConfig {
    pub server: Option<String>,
    pub realm: Option<String>,
    pub auth_realm: Option<String>,
    pub username: Option<String>,
    pub client_id: Option<String>,
    /// Only for service accounts whose secret is already a machine credential. A human
    /// password belongs in the environment or the prompt, never here.
    pub client_secret: Option<String>,
    #[serde(default)]
    pub insecure: bool,
    pub ca_cert: Option<PathBuf>,
    pub screen: Option<String>,
}

impl Config {
    pub fn path() -> Option<PathBuf> {
        directories::ProjectDirs::from("", "", "kct")
            .map(|dirs| dirs.config_dir().join("config.toml"))
    }

    pub fn load() -> Result<Self> {
        let Some(path) = Self::path() else {
            return Ok(Self::default());
        };
        if !path.exists() {
            return Ok(Self::default());
        }

        let text = std::fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        toml::from_str(&text).with_context(|| format!("parsing {}", path.display()))
    }

    /// Resolve a context by name, or the configured default.
    pub fn select(&self, name: Option<&str>) -> Result<ContextConfig> {
        let chosen = name.or(self.default.as_deref());

        match chosen {
            None => Ok(ContextConfig::default()),
            Some(name) => self.contexts.get(name).cloned().with_context(|| {
                format!(
                    "no context named '{name}'; configured: {}",
                    self.names().join(", ")
                )
            }),
        }
    }

    pub fn names(&self) -> Vec<&str> {
        self.contexts.keys().map(String::as_str).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_context_names_the_ones_that_exist() {
        let config: Config = toml::from_str(
            r#"
            default = "dev"
            [contexts.dev]
            server = "http://localhost:8080"
            [contexts.prod]
            server = "https://sso.example.com"
            "#,
        )
        .unwrap();

        assert_eq!(
            config.select(None).unwrap().server.unwrap(),
            "http://localhost:8080"
        );

        let error = config.select(Some("staging")).unwrap_err().to_string();
        assert!(error.contains("dev"), "{error}");
        assert!(error.contains("prod"), "{error}");
    }

    #[test]
    fn an_empty_config_is_not_an_error() {
        let config: Config = toml::from_str("").unwrap();
        assert!(config.select(None).unwrap().server.is_none());
    }
}
