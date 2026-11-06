use atlas_rs::config::{Config, DEFAULT_DB_URL, DownloadConfig};
use std::collections::HashMap;

fn config_from(vars: &[(&str, &str)]) -> Result<Config, String> {
    let vars: HashMap<String, String> = vars
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    Config::from_lookup(|key| vars.get(key).cloned())
}

#[test]
fn test_defaults() {
    let config = config_from(&[]).unwrap();

    assert_eq!(config.db_variant, "GeoLite2-City");
    assert_eq!(config.db_path, "/opt/atlas/db");
    assert_eq!(config.update_interval, 86400);
    assert_eq!(config.host, "0.0.0.0");
    assert_eq!(config.port, 8080);
    assert!(!config.swagger_ui_enabled);
    assert_eq!(config.download.url_template, DEFAULT_DB_URL);
    assert_eq!(config.download.account_id, None);
    assert_eq!(config.download.license_key, None);
}

#[test]
fn test_overrides() {
    let config = config_from(&[
        ("MAXMIND_DB_VARIANT", "GeoLite2-ASN"),
        ("DB_PATH", "/tmp/db"),
        ("DB_UPDATE_INTERVAL_SECONDS", "60"),
        ("HOST", "127.0.0.1"),
        ("PORT", "9000"),
        ("SWAGGER_UI_ENABLED", "true"),
        (
            "MAXMIND_DB_DOWNLOAD_URL",
            "https://mirror.example/{VARIANT}.tar.gz",
        ),
        ("MAXMIND_ACCOUNT_ID", "42"),
        ("MAXMIND_LICENSE_KEY", "key"),
    ])
    .unwrap();

    assert_eq!(config.db_variant, "GeoLite2-ASN");
    assert_eq!(config.db_path, "/tmp/db");
    assert_eq!(config.update_interval, 60);
    assert_eq!(config.host, "127.0.0.1");
    assert_eq!(config.port, 9000);
    assert!(config.swagger_ui_enabled);
    assert_eq!(
        config.download.url_for(&config.db_variant),
        "https://mirror.example/GeoLite2-ASN.tar.gz"
    );
    assert_eq!(config.download.account_id.as_deref(), Some("42"));
    assert_eq!(config.download.license_key.as_deref(), Some("key"));
}

#[test]
fn test_invalid_values() {
    for (key, value) in [
        ("PORT", "http"),
        ("PORT", "70000"),
        ("DB_UPDATE_INTERVAL_SECONDS", "-1"),
        ("SWAGGER_UI_ENABLED", "yes"),
    ] {
        let err = config_from(&[(key, value)]).err().unwrap();
        assert!(err.contains(key), "{err}");
    }
}

#[test]
fn test_rejects_insecure_download_url() {
    for url in [
        "http://download.example/{VARIANT}",
        "ftp://download.example/{VARIANT}",
        "not a url",
    ] {
        let err = config_from(&[("MAXMIND_DB_DOWNLOAD_URL", url)])
            .err()
            .unwrap();
        assert!(err.contains("MAXMIND_DB_DOWNLOAD_URL"), "{err}");
    }
}

#[test]
fn test_allows_http_for_loopback() {
    for url in [
        "http://localhost:8000/{VARIANT}",
        "http://127.0.0.1:8000/{VARIANT}",
        "http://[::1]:8000/{VARIANT}",
    ] {
        assert!(
            config_from(&[("MAXMIND_DB_DOWNLOAD_URL", url)]).is_ok(),
            "{url}"
        );
    }
}

#[test]
fn test_debug_redacts_license_key() {
    let download = DownloadConfig {
        license_key: Some("super-secret".to_string()),
        ..DownloadConfig::default()
    };

    assert!(!format!("{download:?}").contains("super-secret"));
}
