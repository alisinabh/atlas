use crate::network_utils::SpecialIPCheck;
use crate::schemas::{GeoLocation, LookupResponse};

use actix_web::{get, web, HttpResponse, Responder};
use maxminddb::Reader;
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, RwLock};

#[get("/lookup/{ip_addresses}")]
async fn handle(
    data: web::Data<Arc<RwLock<Reader<Vec<u8>>>>>,
    path: web::Path<String>,
) -> impl Responder {
    let ip_addresses: Vec<IpAddr> = match path
        .into_inner()
        .split(',')
        .map(|ip_address| {
            ip_address.parse().map_err(|_| {
                HttpResponse::BadRequest().body(format!("Invalid IP Address {:?}", ip_address))
            })
        })
        .collect()
    {
        Ok(ip_address) => ip_address,
        Err(e) => return e,
    };

    if ip_addresses.len() > 50 {
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
