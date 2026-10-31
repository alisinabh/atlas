use crate::{
    BoxError,
    config::DownloadConfig,
    db_refresher::UpdatableDB,
    download_utils::{ARCHIVE_EXT, AlreadyDownloaded, download_with_basic_auth, extract_db},
};
use arc_swap::ArcSwap;
use maxminddb::{MaxMindDbError, Reader};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::IpAddr,
    path::{Path, PathBuf},
    sync::Arc,
};

const MAXMIND_EXT: &str = "mmdb";
/// Downloads are extracted and verified here before being moved next to the other databases
const STAGING_DIR: &str = ".staging";

#[derive(Debug)]
pub struct MaxmindDB {
    db: ArcSwap<MaxmindDBInner>,
    pub variant: String,
    base_path: PathBuf,
    download: DownloadConfig,
}

#[derive(Debug)]
pub struct MaxmindDBInner {
    pub reader: Reader<Vec<u8>>,
    pub filename: String,
    pub base_path: PathBuf,
}

impl MaxmindDB {
    pub async fn init(
        variant: &str,
        base_path: &str,
        download: DownloadConfig,
    ) -> Result<Self, BoxError> {
        let base_path = PathBuf::from(base_path);
        tokio::fs::create_dir_all(&base_path).await?;

        let existing_db = match Self::get_latest_variant(variant, &base_path).await? {
            Some(db_path) => match MaxmindDBInner::load_async(db_path.clone(), variant).await {
                Ok(db) => Some(db),
                Err(reason) => {
                    println!(
                        "Failed to load database at {}: {reason}. Removing it",
                        db_path.display()
                    );
                    tokio::fs::remove_dir_all(&db_path).await?;
                    None
                }
            },
            None => None,
        };

        let inner_db = match existing_db {
            Some(db) => db,
            None => {
                println!("No database found! Fetching latest from upstream...");
                Self::fetch_latest_db(variant, &base_path, &download).await?
            }
        };

        Ok(Self {
            db: ArcSwap::from_pointee(inner_db),
            variant: variant.to_string(),
            base_path,
            download,
        })
    }

    /// Returns the currently active database. Lookups keep working on it even if it gets swapped.
    pub fn current(&self) -> Arc<MaxmindDBInner> {
        self.db.load_full()
    }

    async fn fetch_latest_db(
        variant: &str,
        base_path: &Path,
        download: &DownloadConfig,
    ) -> Result<MaxmindDBInner, BoxError> {
        let staging_path = base_path.join(STAGING_DIR);

        if tokio::fs::try_exists(&staging_path).await? {
            tokio::fs::remove_dir_all(&staging_path).await?;
        }
        tokio::fs::create_dir(&staging_path).await?;

        let result = Self::fetch_into_staging(variant, base_path, &staging_path, download).await;

        if let Err(reason) = tokio::fs::remove_dir_all(&staging_path).await {
            println!("Failed to clean up {}: {reason}", staging_path.display());
        }

        result
    }

    async fn fetch_into_staging(
        variant: &str,
        base_path: &Path,
        staging_path: &Path,
        download: &DownloadConfig,
    ) -> Result<MaxmindDBInner, BoxError> {
        let account_id = download
            .account_id
            .as_deref()
            .ok_or("MAXMIND_ACCOUNT_ID env var not set")?;
        let license_key = download
            .license_key
            .as_deref()
            .ok_or("MAXMIND_LICENSE_KEY env var not set")?;

        let archive_filename = download_with_basic_auth(
            &download.url_for(variant),
            staging_path,
            base_path,
            account_id,
            Some(license_key),
        )
        .await?;

        extract_db(staging_path, &archive_filename).await?;

        let db_dir_name = archive_filename.trim_end_matches(ARCHIVE_EXT);
        let staged_db_path = staging_path.join(db_dir_name);

        // Make sure the database is usable before it replaces anything
        let mut db = MaxmindDBInner::load_async(staged_db_path.clone(), variant).await?;

        let db_full_path = base_path.join(db_dir_name);
        tokio::fs::rename(&staged_db_path, &db_full_path).await?;
        db.base_path = db_full_path;

        Ok(db)
    }

    pub async fn get_latest_variant(
        variant: &str,
        db_path: &Path,
    ) -> Result<Option<PathBuf>, BoxError> {
        let prefix = format!("{variant}_");
        let mut entries = tokio::fs::read_dir(db_path).await?;
        let mut db_versions: Vec<PathBuf> = Vec::new();

        while let Some(entry) = entries.next_entry().await? {
            if entry.file_type().await?.is_dir()
                && entry
                    .file_name()
                    .to_str()
                    .is_some_and(|name| name.starts_with(&prefix))
            {
                db_versions.push(entry.path())
            }
        }

        Ok(db_versions.into_iter().max())
    }
}

impl UpdatableDB for MaxmindDB {
    async fn update_db(&self, db_min_age_secs: u64) -> Result<(), BoxError> {
        let stale_db = self.current();

        if stale_db.build_epoch().saturating_add(db_min_age_secs) > current_time_unix() {
            println!("Database is too new to update");
            return Ok(());
        }

        let latest_db =
            match Self::fetch_latest_db(&self.variant, &self.base_path, &self.download).await {
                Ok(db) => db,
                Err(error) => match error.downcast_ref::<AlreadyDownloaded>() {
                    Some(AlreadyDownloaded) => return Ok(()),
                    None => return Err(error),
                },
            };

        println!(
            "Database updated successfully. {}",
            latest_db.base_path.display()
        );

        self.db.store(Arc::new(latest_db));

        println!(
            "Removing stale database at {}",
            stale_db.base_path.display()
        );

        if let Err(reason) = tokio::fs::remove_dir_all(&stale_db.base_path).await {
            println!("Failed to remove stale database {reason:?}");
        };

        Ok(())
    }
}

impl MaxmindDBInner {
    fn load(base_path: &Path, variant: &str) -> Result<Self, MaxMindDbError> {
        let mut path = base_path.join(variant);
        path.set_extension(MAXMIND_EXT);

        println!("Loading database from {}", path.display());
        let reader = Reader::open_readfile(&path)?;

        Ok(Self {
            reader,
            filename: path
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_default(),
            base_path: base_path.to_path_buf(),
        })
    }

    /// Reads the database file on a blocking thread
    async fn load_async(base_path: PathBuf, variant: &str) -> Result<Self, BoxError> {
        let variant = variant.to_string();

        Ok(tokio::task::spawn_blocking(move || Self::load(&base_path, &variant)).await??)
    }

    /// Records without any data for the requested type (e.g. an `isp` lookup on a City database)
    /// are returned as `None`
    pub fn lookup<'de, T>(&'de self, ip_addresses: &[IpAddr]) -> HashMap<IpAddr, Option<T>>
    where
        T: Deserialize<'de> + Serialize,
    {
        ip_addresses
            .iter()
            .map(|&ip| {
                let result = self
                    .reader
                    .lookup(ip)
                    .and_then(|lookup_result| lookup_result.decode())
                    .ok()
                    .flatten()
                    .filter(|record| !is_empty_record(record));

                (ip, result)
            })
            .collect()
    }

    pub fn build_epoch(&self) -> u64 {
        self.reader.metadata().build_epoch
    }
}

fn is_empty_record<T: Serialize>(record: &T) -> bool {
    serde_json::to_value(record)
        .is_ok_and(|value| value.as_object().is_some_and(serde_json::Map::is_empty))
}

fn current_time_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}
