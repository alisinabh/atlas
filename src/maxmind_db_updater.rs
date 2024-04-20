use actix_web::web;
use futures_util::StreamExt;
use maxminddb::Reader;
use std::env;
use std::error::Error;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::time::{sleep, Duration};

pub async fn update_db_periodically(
    data: web::Data<Arc<RwLock<Reader<Vec<u8>>>>>,
    db_variant: String,
    interval: u64,
) -> ! {
    loop {
        let duration = match update_db(&data, &db_variant).await {
            Ok(_) => Duration::from_secs(interval),
            Err(err) => {
                println!("Failed to update database {:?}", err);
                Duration::from_secs(5 * 60)
            }
        };

        sleep(duration).await;
    }
}

const DEFAULT_DB_URL: &str =
    "https://download.maxmind.com/geoip/databases/{VARIANT}/download?suffix=tar.gz";

async fn update_db(
    data: &web::Data<Arc<RwLock<Reader<Vec<u8>>>>>,
    db_variant: &str,
) -> Result<(), Box<dyn Error>> {
    println!("Updating GeoIP Database...");

    let db_path = env::var("DB_PATH").unwrap_or("".to_string());

    let db_download_url = env::var("MAXMIND_DB_DOWNLOAD_URL")
        .unwrap_or(DEFAULT_DB_URL.to_string())
        .replace("{VARIANT}", db_variant);

    let account_id = env::var("MAXMIND_ACCOUNT_ID")
        .map_err(|_| "MAXMIND_ACCOUNT_ID env var not set".to_string())?;
    let license_key = env::var("MAXMIND_LICENSE_KEY")
        .map_err(|_| "MAXMIND_LICENSE_KEY env var not set".to_string())?;

    let downloaded_filename =
        download_with_basic_auth(&db_download_url, &db_path, &account_id, Some(&license_key))
            .await?;

    extract_db(&db_path, &downloaded_filename).await?;

    let db_file_name = downloaded_filename.replace(".tar.gz", ".mmdb");
    let db_full_path = PathBuf::from(db_path).join(db_file_name);

    println!("{:?}", db_full_path);

    {
        let new_reader = Reader::open_readfile(db_full_path)?;
        let mut writer = data.write().unwrap(); // acquiring write lock
        *writer = new_reader;
    }

    println!("Database updated successfully");

    Ok(())
}

async fn extract_db(path: &str, filename: &str) -> Result<String, Box<dyn Error>> {
    let full_path = PathBuf::from(path).join(filename);

    let output = Command::new("tar")
        .arg("xvfz")
        .arg(&full_path)
        .arg("*.mmdb")
        .output()
        .await?;

    if !output.status.success() {
        return Err("failed to extract archive".into());
    }

    tokio::fs::remove_file(full_path).await?;

    let extracted_filename = String::from_utf8(output.stderr)?.replace('\n', "")[2..].to_string();

    Ok(extracted_filename)
}

async fn download_with_basic_auth(
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
                    Some(s.trim_start_matches("filename=").trim_matches('"'))
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
