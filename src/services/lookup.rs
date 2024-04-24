use super::bad_request;
use crate::maxmind_db::MaxmindDB;
use crate::network_utils::SpecialIPCheck;
use crate::schemas::{CityResult, ConnectionTypeResult};

use actix_web::{get, web, HttpResponse, Responder};
use std::net::IpAddr;

#[get("/lookup/{lookup_type}/{ip_addresses}")]
async fn handle(data: web::Data<MaxmindDB>, path: web::Path<(String, String)>) -> impl Responder {
    let (lookup_type, ip_addresses) = path.into_inner();

    let ip_addresses: Vec<IpAddr> = match ip_addresses
        .split(',')
        .map(|ip_address| {
            ip_address
                .parse()
                .map_err(|_| bad_request(format!("Invalid IP Address {:?}", ip_address)))
        })
        .collect()
    {
        Ok(ip_address) => ip_address,
        Err(e) => return e,
    };

    if ip_addresses.len() > 50 {
        return bad_request("Too many IP Addresses".to_string());
    }

    let ip_addresses = match ip_addresses
        .iter()
        .map(|&ip| {
            if ip.is_special_ip() {
                Err(bad_request(format!("IP Address is not allowed: {}", ip)))
            } else {
                Ok(ip)
            }
        })
        .collect::<Result<Vec<IpAddr>, HttpResponse>>()
    {
        Ok(ip_addresses) => ip_addresses,
        Err(resp) => return resp,
    };

    let lookup_result = match lookup_type.as_str() {
        "city" => serde_json::to_string(&data.lookup::<CityResult>(ip_addresses).await),
        "connection_type" => {
            serde_json::to_string(&data.lookup::<ConnectionTypeResult>(ip_addresses).await)
        }
        _ => return bad_request("invalid lookup_type".to_string()),
    }
    .unwrap();

    HttpResponse::Ok().json(lookup_result)
}
