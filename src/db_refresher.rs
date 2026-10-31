use crate::BoxError;
use actix_web::web;
use std::future::Future;
use tokio::time::{Duration, sleep};

pub const FAILURE_RETRY_INTERVAL: Duration = Duration::from_secs(5 * 60);

pub trait UpdatableDB: Send + Sync {
    fn update_db(&self, db_min_age_secs: u64) -> impl Future<Output = Result<(), BoxError>> + Send;
}

pub async fn start_db_update_daemon(data: web::Data<impl UpdatableDB>, interval: u64) {
    let success_update_sleep = Duration::from_secs(interval);

    loop {
        println!("Checking for database updates...");

        let duration = match data.update_db(interval).await {
            Ok(_) => success_update_sleep,
            Err(error) => {
                println!("Failed to update database: {error}");
                FAILURE_RETRY_INTERVAL
            }
        };

        println!("Updater sleeping for {duration:?}");
        sleep(duration).await;
    }
}
