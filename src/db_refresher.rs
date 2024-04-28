use actix_web::web;
use std::error::Error;
use std::future::Future;
use tokio::time::{sleep, Duration};

pub trait UpdatableDB: Send + Sync {
    fn update_db(
        &self,
        db_min_age_secs: u64,
    ) -> impl Future<Output = Result<(), Box<dyn Error>>> + Send;
}

pub fn start_db_update_daemon(data: web::Data<impl UpdatableDB + 'static>, interval: u64) {
    let success_update_sleep = Duration::from_secs(interval);
    let failure_update_sleep = Duration::from_secs(5 * 60);

    tokio::spawn(async move {
        loop {
            println!("Checking for database updates...");

            let duration = match data.update_db(interval).await {
                Ok(_) => success_update_sleep,
                Err(error) => {
                    println!("Failed to update database {:?}", error);
                    failure_update_sleep
                }
            };

            println!("Updater sleeping for {:?}", duration);
            sleep(duration).await;
        }
    });
}
