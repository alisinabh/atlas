#![allow(dead_code)]

use atlas_rs::config::DownloadConfig;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

pub const CITY_VARIANT: &str = "GeoIP2-City-Test";
pub const ACCOUNT_ID: &str = "account";
pub const LICENSE_KEY: &str = "secret";

pub fn test_data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests-data")
}

/// Copies `tests-data/<variant>_1` into a fresh temp dir so tests can mutate it.
pub fn temp_db_dir(variant: &str) -> TempDir {
    let dir = TempDir::new().unwrap();
    let src = test_data_dir().join(format!("{variant}_1"));
    let dst = dir.path().join(format!("{variant}_1"));
    std::fs::create_dir(&dst).unwrap();
    for entry in std::fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        std::fs::copy(entry.path(), dst.join(entry.file_name())).unwrap();
    }
    dir
}

/// Builds `<dir_name>.tar.gz` containing `<dir_name>/<variant>.mmdb` with the given contents.
pub fn make_archive(dir_name: &str, variant: &str, mmdb: &[u8]) -> Vec<u8> {
    let work = TempDir::new().unwrap();
    let db_dir = work.path().join(dir_name);
    std::fs::create_dir(&db_dir).unwrap();
    std::fs::write(db_dir.join(format!("{variant}.mmdb")), mmdb).unwrap();
    std::fs::write(db_dir.join("LICENSE.txt"), "license").unwrap();

    let archive = work.path().join("archive.tar.gz");
    let status = Command::new("tar")
        .env("COPYFILE_DISABLE", "1")
        .arg("czf")
        .arg(&archive)
        .arg("-C")
        .arg(work.path())
        .arg(dir_name)
        .status()
        .unwrap();
    assert!(status.success());

    std::fs::read(archive).unwrap()
}

pub fn city_mmdb() -> Vec<u8> {
    std::fs::read(
        test_data_dir()
            .join(format!("{CITY_VARIANT}_1"))
            .join(format!("{CITY_VARIANT}.mmdb")),
    )
    .unwrap()
}

pub fn download_config(server_uri: &str) -> DownloadConfig {
    DownloadConfig {
        url_template: format!("{server_uri}/geoip/databases/{{VARIANT}}/download"),
        account_id: Some(ACCOUNT_ID.to_string()),
        license_key: Some(LICENSE_KEY.to_string()),
    }
}

pub fn archive_response(dir_name: &str, body: Vec<u8>) -> wiremock::ResponseTemplate {
    wiremock::ResponseTemplate::new(200)
        .insert_header(
            "content-disposition",
            format!("attachment; filename={dir_name}.tar.gz").as_str(),
        )
        .set_body_bytes(body)
}

pub fn dir_names(path: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(path)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}
