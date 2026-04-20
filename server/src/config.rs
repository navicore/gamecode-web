use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use std::env;

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
    pub oidc: OidcConfig,
    pub session_key: [u8; 32],
}

#[derive(Debug, Clone)]
pub struct OidcConfig {
    pub issuer: String,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub scopes: String,
}

#[derive(Debug, Clone)]
pub struct ProvidersConfig {
    pub ollama: Option<OllamaConfig>,
}

#[derive(Debug, Clone)]
pub struct OllamaConfig {
    pub enabled: bool,
    pub base_url: String,
    pub default_model: Option<String>,
    pub timeout_seconds: u64,
}

impl Config {
    pub fn load() -> Result<Self> {
        let oidc = OidcConfig {
            issuer: require("GAMECODE_AUTH_OIDC_ISSUER_URL")?,
            client_id: require("GAMECODE_AUTH_OIDC_CLIENT_ID")?,
            client_secret: require("GAMECODE_AUTH_OIDC_CLIENT_SECRET")?,
            redirect_uri: require("GAMECODE_AUTH_OIDC_REDIRECT_URI")?,
            scopes: require("GAMECODE_AUTH_OIDC_SCOPES")?,
        };
        let session_key = decode_session_key(&require("GAMECODE_AUTH_SESSION_KEY")?)?;

        let ollama_enabled = parse_env("GAMECODE_OLLAMA_ENABLED", true);
        let ollama = if ollama_enabled {
            Some(OllamaConfig {
                enabled: true,
                base_url: env::var("GAMECODE_OLLAMA_BASE_URL")
                    .context("GAMECODE_OLLAMA_BASE_URL must be set when ollama is enabled")?,
                default_model: env::var("GAMECODE_OLLAMA_DEFAULT_MODEL")
                    .ok()
                    .filter(|v| !v.is_empty()),
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
            auth: AuthConfig { oidc, session_key },
            providers: ProvidersConfig { ollama },
        })
    }
}

fn require(key: &str) -> Result<String> {
    match env::var(key) {
        Ok(v) if !v.is_empty() => Ok(v),
        _ => Err(anyhow!("{key} must be set")),
    }
}

fn decode_session_key(encoded: &str) -> Result<[u8; 32]> {
    let raw = B64
        .decode(encoded.trim())
        .context("GAMECODE_AUTH_SESSION_KEY must be base64")?;
    <[u8; 32]>::try_from(raw.as_slice())
        .map_err(|_| anyhow!("GAMECODE_AUTH_SESSION_KEY must decode to exactly 32 bytes"))
}

fn parse_env<T: std::str::FromStr>(key: &str, default: T) -> T {
    env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}
