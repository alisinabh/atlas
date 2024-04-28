pub mod db_refresher;
pub mod download_utils;
pub mod maxmind_db;
pub mod models;
pub mod network_utils;
pub mod services;

use actix_web::{web, App, HttpServer};
use maxmind_db::MaxmindDB;
use std::io::Result;

pub async fn init_db(db_path: &str, db_variant: &str) -> web::Data<MaxmindDB> {
    let maxmind_db = MaxmindDB::init(db_variant, db_path)
        .await
        .expect("Failed to load database");

    web::Data::new(maxmind_db)
}

pub fn start_db_refresher(maxmind_db_arc: web::Data<MaxmindDB>, update_interval: u64) {
    db_refresher::start_db_update_daemon(maxmind_db_arc.clone(), update_interval)
}

pub async fn start_server(
    maxmind_db_arc: web::Data<MaxmindDB>,
    host: &str,
    port: u16,
) -> Result<()> {
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
