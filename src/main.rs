mod download_utils;
mod maxmind_db;
mod maxmind_db_updater;
mod network_utils;
mod schemas;
mod services;

use actix_web::{web, App, HttpServer};
use maxmind_db::MaxmindDB;
use std::env;
use std::io::Result;

#[actix_web::main]
async fn main() -> Result<()> {
    // Read environment variables
    let db_variant = env::var("MAXMIND_DB_VARIANT").unwrap_or("GeoLite2-City".to_string());
    let db_path = env::var("DB_PATH").unwrap_or("db/".to_string());

    let update_interval: u64 = env::var("DB_UPDATE_INTERVAL_SECONDS")
        .unwrap_or("86400".to_string())
        .parse()
        .expect("Invalid DB_UPDATE_INTERVAL_SECONDS value");

    let maxmind_db = MaxmindDB::init(db_variant, db_path)
        .await
        .expect("Failed to load database");

    let maxmind_db_arc = web::Data::new(maxmind_db);

    // Start Database Updater Daemon
    maxmind_db_updater::start_db_update_daemon(maxmind_db_arc.clone(), update_interval);

    server(maxmind_db_arc).await
}

async fn server(maxmind_db_arc: web::Data<MaxmindDB>) -> Result<()> {
    let host = env::var("HOST").unwrap_or("0.0.0.0".to_string());
    let port: u16 = env::var("PORT")
        .unwrap_or("8080".to_string())
        .parse()
        .expect("Invalid PORT value");

    HttpServer::new(move || {
        let reader_data = maxmind_db_arc.clone();
        App::new()
            .app_data(reader_data)
            .service(services::lookup::handle)
            .service(services::healthcheck::handle)
    })
    .bind((host, port))?
    .run()
    .await
}
