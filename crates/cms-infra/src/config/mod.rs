//! CMS Configuration
//!
//! This crate provides typed configuration for the CMS application,
//! following pattern of a single Config struct with env-file split
//! for dev vs. deploy.
//!
//! Configuration is loaded from:
//! - Environment variables (prefixed with CMS_)
//! - Base config file (`config.toml` or `config/config.toml`)
//! - Environment profile config (`config/dev.toml`, `config/deploy.toml`, etc.)
//! - Custom config path (`CMS_CONFIG_PATH`)

use cms_error::AppError;
use config::{Config as ConfigLib, Environment, File};
use serde::Deserialize;

pub mod analytics;
pub mod auth;
pub mod database;
pub mod mailer;
pub mod mcp;
pub mod queue;
pub mod search;
pub mod security;
pub mod server;
pub mod site;
pub mod storage;

pub use analytics::AnalyticsConfig;
pub use auth::{AuthConfig, OAuthConfig, OAuthProviderConfig};
pub use database::DatabaseConfig;
pub use mailer::MailerConfig;
pub use mcp::McpConfig;
pub use queue::QueueConfig;
pub use search::{RagConfig, SearchConfig};
pub use security::{AdminOriginConfig, RateLimitConfig, SecurityHeadersConfig};
pub use server::ServerConfig;
pub use site::{DomainTlsConfig, DomainVerificationConfig, SiteConfig};
pub use storage::StorageConfig;

/// Main configuration struct for the CMS application
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Config {
    /// Server configuration
    #[serde(default)]
    pub server: ServerConfig,

    /// Database configuration
    #[serde(default)]
    pub database: DatabaseConfig,

    /// Storage configuration
    #[serde(default)]
    pub storage: StorageConfig,

    /// Search configuration
    #[serde(default)]
    pub search: SearchConfig,

    /// Queue configuration
    #[serde(default)]
    pub queue: QueueConfig,

    /// Analytics configuration
    #[serde(default)]
    pub analytics: AnalyticsConfig,

    /// Authentication configuration
    #[serde(default)]
    pub auth: AuthConfig,

    /// Site serving configuration
    #[serde(default)]
    pub site: SiteConfig,

    /// DNS verification for custom domains
    #[serde(default)]
    pub domain_verification: DomainVerificationConfig,

    /// Trusted reverse-proxy signal for custom-domain TLS status reporting.
    #[serde(default)]
    pub domain_tls: DomainTlsConfig,

    /// MCP configuration
    #[serde(default)]
    pub mcp: McpConfig,

    /// Mailer configuration
    #[serde(default)]
    pub mailer: Option<MailerConfig>,

    /// Rate limiting configuration
    #[serde(default)]
    pub rate_limit: RateLimitConfig,

    /// Security headers configuration
    #[serde(default)]
    pub security_headers: SecurityHeadersConfig,

    /// Admin origin configuration
    #[serde(default)]
    pub admin_origin: AdminOriginConfig,

    /// Environment profile ("dev", "deploy", "production", "test", etc.)
    #[serde(default = "default_environment")]
    pub environment: String,
}

fn default_environment() -> String {
    std::env::var("CMS_ENV").unwrap_or_else(|_| "dev".to_string())
}

impl Config {
    /// Check whether this configuration is operating in production/deploy mode.
    pub fn is_production(&self) -> bool {
        let env = self.environment.trim().to_lowercase();
        if matches!(env.as_str(), "deploy" | "production" | "prod") {
            return true;
        }
        if let Ok(env_var) = std::env::var("CMS_ENV") {
            let env_var = env_var.trim().to_lowercase();
            if matches!(env_var.as_str(), "deploy" | "production" | "prod") {
                return true;
            }
        }
        false
    }

    /// Load configuration from the environment and files
    ///
    /// Configuration is loaded in the following cascading order (later sources override earlier ones):
    /// 1. Default values from struct defaults (implicit via serde defaults)
    /// 2. Base configuration file: `config.toml` (or `config/config.toml` if found)
    /// 3. Environment-specific configuration file based on `CMS_ENV`:
    ///    - `config/{CMS_ENV}.toml` (e.g. `config/dev.toml`, `config/deploy.toml`)
    ///    - Fallback: `config/{CMS_ENV}.env`
    /// 4. Explicit configuration file path from `CMS_CONFIG_PATH` env var (if set)
    /// 5. Environment variables prefixed with `CMS_` (using `__` for nested keys)
    pub fn load() -> Result<Self, anyhow::Error> {
        let mut builder = ConfigLib::builder();

        // 1. Base config file (config.toml in root or config/)
        if std::path::Path::new("config.toml").exists() {
            builder = builder.add_source(File::with_name("config.toml"));
        } else if std::path::Path::new("config/config.toml").exists() {
            builder = builder.add_source(File::with_name("config/config.toml"));
        }

        // 2. Environment profile config
        let env = std::env::var("CMS_ENV").unwrap_or_else(|_| "dev".to_string());
        let env_toml = format!("config/{}.toml", env);
        let env_file = format!("config/{}.env", env);

        if std::path::Path::new(&env_toml).exists() {
            builder = builder.add_source(File::with_name(&env_toml));
        } else if std::path::Path::new(&env_file).exists() {
            builder = builder.add_source(File::with_name(&env_file));
        }

        // 3. Explicit config path override via env var
        if let Ok(custom_path) = std::env::var("CMS_CONFIG_PATH") {
            if std::path::Path::new(&custom_path).exists() {
                builder = builder.add_source(File::with_name(&custom_path));
            }
        }

        // 4. Environment variables with CMS_ prefix
        builder = builder.add_source(
            Environment::with_prefix("CMS")
                .prefix_separator("_")
                .separator("__"),
        );

        let settings = builder.build()?;

        let mut config: Config = settings.try_deserialize()?;
        // Keep environment consistent with CMS_ENV if not explicitly specified in sources
        if config.environment.is_empty() || config.environment == "dev" {
            config.environment = env;
        }
        Ok(config)
    }

    /// Load configuration from a specific path, with environment variable overrides
    pub fn load_from_path(path: &str) -> Result<Self, anyhow::Error> {
        let mut builder = ConfigLib::builder();

        if !std::path::Path::new(path).exists() {
            anyhow::bail!("Config file does not exist at path: {}", path);
        }

        builder = builder.add_source(File::with_name(path));

        builder = builder.add_source(
            Environment::with_prefix("CMS")
                .prefix_separator("_")
                .separator("__"),
        );

        let settings = builder.build()?;

        Ok(settings.try_deserialize()?)
    }

    /// Validate configuration invariants.
    ///
    /// In production mode, rejects startup if:
    /// - JWT or session secret equals a known default or is shorter than 32 chars
    /// - Database password is `postgres` or empty
    /// - Queue backend is in-memory
    pub fn validate(&self) -> Result<(), AppError> {
        if self.security_headers.enable_hsts && self.security_headers.hsts_max_age == 0 {
            return Err(AppError::Validation(
                "HSTS max age cannot be 0 when HSTS is enabled".into(),
            ));
        }

        if let Err(err) = self.admin_origin.validate() {
            return Err(AppError::Validation(format!(
                "Invalid admin origin configuration: {err}"
            )));
        }

        if self.is_production() {
            const KNOWN_DEFAULT_SECRETS: &[&str] = &[
                "dev_session_secret_change_in_production",
                "dev_jwt_secret_change_in_production",
                "secret",
                "changeme",
                "password",
                "admin",
                "default",
            ];

            // 1. JWT and session secrets
            for (name, secret) in [
                ("Session", &self.auth.session_secret),
                ("JWT", &self.auth.jwt_secret),
            ] {
                let trimmed = secret.trim();
                if KNOWN_DEFAULT_SECRETS.contains(&trimmed) {
                    return Err(AppError::Validation(format!(
                        "Production configuration error: {name} secret cannot use a known default \
                         development value"
                    )));
                }
                if trimmed.len() < 32 {
                    return Err(AppError::Validation(format!(
                        "Production configuration error: {name} secret must be at least 32 \
                         characters long"
                    )));
                }
            }

            // 2. Database password cannot be empty or 'postgres'
            match extract_db_password(&self.database.url) {
                None => {
                    return Err(AppError::Validation(
                        "Production configuration error: Database password cannot be empty in \
                         production mode"
                            .into(),
                    ));
                }
                Some(ref pw) if pw.is_empty() => {
                    return Err(AppError::Validation(
                        "Production configuration error: Database password cannot be empty in \
                         production mode"
                            .into(),
                    ));
                }
                Some(ref pw) if pw.to_lowercase() == "postgres" => {
                    return Err(AppError::Validation(
                        "Production configuration error: Database password cannot be default \
                         'postgres' in production mode"
                            .into(),
                    ));
                }
                _ => {}
            }

            // 3. Queue backend cannot be in-memory in production mode
            if self.queue.backend.trim().eq_ignore_ascii_case("memory") {
                return Err(AppError::Validation(
                    "Production configuration error: Queue backend cannot be in-memory in \
                     production mode; configure PostgreSQL or durable queue"
                        .into(),
                ));
            }
        }

        Ok(())
    }
}

fn extract_db_password(url_str: &str) -> Option<String> {
    let after_scheme = url_str
        .strip_prefix("postgres://")
        .or_else(|| url_str.strip_prefix("postgresql://"))?;
    if !after_scheme.contains('@') {
        return None;
    }
    let user_info = after_scheme.split('@').next()?;
    let (_, password) = user_info.split_once(':')?;
    Some(password.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.server.port, 3000);
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(
            config.database.url,
            "postgres://postgres:postgres@localhost:5432/cms"
        );
        assert_eq!(config.storage.backend, "local");
        assert_eq!(config.search.backend, "tantivy");
        assert_eq!(config.queue.backend, "postgres");
        assert_eq!(config.analytics.backend, "postgres");
        assert_eq!(
            config.auth.session_secret,
            "dev_session_secret_change_in_production"
        );
        assert!(!config.mcp.enabled);
        assert!(config.rate_limit.enabled);
        assert!(config.security_headers.enable_hsts);
        assert!(config.admin_origin.enforce);
    }

    #[test]
    fn test_server_config_defaults() {
        let server = ServerConfig::default();
        assert_eq!(server.port, 3000);
        assert_eq!(server.host, "0.0.0.0");
        assert!(!server.https);
        assert_eq!(server.trusted_proxy_hops, 1);
    }

    #[test]
    fn test_storage_config_defaults() {
        let storage = StorageConfig::default();
        assert_eq!(storage.backend, "local");
        assert_eq!(storage.local_root, None);
        assert_eq!(storage.s3_endpoint, None);
    }

    #[test]
    fn test_search_config_defaults() {
        let search = SearchConfig::default();
        assert_eq!(search.backend, "tantivy");
        assert_eq!(search.max_results, 50);
        assert!(search.vector_search_enabled);
        assert_eq!(search.embedding_model, "multilingual-e5-small");
        assert!(!search.rag.enabled);
    }

    #[test]
    fn test_queue_config_defaults() {
        let queue = QueueConfig::default();
        assert_eq!(queue.backend, "postgres");
        assert_eq!(queue.workers, 4);
        assert_eq!(queue.max_retries, 3);
    }

    #[test]
    fn test_analytics_config_defaults() {
        let analytics = AnalyticsConfig::default();
        assert_eq!(analytics.backend, "postgres");
        assert_eq!(analytics.clickhouse_port, 8123);
    }

    #[test]
    fn test_rate_limit_config_defaults() {
        let rate_limit = RateLimitConfig::default();
        assert!(rate_limit.enabled);
        assert_eq!(rate_limit.requests_per_second, 100);
        assert_eq!(rate_limit.burst_size, 200);
        assert_eq!(rate_limit.max_tracked_clients, 10_000);
        assert_eq!(rate_limit.client_ttl_secs, 300);
    }

    #[test]
    fn test_security_headers_defaults() {
        let headers = SecurityHeadersConfig::default();
        assert!(headers.enable_hsts);
        assert_eq!(headers.hsts_max_age, 31536000);
        assert!(headers.enable_x_content_type_options);
        assert!(headers.enable_x_frame_options);
        assert_eq!(headers.x_frame_options, "DENY");
    }

    #[test]
    fn test_admin_origin_defaults() {
        let admin_origin = AdminOriginConfig::default();
        assert!(admin_origin.enforce);
        assert!(admin_origin.allow_localhost);
        assert!(
            admin_origin
                .allowed_origins
                .contains(&"https://admin.cms.com".to_string())
        );
    }

    #[test]
    fn test_load_from_toml_content() {
        let toml_data = r#"
            [server]
            port = 8080
            host = "127.0.0.1"

            [database]
            url = "postgres://custom:custom@localhost:5432/custom_db"
            max_pool_size = 50

            [storage]
            backend = "s3"
            s3_bucket = "my-bucket"
            s3_region = "us-west-2"

            [rate_limit]
            enabled = false
            requests_per_second = 500
        "#;

        let builder =
            ConfigLib::builder().add_source(File::from_str(toml_data, config::FileFormat::Toml));
        let settings = builder.build().expect("should parse toml data");
        let config: Config = settings
            .try_deserialize()
            .expect("should deserialize Config");

        assert_eq!(config.server.port, 8080);
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(
            config.database.url,
            "postgres://custom:custom@localhost:5432/custom_db"
        );
        assert_eq!(config.database.max_pool_size, 50);
        assert_eq!(config.storage.backend, "s3");
        assert_eq!(config.storage.s3_bucket.as_deref(), Some("my-bucket"));
        assert_eq!(config.storage.s3_region.as_deref(), Some("us-west-2"));
        assert!(!config.rate_limit.enabled);
        assert_eq!(config.rate_limit.requests_per_second, 500);
        // Untouched sections retain default values
        assert_eq!(config.queue.backend, "postgres");
        assert_eq!(config.search.backend, "tantivy");
    }

    #[test]
    fn test_load_from_path() {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let deploy_path = format!("{}/../../config/deploy.toml", manifest_dir);
        let config = Config::load_from_path(&deploy_path);
        assert!(
            config.is_ok(),
            "loading config/deploy.toml should succeed: {:?}",
            config.err()
        );
        let config = config.unwrap();
        assert_eq!(config.database.max_pool_size, 30);
        assert_eq!(config.queue.workers, 8);
        assert_eq!(config.queue.max_retries, 5);
        assert!(!config.admin_origin.allow_localhost);
    }

    #[test]
    fn test_is_production() {
        let mut config = Config::default();
        assert!(!config.is_production());

        config.environment = "deploy".to_string();
        assert!(config.is_production());

        config.environment = "production".to_string();
        assert!(config.is_production());

        config.environment = "prod".to_string();
        assert!(config.is_production());

        config.environment = "dev".to_string();
        assert!(!config.is_production());
    }

    #[test]
    fn test_env_var_override() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        let toml_path = temp_dir.path().join("test_override.toml");
        std::fs::write(
            &toml_path,
            "[server]\nport = 9999\n[database]\nmax_pool_size = 77\n",
        )
        .expect("write toml");
        let config = Config::load_from_path(toml_path.to_str().unwrap())
            .expect("Config::load_from_path should succeed");
        assert_eq!(config.server.port, 9999);
        assert_eq!(config.database.max_pool_size, 77);
    }

    #[test]
    fn test_load_cascading() {
        let config = Config::load().expect("Config::load should succeed");
        assert_eq!(config.server.port, 3000);
        assert_eq!(
            config.database.url,
            "postgres://postgres:postgres@localhost:5432/cms"
        );
        assert_eq!(config.storage.backend, "local");
    }

    fn valid_production_config() -> Config {
        Config {
            environment: "deploy".to_string(),
            auth: AuthConfig {
                session_secret: "a_very_secure_and_long_session_secret_for_production_32chars"
                    .to_string(),
                jwt_secret: "a_very_secure_and_long_jwt_secret_for_production_32chars".to_string(),
                ..Default::default()
            },
            database: DatabaseConfig {
                url: "postgres://cms_user:strong_prod_password@localhost:5432/cms".to_string(),
                ..Default::default()
            },
            queue: QueueConfig {
                backend: "postgres".to_string(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[test]
    fn test_validate_config_dev_mode_passes_defaults() {
        let config = Config::default();
        assert!(!config.is_production());
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_config_production_valid() {
        let config = valid_production_config();
        assert!(config.is_production());
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_config_production_rejects_default_session_secret() {
        let mut config = valid_production_config();
        config.auth.session_secret = "dev_session_secret_change_in_production".to_string();
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Session secret"));
    }

    #[test]
    fn test_validate_config_production_rejects_short_jwt_secret() {
        let mut config = valid_production_config();
        config.auth.jwt_secret = "short_secret".to_string();
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("32 characters"));
    }

    #[test]
    fn test_validate_config_production_rejects_postgres_db_password() {
        let mut config = valid_production_config();
        config.database.url = "postgres://postgres:postgres@localhost:5432/cms".to_string();
        let result = config.validate();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("default 'postgres'")
        );
    }

    #[test]
    fn test_validate_config_production_rejects_empty_db_password() {
        let mut config = valid_production_config();
        config.database.url = "postgres://cms_user@localhost:5432/cms".to_string();
        let result = config.validate();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("password cannot be empty")
        );
    }

    #[test]
    fn test_validate_config_production_rejects_in_memory_queue() {
        let mut config = valid_production_config();
        config.queue.backend = "memory".to_string();
        let result = config.validate();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Queue backend cannot be in-memory")
        );
    }
}
