use std::io::{Error, ErrorKind, Result};

use atlas_rs::api_docs;
use atlas_rs::config::Config;
use tokio::io::AsyncWriteExt;

const SPEC_FILENAME: &str = "openapi-spec.json";

#[actix_web::main]
async fn main() -> Result<()> {
    let subcommand = std::env::args().nth(1);

    match subcommand.as_deref() {
        Some("server") | None => {
            let config = load_config()?;

            // Load or Initialize MaxMind database
            let maxmind_db_arc =
                atlas_rs::init_db(&config.db_path, &config.db_variant, config.download)
                    .await
                    .map_err(|reason| {
                        Error::other(format!("Failed to load/initialize database: {reason}"))
                    })?;

            tokio::select! {
                // Start Database Updater Daemon
                _ = atlas_rs::start_db_refresher(maxmind_db_arc.clone(), config.update_interval) => {}
                // Start Server
                _ = atlas_rs::start_server(maxmind_db_arc, &config.host, config.port, config.swagger_ui_enabled) => {}
            }

            Ok(())
        }
        Some("init") => {
            let config = load_config()?;

            atlas_rs::init_db(&config.db_path, &config.db_variant, config.download)
                .await
                .map_err(|reason| {
                    Error::other(format!("Failed to initialize database: {reason}"))
                })?;

            println!("Database initiation was successful");

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
            format!("Invalid command: {command}"),
        )),
    }
}

fn load_config() -> Result<Config> {
    Config::from_env().map_err(|reason| Error::new(ErrorKind::InvalidInput, reason))
}
