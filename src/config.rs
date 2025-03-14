use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::fs;

#[derive(Serialize, Deserialize)]
struct Config {
    patterns: Vec<String>,
    ignore: Vec<String>,
}

pub async fn load_config(file_path: &str) -> Result<Config> {
    let content = fs::read_to_string(file_path)
        .await
        .context("Config file not found")?;
    let config = toml::from_str(&content).context("Unable to parse config.toml")?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    use tokio::test;

    #[test]
    async fn load_valid_config_file() {
        let mut temp_file = NamedTempFile::new().expect("Failed to create tempfile");
        writeln!(
            temp_file,
            "patterns = [\"/first/:slug\", \"/second/:slug\"]\nignore = [\"/ignore-this\"]"
        )
        .expect("Failed to write to temp file");
        let file_path = temp_file.path().to_str().unwrap();

        let config = load_config(file_path).await.unwrap();
        assert_eq!(config.patterns.len(), 2);
        assert_eq!(config.ignore.len(), 1);
        assert_eq!(config.patterns[0], "/first/:slug");
        assert_eq!(config.ignore[0], "/ignore-this");
    }

    #[test]
    async fn load_invalid_config_file() {
        let mut temp_file = NamedTempFile::new().expect("Failed to create tempfile");
        writeln!(temp_file, "pattern [\"/first/:slug\"]").expect("Failed to write to tempfile");
        let file_path = temp_file.path().to_str().unwrap();

        let config = load_config(file_path).await;
        assert!(config.is_err());
    }

    #[test]
    async fn missing_config_file() {
        let file_path = "missing.toml";
        let config = load_config(file_path).await;
        assert!(config.is_err());
    }
}
