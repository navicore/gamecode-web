use anyhow::{anyhow, Context, Result};
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHasher,
};
use rand::{distributions::Alphanumeric, Rng};
use std::env;
use tracing::warn;

#[derive(Debug, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub auth: AuthConfig,
    pub providers: ProvidersConfig,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub port: u16,
    pub static_dir: String,
    pub max_request_size: usize,
}

#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub password_hash: String,
    pub jwt_secret: String,
    pub session_duration_hours: u64,
}

#[derive(Debug, Clone)]
pub struct ProvidersConfig {
    pub ollama: Option<OllamaConfig>,
}

#[derive(Debug, Clone)]
pub struct OllamaConfig {
    pub enabled: bool,
    pub base_url: String,
    pub default_model: String,
    pub timeout_seconds: u64,
}

impl Config {
    pub fn load() -> Result<Self> {
        let plaintext_password = env::var("GAMECODE_AUTH_PASSWORD")
            .context("GAMECODE_AUTH_PASSWORD must be set")?;
        let password_hash = hash_password(&plaintext_password)?;

        let jwt_secret = match env::var("GAMECODE_AUTH_JWT_SECRET") {
            Ok(s) if !s.is_empty() => s,
            _ => {
                warn!(
                    "GAMECODE_AUTH_JWT_SECRET not set — generating ephemeral secret; \
                     all sessions invalidate on restart"
                );
                generate_random_secret(48)
            }
        };

        let ollama_enabled = parse_env("GAMECODE_OLLAMA_ENABLED", true);
        let ollama = if ollama_enabled {
            Some(OllamaConfig {
                enabled: true,
                base_url: env::var("GAMECODE_OLLAMA_BASE_URL")
                    .context("GAMECODE_OLLAMA_BASE_URL must be set when ollama is enabled")?,
                default_model: env::var("GAMECODE_OLLAMA_DEFAULT_MODEL")
                    .context("GAMECODE_OLLAMA_DEFAULT_MODEL must be set when ollama is enabled")?,
                timeout_seconds: parse_env("GAMECODE_OLLAMA_TIMEOUT_SECONDS", 60u64),
            })
        } else {
            None
        };

        Ok(Config {
            server: ServerConfig {
                port: parse_env("GAMECODE_SERVER_PORT", 8080u16),
                static_dir: env::var("GAMECODE_SERVER_STATIC_DIR")
                    .unwrap_or_else(|_| "dist".to_string()),
                max_request_size: parse_env("GAMECODE_SERVER_MAX_REQUEST_SIZE", 10 * 1024 * 1024),
            },
            auth: AuthConfig {
                password_hash,
                jwt_secret,
                session_duration_hours: parse_env("GAMECODE_AUTH_SESSION_DURATION_HOURS", 24u64),
            },
            providers: ProvidersConfig { ollama },
        })
    }
}

fn hash_password(plaintext: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(plaintext.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| anyhow!("failed to hash password: {e}"))
}

fn generate_random_secret(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

fn parse_env<T: std::str::FromStr>(key: &str, default: T) -> T {
    env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}
