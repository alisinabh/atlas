pub mod api_docs;
pub mod db_refresher;
pub mod download_utils;
pub mod maxmind_db;
pub mod models;
pub mod network_utils;
pub mod services;

use actix_web::{web, App, HttpServer};
use maxmind_db::MaxmindDB;
use std::io::Result;
use utoipa_swagger_ui::SwaggerUi;

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
    swagger_ui_enabled: bool,
) -> Result<()> {
    // Start HTTP Server
    HttpServer::new(move || {
        let reader_data = maxmind_db_arc.clone();
        let app = App::new()
            .app_data(reader_data)
            .service(services::lookup::handle)
            .service(services::healthcheck::handle);

        if swagger_ui_enabled {
            app.service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", api_docs::api_doc()),
            )
            .service(web::redirect("/swagger-ui", "/swagger-ui/"))
        } else {
            app
        }
    })
    .bind((host, port))?
    .run()
    .await
}
