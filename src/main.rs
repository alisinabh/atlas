use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use futures_util::StreamExt;
use maxminddb::Reader;
use serde::Serialize;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::net::{AddrParseError, IpAddr};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::time::{sleep, Duration};

#[derive(Serialize)]
struct LookupResponse<'a> {
    results: HashMap<IpAddr, Option<GeoLocation<'a>>>,
    database_version: u64,
}

#[derive(Serialize)]
struct GeoLocation<'a> {
    city: Option<City<'a>>,
    country: Option<Country<'a>>,
    postal: Option<Postal<'a>>,
    subdibisions: Option<Vec<Subdivision<'a>>>,
}

#[derive(Serialize)]
struct Country<'a> {
    iso_code: Option<&'a str>,
    names: Option<BTreeMap<&'a str, &'a str>>,
}

#[derive(Serialize)]
struct City<'a> {
    names: Option<BTreeMap<&'a str, &'a str>>,
}

#[derive(Serialize)]
struct Postal<'a> {
    code: Option<&'a str>,
}

#[derive(Serialize)]
struct Subdivision<'a> {
    iso_code: Option<&'a str>,
    names: Option<BTreeMap<&'a str, &'a str>>,
}

impl<'a> GeoLocation<'a> {
    fn from_maxmind(mm_city: Option<maxminddb::geoip2::City<'a>>) -> Option<Self> {
        mm_city.map(|city| Self {
            city: City::from_maxmind(city.city),
            country: Country::from_maxmind(city.country),
            postal: Postal::from_maxmind(city.postal),
            subdibisions: city.subdivisions.map(|sub| {
                sub.into_iter()
                    .map(|sub| Subdivision::from_maxmind(Some(sub)).unwrap())
                    .collect()
            }),
        })
    }
}

impl<'a> Country<'a> {
    fn from_maxmind(mm_country: Option<maxminddb::geoip2::country::Country<'a>>) -> Option<Self> {
        mm_country.map(|country| Self {
            iso_code: country.iso_code,
            names: country.names,
        })
    }
}

impl<'a> City<'a> {
    fn from_maxmind(mm_city: Option<maxminddb::geoip2::city::City<'a>>) -> Option<Self> {
        mm_city.map(|city| Self { names: city.names })
    }
}

impl<'a> Postal<'a> {
    fn from_maxmind(mm_postal: Option<maxminddb::geoip2::city::Postal<'a>>) -> Option<Self> {
        mm_postal.map(|postal| Self { code: postal.code })
    }
}

impl<'a> Subdivision<'a> {
    fn from_maxmind(mm_sub: Option<maxminddb::geoip2::city::Subdivision<'a>>) -> Option<Self> {
        mm_sub.map(|sub| Self {
            iso_code: sub.iso_code,
            names: sub.names,
        })
    }
}

trait SpecialIPCheck {
    fn is_special_ip(&self) -> bool;
}

impl SpecialIPCheck for IpAddr {
    fn is_special_ip(&self) -> bool {
        match self {
            IpAddr::V4(ip4) => {
                ip4.is_private()
                    || ip4.is_loopback()
                    || ip4.is_multicast()
                    || ip4.is_broadcast()
                    || ip4.is_unspecified()
            }
            IpAddr::V6(ip6) => ip6.is_unspecified() || ip6.is_multicast() || ip6.is_loopback(),
        }
    }
}

#[get("/lookup/{ip_addresses}")]
async fn lookup(
    data: web::Data<Arc<RwLock<Reader<Vec<u8>>>>>,
    path: web::Path<String>,
) -> impl Responder {
    let Ok(ip_addresses): Result<Vec<IpAddr>, AddrParseError> = path
        .into_inner()
        .split(",")
        .map(|ip_address| ip_address.parse())
        .collect()
    else {
        return HttpResponse::BadRequest().body("Invalid IP Address");
    };

    if ip_addresses.len() > 5000 {
        return HttpResponse::BadRequest().body("Too many IP Addresses");
    }

    let ip_addresses = match ip_addresses
        .iter()
        .map(|&ip| {
            if ip.is_special_ip() {
                Err(HttpResponse::BadRequest().body(format!("IP Address is not allowed: {}", ip)))
            } else {
                Ok(ip)
            }
        })
        .collect::<Result<Vec<IpAddr>, HttpResponse>>()
    {
        Ok(ip_addresses) => ip_addresses,
        Err(resp) => return resp,
    };

    let reader = data.read().unwrap();

    let lookup_results: HashMap<_, _> = ip_addresses
        .iter()
        .map(|&ip| (ip, reader.lookup::<maxminddb::geoip2::City>(ip)))
        .map(|(ip, city)| (ip, GeoLocation::from_maxmind(city.ok())))
        .collect();

    HttpResponse::Ok().json(LookupResponse {
        results: lookup_results,
        database_version: reader.metadata.build_epoch,
    })
}

#[get("/health")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("Ok")
}

async fn update_db_periodically(data: web::Data<Arc<RwLock<Reader<Vec<u8>>>>>, interval: u64) -> ! {
    loop {
        let timeout = match update_db(&data).await {
            Ok(_) => interval,
            Err(err) => {
                println!("Failed to update database {:?}", err);
                60
            }
        };

        sleep(Duration::from_secs(timeout)).await;
    }
}

const DB_UPDATE_URL: &str =
    "https://download.maxmind.com/geoip/databases/GeoLite2-Country/download?suffix=tar.gz";

async fn update_db(data: &web::Data<Arc<RwLock<Reader<Vec<u8>>>>>) -> Result<(), Box<dyn Error>> {
    println!("Updating GeoIP Database...");

    let db_path = env::var("DB_PATH").unwrap_or("".to_string());

    let db_download_url = env::var("MAXMIND_DB_DOWNLOAD_URL").unwrap_or(DB_UPDATE_URL.to_string());
    let account_id = env::var("MAXMIND_ACCOUNT_ID")
        .map_err(|_| "MAXMIND_ACCOUNT_ID env var not set".to_string())?;
    let license_key = env::var("MAXMIND_LICENSE_KEY")
        .map_err(|_| "MAXMIND_LICENSE_KEY env var not set".to_string())?;

    let downloaded_filename =
        download_with_basic_auth(&db_download_url, &db_path, &account_id, Some(&license_key))
            .await?;

    extract_db(&db_path, &downloaded_filename).await?;

    let db_file_name = downloaded_filename.replace(".tar.gz", ".mmdb");
    let db_full_path = PathBuf::from(db_path).join(db_file_name);

    println!("{:?}", db_full_path);

    {
        let new_reader = Reader::open_readfile(db_full_path)?;
        let mut writer = data.write().unwrap(); // acquiring write lock
        *writer = new_reader;
    }

    println!("Database updated successfully");

    Ok(())
}

async fn extract_db(path: &str, filename: &str) -> Result<(), Box<dyn Error>> {
    let full_path = PathBuf::from(path).join(filename);

    let output = Command::new("tar")
        .arg("xvfz")
        .arg(&full_path)
        .arg("*.mmdb")
        .output()
        .await?;

    if !output.status.success() {
        return Err("failed to extract archive".into());
    }

    tokio::fs::remove_file(full_path).await?;

    let extracted_filename = &String::from_utf8(output.stderr)?.replace("\n", "")[2..];

    println!("{:?}", extracted_filename);

    Ok(())
}

async fn download_with_basic_auth(
    url: &str,
    output_path: &str,
    username: &str,
    password: Option<&str>,
) -> Result<String, Box<dyn Error>> {
    let response = reqwest::Client::new()
        .get(url)
        .basic_auth(username, password)
        .send()
        .await?;

    // Check if the request was successful
    if !response.status().is_success() {
        return Err(format!("Bad download response status code: {}", response.status()).into());
    }

    // Extract filename from Content-Disposition header
    let filename = response
        .headers()
        .get(reqwest::header::CONTENT_DISPOSITION)
        .and_then(|cd| {
            cd.to_str().ok()?.split(';').find_map(|s| {
                if s.trim().starts_with("filename=") {
                    Some(s.trim_start_matches("filename=").trim_matches('"'))
                } else {
                    None
                }
            })
        })
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Content-Disposition header missing or invalid",
            )
        })?
        .to_string();

    let full_path = PathBuf::from(output_path).join(&filename);

    // Stream the body of the response
    let mut file = File::create(full_path).await?;
    let mut stream = response.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item?;
        file.write_all(&chunk).await?;
    }

    file.flush().await?;

    Ok(filename)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let init_db_path = Path::new("database-init.mmdb");
    let reader = Reader::open_readfile(init_db_path).expect("Unable to open IP database");

    let reader_data = web::Data::new(Arc::new(RwLock::new(reader)));

    let update_interval: u64 = env::var("DB_UPDATE_INTERVAL_SECONDS")
        .unwrap_or("86400".to_string())
        .parse()
        .expect("Invalid DB_UPDATE_INTERVAL_SECONDS value");

    let reader_data_clone = reader_data.clone();
    // Spawn the periodic update task
    tokio::spawn(async move { update_db_periodically(reader_data_clone, update_interval).await });

    let host = env::var("HOST").unwrap_or("0.0.0.0".to_string());
    let port: u16 = env::var("PORT")
        .unwrap_or("8080".to_string())
        .parse()
        .expect("Invalid PORT value");

    HttpServer::new(move || {
        let reader_data = reader_data.clone();
        App::new()
            .app_data(reader_data)
            .service(lookup)
            .service(health_check)
    })
    .bind((host, port))?
    .run()
    .await
}
