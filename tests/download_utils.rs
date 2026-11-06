mod common;

use atlas_rs::download_utils::{
    AlreadyDownloaded, download_with_basic_auth, extract_db, sanitize_archive_filename,
    validate_download_url,
};
use common::{ACCOUNT_ID, CITY_VARIANT, LICENSE_KEY};
use tempfile::TempDir;
use wiremock::matchers::{basic_auth, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const DIR_NAME: &str = "GeoIP2-City-Test_2";

async fn download(server: &MockServer, dir: &TempDir) -> Result<String, atlas_rs::BoxError> {
    download_with_basic_auth(
        &format!("{}/db", server.uri()),
        dir.path(),
        dir.path(),
        ACCOUNT_ID,
        Some(LICENSE_KEY),
    )
    .await
}

async fn server_with(response: ResponseTemplate) -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/db"))
        .respond_with(response)
        .mount(&server)
        .await;
    server
}

#[test]
fn test_sanitize_archive_filename() {
    assert_eq!(
        sanitize_archive_filename("GeoLite2-City_20240101.tar.gz").unwrap(),
        "GeoLite2-City_20240101.tar.gz"
    );

    for name in [
        "../GeoLite2-City.tar.gz",
        "../../etc/passwd.tar.gz",
        "/etc/GeoLite2-City.tar.gz",
        "sub/GeoLite2-City.tar.gz",
        "..",
        ".tar.gz",
        ".hidden.tar.gz",
        "GeoLite2-City.zip",
        "",
    ] {
        assert!(sanitize_archive_filename(name).is_err(), "{name:?}");
    }
}

#[test]
fn test_validate_download_url() {
    assert!(validate_download_url("https://download.maxmind.com/x").is_ok());
    assert!(validate_download_url("http://localhost:1234/x").is_ok());
    assert!(validate_download_url("http://127.0.0.1:1234/x").is_ok());
    assert!(validate_download_url("http://[::1]:1234/x").is_ok());

    assert!(validate_download_url("http://download.maxmind.com/x").is_err());
    assert!(validate_download_url("http://localhost.evil.com/x").is_err());
    assert!(validate_download_url("ftp://download.maxmind.com/x").is_err());
    assert!(validate_download_url("nonsense").is_err());
}

#[tokio::test]
async fn test_download_with_basic_auth() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/db"))
        .and(basic_auth(ACCOUNT_ID, LICENSE_KEY))
        .respond_with(common::archive_response(DIR_NAME, b"archive".to_vec()))
        .expect(1)
        .mount(&server)
        .await;
    let dir = TempDir::new().unwrap();

    let filename = download(&server, &dir).await.unwrap();

    assert_eq!(filename, format!("{DIR_NAME}.tar.gz"));
    assert_eq!(
        std::fs::read(dir.path().join(&filename)).unwrap(),
        b"archive"
    );
}

#[tokio::test]
async fn test_download_quoted_filename() {
    let server = server_with(
        ResponseTemplate::new(200)
            .insert_header(
                "content-disposition",
                format!("attachment; filename=\"{DIR_NAME}.tar.gz\"").as_str(),
            )
            .set_body_bytes(b"archive".to_vec()),
    )
    .await;
    let dir = TempDir::new().unwrap();

    assert_eq!(
        download(&server, &dir).await.unwrap(),
        format!("{DIR_NAME}.tar.gz")
    );
}

#[tokio::test]
async fn test_download_bad_status() {
    let server = server_with(ResponseTemplate::new(401)).await;
    let dir = TempDir::new().unwrap();

    let err = download(&server, &dir).await.unwrap_err();

    assert!(err.to_string().contains("401"), "{err}");
    assert!(common::dir_names(dir.path()).is_empty());
}

#[tokio::test]
async fn test_download_missing_content_disposition() {
    let server = server_with(ResponseTemplate::new(200).set_body_bytes(b"x".to_vec())).await;
    let dir = TempDir::new().unwrap();

    assert!(download(&server, &dir).await.is_err());
    assert!(common::dir_names(dir.path()).is_empty());
}

#[tokio::test]
async fn test_download_rejects_path_traversal() {
    let server = server_with(
        ResponseTemplate::new(200)
            .insert_header(
                "content-disposition",
                "attachment; filename=../escape.tar.gz",
            )
            .set_body_bytes(b"x".to_vec()),
    )
    .await;
    let root = TempDir::new().unwrap();
    let dir = root.path().join("db");
    std::fs::create_dir(&dir).unwrap();

    let result =
        download_with_basic_auth(&format!("{}/db", server.uri()), &dir, &dir, "a", Some("b")).await;

    assert!(result.is_err());
    assert_eq!(common::dir_names(root.path()), vec!["db"]);
    assert!(common::dir_names(&dir).is_empty());
}

#[tokio::test]
async fn test_download_rejects_insecure_url() {
    let dir = TempDir::new().unwrap();
    let result = download_with_basic_auth(
        "http://download.example.com/db",
        dir.path(),
        dir.path(),
        "a",
        Some("b"),
    )
    .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_download_already_downloaded() {
    let server = server_with(common::archive_response(DIR_NAME, b"archive".to_vec())).await;
    let dir = TempDir::new().unwrap();
    std::fs::create_dir(dir.path().join(DIR_NAME)).unwrap();

    let err = download(&server, &dir).await.unwrap_err();

    assert!(err.downcast_ref::<AlreadyDownloaded>().is_some(), "{err}");
    assert_eq!(common::dir_names(dir.path()), vec![DIR_NAME]);
}

#[tokio::test]
async fn test_extract_db() {
    let dir = TempDir::new().unwrap();
    let archive = format!("{DIR_NAME}.tar.gz");
    std::fs::write(
        dir.path().join(&archive),
        common::make_archive(DIR_NAME, CITY_VARIANT, b"mmdb-bytes"),
    )
    .unwrap();

    extract_db(dir.path(), &archive).await.unwrap();

    // Only the .mmdb file is extracted and the archive is removed
    assert_eq!(common::dir_names(dir.path()), vec![DIR_NAME]);
    assert_eq!(
        common::dir_names(&dir.path().join(DIR_NAME)),
        vec![format!("{CITY_VARIANT}.mmdb")]
    );
}

#[tokio::test]
async fn test_extract_db_corrupt_archive() {
    let dir = TempDir::new().unwrap();
    let archive = format!("{DIR_NAME}.tar.gz");
    std::fs::write(dir.path().join(&archive), b"not a tarball").unwrap();

    assert!(extract_db(dir.path(), &archive).await.is_err());
}
