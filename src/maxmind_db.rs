use core::fmt;
use futures_util::StreamExt;
use maxminddb::Reader;
use std::{env, error::Error, path::PathBuf};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::sync::RwLock;

const DEFAULT_DB_URL: &str =
    "https://download.maxmind.com/geoip/databases/{VARIANT}/download?suffix=tar.gz";

#[derive(Debug)]
pub struct MaxmindDB {
    pub reader: RwLock<Reader<Vec<u8>>>,
    pub variant: String,
    base_path: String,
}

#[derive(Debug)]
struct DBAlreadyExists;

impl fmt::Display for DBAlreadyExists {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for DBAlreadyExists {}

impl MaxmindDB {
    pub async fn init(variant: String, base_path: String) -> Result<Self, Box<dyn Error>> {
        let mut db_path = match Self::get_latest_variant(&variant, &base_path).await? {
            Some(db) => db,
            None => {
                println!("No database found! Fetching latest from upstream...");
                Self::fetch_latest_db(&variant, &base_path).await?
            }
        };

        db_path.push(format!("{}.mmdb", variant));

        println!("Initializing database from {}", db_path.to_str().unwrap());

        let reader = Reader::open_readfile(db_path)?;

        Ok(Self {
            reader: RwLock::new(reader),
            variant,
            base_path,
        })
    }

    pub async fn update_db(&self) -> Result<(), Box<dyn Error>> {
        let latest_db_path = match Self::fetch_latest_db(&self.variant, &self.base_path).await {
            Ok(path) => path,
            Err(error) => match error.downcast_ref::<DBAlreadyExists>() {
                Some(DBAlreadyExists) => return Ok(()),
                None => return Err(error),
            },
        };

        println!(
            "Updating Maxmind DB to {}",
            latest_db_path.to_str().unwrap()
        );

        {
            let new_reader = Reader::open_readfile(latest_db_path)?;
            let mut writer = self.reader.write().await;
            *writer = new_reader;
        }

        println!("Database updated successfully");

        Ok(())
    }

    async fn fetch_latest_db(variant: &str, output_path: &str) -> Result<PathBuf, Box<dyn Error>> {
        let db_download_url = env::var("MAXMIND_DB_DOWNLOAD_URL")
            .unwrap_or(DEFAULT_DB_URL.to_string())
            .replace("{VARIANT}", variant);

        let account_id = env::var("MAXMIND_ACCOUNT_ID")
            .map_err(|_| "MAXMIND_ACCOUNT_ID env var not set".to_string())?;
        let license_key = env::var("MAXMIND_LICENSE_KEY")
            .map_err(|_| "MAXMIND_LICENSE_KEY env var not set".to_string())?;

        let downloaded_filename = Self::download_with_basic_auth(
            &db_download_url,
            output_path,
            &account_id,
            Some(&license_key),
        )
        .await?;

        Self::extract_db(output_path, &downloaded_filename).await?;

        let db_file_name = downloaded_filename.replace(".tar.gz", ".mmdb");
        let db_full_path = PathBuf::from(output_path).join(db_file_name);

        Ok(db_full_path)
    }

    async fn get_latest_variant(
        variant: &str,
        db_path: &str,
    ) -> Result<Option<PathBuf>, Box<dyn Error>> {
        let mut entries = tokio::fs::read_dir(db_path).await?;
        let mut db_versions: Vec<PathBuf> = Vec::new();

        while let Some(entry) = entries.next_entry().await? {
            if entry.file_type().await?.is_dir()
                && entry
                    .file_name()
                    .to_str()
                    .ok_or("Invalid directory entry")?
                    .starts_with(variant)
            {
                db_versions.push(entry.path())
            }
        }

        Ok(db_versions.iter().max().cloned())
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

        if tokio::fs::try_exists(&full_path).await? {
            return Err(DBAlreadyExists.into());
        }

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

        let extracted_filename =
            String::from_utf8(output.stderr)?.replace('\n', "")[2..].to_string();

        Ok(extracted_filename)
    }
}
