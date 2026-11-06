use crate::BoxError;
use core::fmt;
use futures_util::StreamExt;
use std::env;
use std::net::IpAddr;
use std::path::Path;
use std::time::Duration;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

pub const ARCHIVE_EXT: &str = ".tar.gz";

const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);
const READ_TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Debug)]
pub struct AlreadyDownloaded;

impl fmt::Display for AlreadyDownloaded {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for AlreadyDownloaded {}

/// Only allows https, or http to loopback hosts, so credentials are never sent in clear text
pub fn validate_download_url(url: &str) -> Result<reqwest::Url, BoxError> {
    let url = reqwest::Url::parse(url)?;

    let is_loopback = match url.host_str() {
        Some("localhost") => true,
        Some(host) => host
            .trim_start_matches('[')
            .trim_end_matches(']')
            .parse::<IpAddr>()
            .is_ok_and(|ip| ip.is_loopback()),
        None => false,
    };

    match url.scheme() {
        "https" => Ok(url),
        "http" if is_loopback => Ok(url),
        scheme => Err(format!("insecure or unsupported URL scheme {scheme:?}, use https").into()),
    }
}

/// Ensures the server supplied archive name can't escape the download directory
pub fn sanitize_archive_filename(filename: &str) -> Result<String, BoxError> {
    let is_plain_name = Path::new(filename)
        .file_name()
        .is_some_and(|name| name == filename);

    if !is_plain_name
        || filename.starts_with('.')
        || !filename.ends_with(ARCHIVE_EXT)
        || filename.len() == ARCHIVE_EXT.len()
    {
        return Err(format!("Invalid archive filename {filename:?}").into());
    }

    Ok(filename.to_string())
}

/// Downloads the archive into `download_dir` and returns its filename.
///
/// Fails with `AlreadyDownloaded` if `extracted_dir` already contains the extracted archive.
pub async fn download_with_basic_auth(
    url: &str,
    download_dir: &Path,
    extracted_dir: &Path,
    username: &str,
    password: Option<&str>,
) -> Result<String, BoxError> {
    let url = validate_download_url(url)?;

    let response = reqwest::Client::builder()
        .connect_timeout(CONNECT_TIMEOUT)
        .read_timeout(READ_TIMEOUT)
        .build()?
        .get(url)
        .basic_auth(username, password)
        .send()
        .await?;

    // Check if the request was successful
    if !response.status().is_success() {
        return Err(format!("Bad download response status code: {}", response.status()).into());
    }

    // Extract filename from Content-Disposition header
    let filename = response
        .headers()
        .get(reqwest::header::CONTENT_DISPOSITION)
        .and_then(|cd| {
            cd.to_str().ok()?.split(';').find_map(|s| {
                if s.trim().starts_with("filename=") {
                    Some(s.trim().trim_start_matches("filename=").trim_matches('"'))
                } else {
                    None
                }
            })
        })
        .ok_or("Content-Disposition header missing or invalid")?;

    let filename = sanitize_archive_filename(filename)?;

    if tokio::fs::try_exists(extracted_dir.join(filename.trim_end_matches(ARCHIVE_EXT))).await? {
        return Err(AlreadyDownloaded.into());
    }

    let full_path = download_dir.join(&filename);

    println!("Saving database in {}", full_path.display());

    // Stream the body of the response
    let mut file = File::create(full_path).await?;
    let mut stream = response.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item?;
        file.write_all(&chunk).await?;
    }

    file.flush().await?;

    Ok(filename)
}

/// Extracts the `.mmdb` files of `dir/filename` into `dir` and removes the archive
pub async fn extract_db(dir: &Path, filename: &str) -> Result<(), BoxError> {
    let full_path = dir.join(filename);

    let mut command = Command::new("tar");

    command.arg("xfz").arg(&full_path).arg("-C").arg(dir);

    if env::consts::OS != "macos" {
        command.arg("--wildcards");
    }

    let output = command.arg("*.mmdb").output().await?;

    if !output.status.success() {
        return Err(format!(
            "Failed to extract archive: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )
        .into());
    }

    tokio::fs::remove_file(full_path).await?;

    Ok(())
}
