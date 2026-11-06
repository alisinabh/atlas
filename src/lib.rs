pub mod api_docs;
pub mod config;
pub mod db_refresher;
pub mod download_utils;
pub mod maxmind_db;
pub mod models;
pub mod network_utils;
pub mod services;

use std::error::Error;

use actix_web::{App, HttpServer, web};
use config::DownloadConfig;
use maxmind_db::MaxmindDB;
use utoipa_swagger_ui::SwaggerUi;

pub type BoxError = Box<dyn Error + Send + Sync>;

pub async fn init_db(
    db_path: &str,
    db_variant: &str,
    download: DownloadConfig,
) -> Result<web::Data<MaxmindDB>, BoxError> {
    let maxmind_db = MaxmindDB::init(db_variant, db_path, download).await?;

    Ok(web::Data::new(maxmind_db))
}

pub async fn start_db_refresher(maxmind_db_arc: web::Data<MaxmindDB>, update_interval: u64) {
    db_refresher::start_db_update_daemon(maxmind_db_arc, update_interval).await;
}

/// Registers all routes. MaxmindDB app data must be added separately.
pub fn configure_app(swagger_ui_enabled: bool) -> impl FnOnce(&mut web::ServiceConfig) {
    move |cfg| {
        cfg.service(services::lookup::handle)
            .service(services::healthcheck::handle);

        if swagger_ui_enabled {
            cfg.service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", api_docs::api_doc()),
            )
            .service(web::redirect("/swagger-ui", "/swagger-ui/"));
        }
    }
}

pub async fn start_server(
    maxmind_db_arc: web::Data<MaxmindDB>,
    host: &str,
    port: u16,
    swagger_ui_enabled: bool,
) {
    // Start HTTP Server
    HttpServer::new(move || {
        App::new()
            .app_data(maxmind_db_arc.clone())
            .configure(configure_app(swagger_ui_enabled))
    })
    .bind((host, port))
    .expect("Cannot bind to specified host and port")
    .run()
    .await
    .expect("HTTP Server crashed");
}
