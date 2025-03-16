use serde::{Deserialize, Serialize};
use tokio::fs;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub patterns: Vec<String>,
    pub ignore_paths: Vec<String>,
    pub sites: Vec<String>,
}

pub async fn load_config(file_path: &str) -> Option<Config> {
    let content = fs::read_to_string(file_path).await.ok()?;
    toml::from_str(&content).ok()
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
            "patterns = [\"/first/:slug\", \"/second/:slug\"]\nignore_paths = [\"/ignore-this\"]\nsites = [\"https://example.com\"]"
        )
        .expect("Failed to write to temp file");
        let file_path = temp_file.path().to_str().unwrap();

        let config = load_config(file_path).await.unwrap();
        assert_eq!(config.patterns.len(), 2);
        assert_eq!(config.ignore_paths.len(), 1);
        assert_eq!(config.patterns[0], "/first/:slug");
        assert_eq!(config.ignore_paths[0], "/ignore-this");
    }

    #[test]
    async fn load_invalid_config_file() {
        let mut temp_file = NamedTempFile::new().expect("Failed to create tempfile");
        writeln!(temp_file, "pattern [\"/first/:slug\"]").expect("Failed to write to tempfile");
        let file_path = temp_file.path().to_str().unwrap();

        let config = load_config(file_path).await;
        assert!(config.is_none());
    }

    #[test]
    async fn missing_config_file() {
        let file_path = "missing.toml";
        let config = load_config(file_path).await;
        assert!(config.is_none());
    }
}
