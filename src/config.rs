use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

  #[derive(Serialize, Deserialize, Clone)]
  pub struct Config {
      pub api_url: String,
      pub api_key: String,
      pub model: String,
      pub prompt: String,
  }

  impl Default for Config {
      fn default() -> Self {
          Self {
              api_url: "http://llm-api.fixstars.com/v1".to_string(),
              api_key: "".to_string(),
              model: "latest-chat".to_string(),
              prompt: "If the text is in English, translate to Japanese. If in Japanese, translate to English.
  Only output the translation.".to_string(),
          }
      }
  }

  impl Config {
      pub fn config_path() -> Option<PathBuf> {
          let proj_dirs = ProjectDirs::from("com", "translate", "translate_tool")?;
          let config_dir = proj_dirs.config_dir();
          fs::create_dir_all(config_dir).ok()?;
          Some(config_dir.join("config.json"))
      }

      pub fn load() -> Self {
          Self::config_path()
              .and_then(|path| fs::read_to_string(path).ok())
              .and_then(|content| serde_json::from_str(&content).ok())
              .unwrap_or_default()
      }

      pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
          let path = Self::config_path().ok_or("Could not determine config path")?;
          let content = serde_json::to_string_pretty(self)?;
          fs::write(path, content)?;
          Ok(())
      }
  }