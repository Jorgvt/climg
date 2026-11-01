use crate::cli::ProtocolChoice;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub protocol: Option<ProtocolChoice>,
    pub quiet: Option<bool>,
    pub max_width: Option<String>,
    pub max_height: Option<String>,
}

impl AppConfig {
    /// Return the path to the config file: $XDG_CONFIG_HOME/climg/config.toml or ~/.config/climg/config.toml
    pub fn config_path() -> Option<PathBuf> {
        if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
            if !xdg.is_empty() {
                return Some(PathBuf::from(xdg).join("climg").join("config.toml"));
            }
        }
        if let Ok(home) = std::env::var("HOME") {
            if !home.is_empty() {
                return Some(PathBuf::from(home).join(".config").join("climg").join("config.toml"));
            }
        }
        None
    }

    /// Load the configuration file if it exists, otherwise return default empty config
    pub fn load() -> Self {
        Self::config_path()
            .and_then(|p| Self::load_from_path(&p).ok())
            .unwrap_or_default()
    }

    pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: AppConfig = toml::from_str(&content)?;
        Ok(config)
    }

    /// Write an initial configuration file with the specified protocol
    pub fn init_config(protocol: Option<ProtocolChoice>) -> Result<PathBuf> {
        let path = Self::config_path()
            .context("Could not determine user configuration directory ($HOME or $XDG_CONFIG_HOME)")?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {}", parent.display()))?;
        }

        let proto_str = match protocol {
            Some(ProtocolChoice::Kitty) => "\"kitty\"",
            Some(ProtocolChoice::Iterm2) => "\"iterm2\"",
            Some(ProtocolChoice::Auto) | None => "\"kitty\"",
        };

        let content = format!(
            r#"# climg configuration file
# Location: {}

# Default graphics protocol: "kitty", "iterm2", or "auto"
protocol = {}

# Suppress metadata banner by default (uncomment to enable)
# quiet = true

# Default max sizing constraints (uncomment to enable)
# max_width = "80"
# max_height = "30"
"#,
            path.display(),
            proto_str
        );

        fs::write(&path, content)
            .with_context(|| format!("Failed to write configuration file: {}", path.display()))?;

        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_config() {
        let toml_str = r#"
            protocol = "kitty"
            quiet = true
            max_width = "80"
        "#;
        let config: AppConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.protocol, Some(ProtocolChoice::Kitty));
        assert_eq!(config.quiet, Some(true));
        assert_eq!(config.max_width, Some("80".to_string()));
    }
}
