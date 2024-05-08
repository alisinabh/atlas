use crate::{db_refresher::UpdatableDB, download_utils::*};
use maxminddb::{MaxMindDBError, Reader};
use serde::Deserialize;
use std::{
    collections::HashMap,
    env,
    error::Error,
    net::IpAddr,
    path::{Path, PathBuf},
};
use tokio::sync::RwLock;

const MAXMIND_EXT: &str = "mmdb";
const DEFAULT_DB_URL: &str =
    "https://download.maxmind.com/geoip/databases/{VARIANT}/download?suffix=tar.gz";

#[derive(Debug)]
pub struct MaxmindDB {
    pub db: RwLock<MaxmindDBInner>,
    pub variant: String,
    base_path: String,
}

#[derive(Debug)]
pub struct MaxmindDBInner {
    pub reader: Reader<Vec<u8>>,
    pub filename: String,
    pub base_path: String,
}

impl<'de> MaxmindDB {
    pub async fn init(variant: &str, base_path: &str) -> Result<Self, Box<dyn Error>> {
        let db_path = match Self::get_latest_variant(variant, base_path).await? {
            Some(db) => db,
            None => {
                println!("No database found! Fetching latest from upstream...");
                Self::fetch_latest_db(variant, base_path).await?
            }
        };

        let inner_db = MaxmindDBInner::load(db_path, variant)?;

        Ok(Self {
            db: RwLock::new(inner_db),
            variant: variant.to_string(),
            base_path: base_path.to_string(),
        })
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

impl UpdatableDB for MaxmindDB {
    async fn update_db(&self, db_min_age_secs: u64) -> Result<(), Box<dyn Error>> {
        if self.db.build_epoch().await + db_min_age_secs > current_time_unix() {
            println!("Database is too new to update");
            return Ok(());
        }

        let latest_db_path = match Self::fetch_latest_db(&self.variant, &self.base_path).await {
            Ok(path) => path,
            Err(error) => match error.downcast_ref::<AlreadyDownloaded>() {
                Some(AlreadyDownloaded) => return Ok(()),
                None => return Err(error),
            },
        };

        let stale_db_path = self.db.db_base_path().await;

        let new_db = MaxmindDBInner::load(&latest_db_path, &self.variant)?;
        self.db.update_inner_db(new_db).await;

        println!(
            "Database updated successfully. {}",
            latest_db_path.to_str().unwrap()
        );

        print!("Removing stale database at {stale_db_path}");

        if let Err(reason) = tokio::fs::remove_dir_all(stale_db_path).await {
            print!("Failed to remove stale database {reason:?}");
        };

        Ok(())
    }
}

impl<'de> MaxmindDBInner {
    fn load<P: AsRef<Path>, S: AsRef<str>>(
        base_path: P,
        variant: S,
    ) -> Result<Self, MaxMindDBError> {
        let mut path = base_path.as_ref().to_path_buf();

        path.push(variant.as_ref());
        path.set_extension(MAXMIND_EXT);

        let filename = path.file_name().unwrap().to_str().unwrap().to_string();
        let full_path = path.to_str().unwrap().to_string();

        println!("Loading database from {}", full_path);
        let reader = Reader::open_readfile(&full_path)?;

        Ok(Self {
            reader,
            filename,
            base_path: base_path.as_ref().to_str().unwrap().to_string(),
        })
    }

    pub async fn lookup<T>(&'de self, ip_addresses: Vec<IpAddr>) -> HashMap<IpAddr, Option<T>>
    where
        T: Deserialize<'de>,
    {
        ip_addresses
            .iter()
            .map(|&ip| (ip, self.reader.lookup::<T>(ip).ok()))
            .collect()
    }

    pub fn build_epoch(&self) -> u64 {
        self.reader.metadata.build_epoch
    }
}

trait MaxmindDBRwLockTrait {
    async fn build_epoch(&self) -> u64;
    async fn db_base_path(&self) -> String;
    async fn update_inner_db(&self, new_db: MaxmindDBInner);
}

impl MaxmindDBRwLockTrait for RwLock<MaxmindDBInner> {
    async fn build_epoch(&self) -> u64 {
        let db = self.read().await;
        db.build_epoch()
    }

    async fn update_inner_db(&self, new_db: MaxmindDBInner) {
        let mut writer = self.write().await;
        *writer = new_db;
    }

    async fn db_base_path(&self) -> String {
        self.read().await.base_path.clone()
    }
}

fn current_time_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
