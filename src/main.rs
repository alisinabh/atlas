use std::env;
use std::io::{Error, ErrorKind, Result};

use atlas_rs::api_docs;
use tokio::io::AsyncWriteExt;

const SPEC_FILENAME: &str = "openapi-spec.json";

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

    let subcommand = env::args().nth(1);

    match subcommand.as_deref() {
        Some("server") | None => {
            // Load or Initialize MaxMind database
            let maxmind_db_arc = atlas_rs::init_db(&db_path, &db_variant)
                .await
                .expect("Failed to load/initialize database");

            tokio::select! {
                // Start Database Updater Daemon
                _ = atlas_rs::start_db_refresher(maxmind_db_arc.clone(), update_interval) => {}
                // Start Server
                _ = atlas_rs::start_server(maxmind_db_arc, &host, port, swagger_ui_enabled) => {}
            }

            Ok(())
        }
        Some("init") => {
            match atlas_rs::init_db(&db_path, &db_variant).await {
                Ok(_) => println!("Database initiation was successful"),
                Err(reason) => print!("Failed to initialize database {reason:?}"),
            }

            Ok(())
        }
        Some("spec") => {
            let api_doc = api_docs::api_doc();
            let json_api_doc = api_doc.to_json().expect("Failed to generate API spec");

            let mut file = tokio::fs::File::create(SPEC_FILENAME)
                .await
                .expect("Could not create spec file");

            file.write_all(json_api_doc.as_bytes())
                .await
                .expect("Could not write to file");

            println!("Generated {SPEC_FILENAME}");

            Ok(())
        }
        Some(command) => Err(Error::new(
            ErrorKind::InvalidInput,
            format!("Invalid command: {}", command),
        )),
    }
}
