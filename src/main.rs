use std::env::{self, args};
use std::io::{Error, ErrorKind, Result};

#[actix_web::main]
async fn main() -> Result<()> {
    let db_variant = env::var("MAXMIND_DB_VARIANT").unwrap_or("GeoLite2-City".to_string());
    let db_path = env::var("DB_PATH").unwrap_or("/opt/atlas/db".to_string());

    let update_interval: u64 = env::var("DB_UPDATE_INTERVAL_SECONDS")
        .unwrap_or("86400".to_string())
        .parse()
        .expect("Invalid DB_UPDATE_INTERVAL_SECONDS value");

    let host = env::var("HOST").unwrap_or("0.0.0.0".to_string());
    let port: u16 = env::var("PORT")
        .unwrap_or("8080".to_string())
        .parse()
        .expect("Invalid PORT value");

    let swagger_ui_enabled: bool = env::var("SWAGGER_UI_ENABLED")
        .unwrap_or("false".to_string())
        .parse()
        .expect("Invalid SWAGGER_UI_ENABLED value. Expected `false` or `true`");

    let maxmind_db_arc = atlas_rs::init_db(&db_path, &db_variant).await;

    let subcommand = args().skip(1).next();

    match subcommand.as_deref() {
        Some("server") | None => {
            // Start Database Updater Daemon
            atlas_rs::start_db_refresher(maxmind_db_arc.clone(), update_interval);
            // Start Server
            atlas_rs::start_server(maxmind_db_arc, &host, port, swagger_ui_enabled).await
        }
        Some("init") => Ok(()),
        Some(command) => Err(Error::new(
            ErrorKind::InvalidInput,
            format!("Invalid command: {}", command),
        )),
    }
}
