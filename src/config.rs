use crate::download_utils::validate_download_url;
use std::{env, fmt, str::FromStr};

pub const DEFAULT_DB_URL: &str =
    "https://download.maxmind.com/geoip/databases/{VARIANT}/download?suffix=tar.gz";

#[derive(Debug, Clone)]
pub struct Config {
    pub db_variant: String,
    pub db_path: String,
    pub update_interval: u64,
    pub host: String,
    pub port: u16,
    pub swagger_ui_enabled: bool,
    pub download: DownloadConfig,
}

#[derive(Clone)]
pub struct DownloadConfig {
    /// Download URL, `{VARIANT}` is replaced by the database variant
    pub url_template: String,
    pub account_id: Option<String>,
    pub license_key: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        Self::from_lookup(|key| env::var(key).ok())
    }

    pub fn from_lookup(get: impl Fn(&str) -> Option<String>) -> Result<Self, String> {
        let var = |key: &str, default: &str| get(key).unwrap_or_else(|| default.to_string());

        let download = DownloadConfig {
            url_template: var("MAXMIND_DB_DOWNLOAD_URL", DEFAULT_DB_URL),
            account_id: get("MAXMIND_ACCOUNT_ID"),
            license_key: get("MAXMIND_LICENSE_KEY"),
        };

        validate_download_url(&download.url_template)
            .map_err(|reason| format!("Invalid MAXMIND_DB_DOWNLOAD_URL value: {reason}"))?;

        Ok(Self {
            db_variant: var("MAXMIND_DB_VARIANT", "GeoLite2-City"),
            db_path: var("DB_PATH", "/opt/atlas/db"),
            update_interval: parse(&get, "DB_UPDATE_INTERVAL_SECONDS", "86400")?,
            host: var("HOST", "0.0.0.0"),
            port: parse(&get, "PORT", "8080")?,
            swagger_ui_enabled: parse(&get, "SWAGGER_UI_ENABLED", "false")?,
            download,
        })
    }
}

fn parse<T: FromStr>(
    get: &impl Fn(&str) -> Option<String>,
    key: &str,
    default: &str,
) -> Result<T, String> {
    let value = get(key).unwrap_or_else(|| default.to_string());

    value
        .parse()
        .map_err(|_| format!("Invalid {key} value: {value:?}"))
}

impl DownloadConfig {
    pub fn url_for(&self, variant: &str) -> String {
        self.url_template.replace("{VARIANT}", variant)
    }
}

impl Default for DownloadConfig {
    fn default() -> Self {
        Self {
            url_template: DEFAULT_DB_URL.to_string(),
            account_id: None,
            license_key: None,
        }
    }
}

impl fmt::Debug for DownloadConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DownloadConfig")
            .field("url_template", &self.url_template)
            .field("account_id", &self.account_id)
            .field(
                "license_key",
                &self.license_key.as_ref().map(|_| "<redacted>"),
            )
            .finish()
    }
}
