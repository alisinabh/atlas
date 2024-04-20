mod maxmind_db_updater;
mod network_utils;
mod schemas;
mod services;

use actix_web::{web, App, HttpServer};
use maxminddb::Reader;
use std::env;
use std::path::Path;
use std::sync::{Arc, RwLock};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db_variant = env::var("MAXMIND_DB_VARIANT").unwrap_or("GeoLite2-City".to_string());

    let init_db_path = Path::new("database-init.mmdb");
    let reader = Reader::open_readfile(init_db_path).expect("Unable to open IP database");

    let reader_data = web::Data::new(Arc::new(RwLock::new(reader)));

    let update_interval: u64 = env::var("DB_UPDATE_INTERVAL_SECONDS")
        .unwrap_or("86400".to_string())
        .parse()
        .expect("Invalid DB_UPDATE_INTERVAL_SECONDS value");

    let reader_data_clone = reader_data.clone();
    // Spawn the periodic update task
    tokio::spawn(async move {
        maxmind_db_updater::update_db_periodically(reader_data_clone, db_variant, update_interval)
            .await
    });

    let host = env::var("HOST").unwrap_or("0.0.0.0".to_string());
    let port: u16 = env::var("PORT")
        .unwrap_or("8080".to_string())
        .parse()
        .expect("Invalid PORT value");

    HttpServer::new(move || {
        let reader_data = reader_data.clone();
        App::new()
            .app_data(reader_data)
            .service(services::lookup::handle)
            .service(services::healthcheck::handle)
    })
    .bind((host, port))?
    .run()
    .await
}
