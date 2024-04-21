use crate::download_utils::*;
use maxminddb::{MaxMindDBError, Reader};
use std::{env, error::Error, path::PathBuf};
use tokio::sync::RwLock;

const DEFAULT_DB_URL: &str =
    "https://download.maxmind.com/geoip/databases/{VARIANT}/download?suffix=tar.gz";

#[derive(Debug)]
pub struct MaxmindDB {
    pub reader: RwLock<Reader<Vec<u8>>>,
    pub variant: String,
    base_path: String,
}

impl MaxmindDB {
    pub async fn init(variant: String, base_path: String) -> Result<Self, Box<dyn Error>> {
        let db_path = match Self::get_latest_variant(&variant, &base_path).await? {
            Some(db) => db,
            None => {
                println!("No database found! Fetching latest from upstream...");
                Self::fetch_latest_db(&variant, &base_path).await?
            }
        };

        let reader = Self::load_db(db_path, &variant)?;

        Ok(Self {
            reader: RwLock::new(reader),
            variant,
            base_path,
        })
    }

    fn load_db(mut path: PathBuf, variant: &str) -> Result<Reader<Vec<u8>>, MaxMindDBError> {
        path.push(format!("{}.mmdb", variant));

        println!("Loading database from {}", path.to_str().unwrap());

        Reader::open_readfile(path)
    }

    pub async fn update_db(&self) -> Result<(), Box<dyn Error>> {
        let latest_db_path = match Self::fetch_latest_db(&self.variant, &self.base_path).await {
            Ok(path) => path,
            Err(error) => match error.downcast_ref::<AlreadyDownloaded>() {
                Some(AlreadyDownloaded) => return Ok(()),
                None => return Err(error),
            },
        };

        let new_reader = Self::load_db(latest_db_path, &self.variant)?;
        let mut writer = self.reader.write().await;
        *writer = new_reader;

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

        let downloaded_filename = download_with_basic_auth(
            &db_download_url,
            output_path,
            &account_id,
            Some(&license_key),
        )
        .await?;

        extract_db(output_path, &downloaded_filename).await?;

        let db_dir_name = downloaded_filename.trim_end_matches(".tar.gz");
        let db_full_path = PathBuf::from(output_path).join(db_dir_name);

        Ok(db_full_path)
    }

    pub async fn get_latest_variant(
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
}
