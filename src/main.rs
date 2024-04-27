mod download_utils;
mod maxmind_db;
mod maxmind_db_refresher;
mod models;
mod network_utils;
mod services;

use actix_web::{web, App, HttpServer};
use maxmind_db::MaxmindDB;
use std::env::{self, args};
use std::io::{Error, ErrorKind, Result};

#[actix_web::main]
async fn main() -> Result<()> {
    let db_variant = env::var("MAXMIND_DB_VARIANT").unwrap_or("GeoLite2-City".to_string());
    let db_path = env::var("DB_PATH").unwrap_or("db/".to_string());

    let maxmind_db = MaxmindDB::init(db_variant, db_path)
        .await
        .expect("Failed to load database");

    let maxmind_db_arc = web::Data::new(maxmind_db);

    let subcommand = args().skip(1).next();

    match subcommand.as_deref() {
        Some("server") | None => server(maxmind_db_arc).await,
        Some("init") => Ok(()),
        Some(command) => Err(Error::new(
            ErrorKind::InvalidInput,
            format!("Invalid command: {}", command),
        )),
    }
}

async fn server(maxmind_db_arc: web::Data<MaxmindDB>) -> Result<()> {
    let update_interval: u64 = env::var("DB_UPDATE_INTERVAL_SECONDS")
        .unwrap_or("86400".to_string())
        .parse()
        .expect("Invalid DB_UPDATE_INTERVAL_SECONDS value");

    let host = env::var("HOST").unwrap_or("0.0.0.0".to_string());
    let port: u16 = env::var("PORT")
        .unwrap_or("8080".to_string())
        .parse()
        .expect("Invalid PORT value");

    // Start Database Updater Daemon
    maxmind_db_refresher::start_db_update_daemon(maxmind_db_arc.clone(), update_interval);

    // Start HTTP Server
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
