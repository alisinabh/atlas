use core::fmt;
use futures_util::StreamExt;
use std::env;
use std::error::Error;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

#[derive(Debug)]
pub struct AlreadyDownloaded;

impl fmt::Display for AlreadyDownloaded {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for AlreadyDownloaded {}

pub async fn download_with_basic_auth(
    url: &str,
    output_path: &str,
    username: &str,
    password: Option<&str>,
) -> Result<String, Box<dyn Error>> {
    let response = reqwest::Client::new()
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
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Content-Disposition header missing or invalid",
            )
        })?
        .to_string();

    let full_path = PathBuf::from(output_path).join(&filename);

    let dir_path = PathBuf::from(output_path).join(filename.trim_end_matches(".tar.gz"));

    if tokio::fs::try_exists(&dir_path).await? {
        return Err(AlreadyDownloaded.into());
    }

    println!("Saving database in {}", full_path.to_str().unwrap());

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

pub async fn extract_db(path: &str, filename: &str) -> Result<String, Box<dyn Error>> {
    let full_path = PathBuf::from(path).join(filename);

    let mut command = Command::new("tar");

    command.arg("xvfz").arg(&full_path).arg("-C").arg(path);

    if env::consts::OS != "macos" {
        command.arg("--wildcards");
    }

    let output = command.arg("*.mmdb").output().await?;

    if !output.status.success() {
        println!("{output:?}");
        return Err("failed to extract archive".into());
    }

    tokio::fs::remove_file(full_path).await?;

    let result = match env::consts::OS {
        "macos" => output.stderr,
        _ => output.stdout,
    };

    let extracted_filename = String::from_utf8(result)?.replace('\n', "")[2..].to_string();

    Ok(extracted_filename)
}
