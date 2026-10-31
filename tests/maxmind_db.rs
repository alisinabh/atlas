mod common;

use atlas_rs::config::DownloadConfig;
use atlas_rs::db_refresher::UpdatableDB;
use atlas_rs::maxmind_db::MaxmindDB;
use common::{CITY_VARIANT, city_mmdb, dir_names, download_config, make_archive, temp_db_dir};
use std::net::IpAddr;
use tempfile::TempDir;
use wiremock::matchers::method;
use wiremock::{Mock, MockServer, ResponseTemplate};

const NEW_DIR: &str = "GeoIP2-City-Test_2";

async fn server_with(response: ResponseTemplate, expected_calls: u64) -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(response)
        .expect(expected_calls)
        .mount(&server)
        .await;
    server
}

fn path_str(dir: &TempDir) -> &str {
    dir.path().to_str().unwrap()
}

fn san_diego_lookup(db: &MaxmindDB) -> Option<String> {
    let ip: IpAddr = "214.78.120.1".parse().unwrap();
    let current = db.current();
    let result = current.lookup::<maxminddb::geoip2::City>(&[ip]);
    result[&ip]
        .as_ref()
        .and_then(|city| city.city.names.english)
        .map(str::to_string)
}

#[tokio::test]
async fn test_get_latest_variant() {
    let dir = TempDir::new().unwrap();
    for name in [
        "GeoIP2-City-Test_20240101",
        "GeoIP2-City-Test_20240301",
        "GeoIP2-City-Test-Extra_20991231",
        "GeoIP2-Country-Test_20991231",
        ".staging",
    ] {
        std::fs::create_dir(dir.path().join(name)).unwrap();
    }
    std::fs::write(dir.path().join("GeoIP2-City-Test_20991231"), "a file").unwrap();

    let latest = MaxmindDB::get_latest_variant(CITY_VARIANT, dir.path())
        .await
        .unwrap()
        .unwrap();

    assert_eq!(latest, dir.path().join("GeoIP2-City-Test_20240301"));
}

#[tokio::test]
async fn test_get_latest_variant_none() {
    let dir = TempDir::new().unwrap();

    assert!(
        MaxmindDB::get_latest_variant(CITY_VARIANT, dir.path())
            .await
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
async fn test_init_loads_existing_db_without_download() {
    let dir = temp_db_dir(CITY_VARIANT);
    let server = server_with(ResponseTemplate::new(500), 0).await;

    let db = MaxmindDB::init(CITY_VARIANT, path_str(&dir), download_config(&server.uri()))
        .await
        .unwrap();

    assert_eq!(san_diego_lookup(&db).as_deref(), Some("San Diego"));
}

#[tokio::test]
async fn test_init_downloads_into_missing_dir() {
    let root = TempDir::new().unwrap();
    let db_path = root.path().join("nested").join("db");
    let server = server_with(
        common::archive_response(NEW_DIR, make_archive(NEW_DIR, CITY_VARIANT, &city_mmdb())),
        1,
    )
    .await;

    let db = MaxmindDB::init(
        CITY_VARIANT,
        db_path.to_str().unwrap(),
        download_config(&server.uri()),
    )
    .await
    .unwrap();

    assert_eq!(db.current().base_path, db_path.join(NEW_DIR));
    assert_eq!(dir_names(&db_path), vec![NEW_DIR]);
    assert_eq!(san_diego_lookup(&db).as_deref(), Some("San Diego"));
}

#[tokio::test]
async fn test_init_without_credentials_fails() {
    let dir = TempDir::new().unwrap();

    let err = MaxmindDB::init(CITY_VARIANT, path_str(&dir), DownloadConfig::default())
        .await
        .unwrap_err();

    assert!(err.to_string().contains("MAXMIND_ACCOUNT_ID"), "{err}");
}

#[tokio::test]
async fn test_init_replaces_broken_db() {
    let dir = TempDir::new().unwrap();
    let broken = dir.path().join("GeoIP2-City-Test_1");
    std::fs::create_dir(&broken).unwrap();
    std::fs::write(broken.join(format!("{CITY_VARIANT}.mmdb")), b"garbage").unwrap();
    let server = server_with(
        common::archive_response(NEW_DIR, make_archive(NEW_DIR, CITY_VARIANT, &city_mmdb())),
        1,
    )
    .await;

    let db = MaxmindDB::init(CITY_VARIANT, path_str(&dir), download_config(&server.uri()))
        .await
        .unwrap();

    assert_eq!(dir_names(dir.path()), vec![NEW_DIR]);
    assert_eq!(san_diego_lookup(&db).as_deref(), Some("San Diego"));
}

#[tokio::test]
async fn test_update_skips_when_db_is_too_new() {
    let dir = temp_db_dir(CITY_VARIANT);
    let server = server_with(ResponseTemplate::new(500), 0).await;
    let db = MaxmindDB::init(CITY_VARIANT, path_str(&dir), download_config(&server.uri()))
        .await
        .unwrap();

    db.update_db(u64::MAX).await.unwrap();

    assert_eq!(dir_names(dir.path()), vec!["GeoIP2-City-Test_1"]);
}

#[tokio::test]
async fn test_update_swaps_db_and_removes_stale() {
    let dir = temp_db_dir(CITY_VARIANT);
    let server = server_with(
        common::archive_response(NEW_DIR, make_archive(NEW_DIR, CITY_VARIANT, &city_mmdb())),
        1,
    )
    .await;
    let db = MaxmindDB::init(CITY_VARIANT, path_str(&dir), download_config(&server.uri()))
        .await
        .unwrap();
    let old = db.current();

    db.update_db(0).await.unwrap();

    assert_eq!(db.current().base_path, dir.path().join(NEW_DIR));
    assert_eq!(dir_names(dir.path()), vec![NEW_DIR]);
    assert_eq!(san_diego_lookup(&db).as_deref(), Some("San Diego"));
    // In-flight readers holding the old database keep working
    assert!(old.build_epoch() > 0);
}

#[tokio::test]
async fn test_update_already_downloaded_is_noop() {
    let dir = temp_db_dir(CITY_VARIANT);
    let server = server_with(
        common::archive_response("GeoIP2-City-Test_1", b"unused".to_vec()),
        1,
    )
    .await;
    let db = MaxmindDB::init(CITY_VARIANT, path_str(&dir), download_config(&server.uri()))
        .await
        .unwrap();

    db.update_db(0).await.unwrap();

    assert_eq!(
        db.current().base_path,
        dir.path().join("GeoIP2-City-Test_1")
    );
    assert_eq!(dir_names(dir.path()), vec!["GeoIP2-City-Test_1"]);
}

#[tokio::test]
async fn test_update_with_broken_db_keeps_current_and_retries() {
    let dir = temp_db_dir(CITY_VARIANT);
    let server = server_with(
        common::archive_response(NEW_DIR, make_archive(NEW_DIR, CITY_VARIANT, b"garbage")),
        2,
    )
    .await;
    let db = MaxmindDB::init(CITY_VARIANT, path_str(&dir), download_config(&server.uri()))
        .await
        .unwrap();

    assert!(db.update_db(0).await.is_err());
    // Not marked as downloaded, so the next update tries again
    assert!(db.update_db(0).await.is_err());

    assert_eq!(
        db.current().base_path,
        dir.path().join("GeoIP2-City-Test_1")
    );
    assert_eq!(dir_names(dir.path()), vec!["GeoIP2-City-Test_1"]);
    assert_eq!(san_diego_lookup(&db).as_deref(), Some("San Diego"));
}

#[tokio::test]
async fn test_update_with_server_error() {
    let dir = temp_db_dir(CITY_VARIANT);
    let server = server_with(ResponseTemplate::new(500), 1).await;
    let db = MaxmindDB::init(CITY_VARIANT, path_str(&dir), download_config(&server.uri()))
        .await
        .unwrap();

    assert!(db.update_db(0).await.is_err());
    assert_eq!(dir_names(dir.path()), vec!["GeoIP2-City-Test_1"]);
}
