use actix_web::web;
use tokio::time::{sleep, Duration};

use crate::maxmind_db::MaxmindDB;

pub fn start_db_update_daemon(data: web::Data<MaxmindDB>, interval: u64) {
    let success_update_sleep = Duration::from_secs(interval);
    let failure_update_sleep = Duration::from_secs(5 * 60);

    tokio::spawn(async move {
        loop {
            println!("Checking for database updates...");

            let duration = match data.update_db().await {
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
