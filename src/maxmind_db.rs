use crate::{
    download_utils::*,
    schemas::{LookupResult, Lookupable},
};
use maxminddb::{MaxMindDBError, Reader};
use serde::Serialize;
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
    pub path: String,
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

        let inner_db = MaxmindDBInner::load(&db_path, &variant)?;

        Ok(Self {
            db: RwLock::new(inner_db),
            variant,
            base_path,
        })
    }

    pub async fn lookup<T: Lookupable + Serialize>(
        &self,
        ip_addresses: Vec<IpAddr>,
    ) -> (HashMap<IpAddr, Option<LookupResult>>, u64) {
        let db_read = self.db.read().await;

        let results = ip_addresses
            .iter()
            .map(|&ip| (ip, T::lookup(&db_read.reader, ip).ok()))
            .collect();

        let database_build_epoch = db_read.reader.metadata.build_epoch;

        (results, database_build_epoch)
    }

    pub async fn update_db(&self, db_min_age_secs: u64) -> Result<(), Box<dyn Error>> {
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

        let new_db = MaxmindDBInner::load(&latest_db_path, &self.variant)?;
        self.db.update_inner_db(new_db).await;

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

impl MaxmindDBInner {
    fn load<P: AsRef<Path>, S: AsRef<str>>(path: P, variant: S) -> Result<Self, MaxMindDBError> {
        let mut path = path.as_ref().to_path_buf();

        path.push(variant.as_ref());
        path.set_extension(MAXMIND_EXT);

        let path = path.to_str().unwrap().to_string();

        println!("Loading database from {}", path);
        let reader = Reader::open_readfile(&path)?;

        Ok(Self { reader, path })
    }
}

trait MaxmindDBRwLockTrait {
    async fn build_epoch(&self) -> u64;
    async fn update_inner_db(&self, new_db: MaxmindDBInner);
}

impl MaxmindDBRwLockTrait for RwLock<MaxmindDBInner> {
    async fn build_epoch(&self) -> u64 {
        let db = self.read().await;
        db.reader.metadata.build_epoch
    }

    async fn update_inner_db(&self, new_db: MaxmindDBInner) {
        let mut writer = self.write().await;
        *writer = new_db;
    }
}

fn current_time_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
