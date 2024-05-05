use super::bad_request;
use crate::maxmind_db::MaxmindDB;
use crate::models::{LookupResponseModel, LookupResult};
use crate::network_utils::SpecialIPCheck;

use actix_web::{get, web, HttpResponse, Responder};
use std::net::IpAddr;

/// Lookup information on many IP addresses at once
///
/// ## Path Parameters
///
/// ### Lookup Type (`lookup_type`)
///
/// Type of the lookup. Must be one of the below values. If your Maxmind DB does not support it you
/// will get `null` values.
///
/// * `anonymous_ip`
///
/// * `asn`
///
/// * `city`
///
/// * `connection_type`
///
/// * `country`
///
/// * `density_income`
///
/// * `enterprise`
///
/// * `isp`
///
/// ### IP or IP Addresses (`ip_addresses`)
///
/// Either a single IP Address (V4 or V6) or a list of comma (`,`) separated IP Addresses.
///
/// Example: `1.1.1.1,2.2.2.2`
#[utoipa::path(
    get,
    path = "/geoip/lookup/{lookup_type}/{ip_addresses}",
    tag = "GeoIP",
    responses(
        (status = 200, description = "Ok", body = LookupResponseModel)
    ),
    params(
        ("lookup_type" = String, Path, description = "Type of the lookup", example = "city"),
        ("ip_addresses" = String, Path, description = "List of ip addresses separated by comma", example = "4.2.2.4")
    )
)]
#[get("/geoip/lookup/{lookup_type}/{ip_addresses}")]
async fn handle(data: web::Data<MaxmindDB>, path: web::Path<(String, String)>) -> impl Responder {
    let (lookup_type, ip_addresses) = path.into_inner();

    let ip_addresses: Vec<IpAddr> = match ip_addresses
        .split(',')
        .map(|ip_address| {
            ip_address.parse().map_err(|_| {
                bad_request(
                    format!("Invalid IP Address {:?}", ip_address),
                    "INVALID_IP".to_string(),
                )
            })
        })
        .collect()
    {
        Ok(ip_address) => ip_address,
        Err(e) => return e,
    };

    if ip_addresses.len() > 50 {
        return bad_request(
            "Too many IP Addresses".to_string(),
            "TOO_MANY_IPS".to_string(),
        );
    }

    let ip_addresses = match ip_addresses
        .iter()
        .map(|&ip| {
            if ip.is_special_ip() {
                Err(bad_request(
                    format!(
                        "IP Address is part of a special list and not allowed: {}",
                        ip
                    ),
                    "SPECIAL_IP".to_string(),
                ))
            } else {
                Ok(ip)
            }
        })
        .collect::<Result<Vec<IpAddr>, HttpResponse>>()
    {
        Ok(ip_addresses) => ip_addresses,
        Err(resp) => return resp,
    };

    let db_inner = data.db.read().await;

    let results: LookupResult = match lookup_type.as_str() {
        "anonymous_ip" => LookupResult::AnonymousIp(db_inner.lookup(ip_addresses).await),
        "asn" => LookupResult::Asn(db_inner.lookup(ip_addresses).await),
        "city" => LookupResult::City(db_inner.lookup(ip_addresses).await),
        "connection_type" => LookupResult::ConnectionType(db_inner.lookup(ip_addresses).await),
        "country" => LookupResult::Country(db_inner.lookup(ip_addresses).await),
        "density_income" => LookupResult::DensityIncome(db_inner.lookup(ip_addresses).await),
        "enterprise" => LookupResult::Enterprise(db_inner.lookup(ip_addresses).await),
        "isp" => LookupResult::Isp(db_inner.lookup(ip_addresses).await),
        _ => {
            return bad_request(
                "invalid lookup_type".to_string(),
                "INVALID_LOOKUP_TYPE".to_string(),
            )
        }
    };

    HttpResponse::Ok().json(LookupResponseModel {
        results,
        database_build_epoch: db_inner.build_epoch(),
    })
}
