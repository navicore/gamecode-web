use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub server: ServerConfig,
    pub auth: AuthConfig,
    pub providers: ProvidersConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub port: u16,
    pub static_dir: String,
    pub max_request_size: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthConfig {
    pub password_hash: String,  // Argon2 hash of the shared password
    pub jwt_secret: String,
    pub session_duration_hours: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProvidersConfig {
    pub ollama: Option<OllamaConfig>,
    // Future: bedrock, openai, etc.
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OllamaConfig {
    pub enabled: bool,
    pub base_url: String,
    pub models: Vec<String>,
    pub default_model: String,
    pub timeout_seconds: u64,
}

impl Config {
    pub fn load() -> Result<Self> {
        // Try environment variable first
        if let Ok(config_path) = std::env::var("GAMECODE_CONFIG") {
            Self::from_file(&config_path)
        } else {
            // Default config path
            Self::from_file("config/default.toml")
        }
    }

    fn from_file(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                port: 8080,
                static_dir: "dist".to_string(),
                max_request_size: 10 * 1024 * 1024, // 10MB
            },
            auth: AuthConfig {
                // This is "gamecode" hashed - CHANGE IN PRODUCTION!
                password_hash: "$argon2id$v=19$m=19456,t=2,p=1$VE0Yc3hKakUwZWhqazhEMg$Rvzj1F8qRvLiDZ2bxiXPYdjUzZ3S4E8uFz2dMcLbKj0".to_string(),
                jwt_secret: "change-me-in-production".to_string(),
                session_duration_hours: 24,
            },
            providers: ProvidersConfig {
                ollama: Some(OllamaConfig {
                    enabled: true,
                    base_url: "http://localhost:11434".to_string(),
                    models: vec!["fortean".to_string()],
                    default_model: "fortean".to_string(),
                    timeout_seconds: 300,
                }),
            },
        }
    }
}