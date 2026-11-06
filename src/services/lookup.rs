use super::bad_request;
use crate::maxmind_db::MaxmindDB;
use crate::models::{LookupResponseModel, LookupResult};
use crate::network_utils::SpecialIPCheck;

use actix_web::http::header::{CacheControl, CacheDirective};
use actix_web::{HttpResponse, Responder, get, web};
use std::net::IpAddr;

const MAX_IP_ADDRESSES: usize = 50;
/// Results only change when the database is updated, so clients may cache them for a while
const CACHE_MAX_AGE_SECS: u32 = 60 * 60;

/// Lookup information on many IP addresses at once
///
/// ## Path Parameters
///
/// ### Lookup Type (`lookup_type`)
///
/// Type of the lookup. Must be one of the below values, otherwise `INVALID_LOOKUP_TYPE` is
/// returned. If your Maxmind DB does not contain data for the lookup type (e.g. `isp` on a City
/// database) every result is `null`.
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
/// Surrounding whitespace is ignored. At most 50 addresses are allowed per request.
///
/// Example: `1.1.1.1,2.2.2.2`
///
/// ## Response
///
/// `results` maps every requested IP Address to its lookup result. Duplicate addresses appear
/// once. The result is `null` if the address is not in the database or the database has no data
/// for the lookup type. Responses may be cached for one hour (`Cache-Control`).
///
/// ## Errors
///
/// A `400` response contains `{"error": {"message": ..., "code": ...}}` with one of these codes:
///
/// * `INVALID_IP`: an entry is not a valid IP Address, including empty entries such as a
///   trailing comma
///
/// * `SPECIAL_IP`: an address is private, loopback, link-local, multicast, reserved or otherwise
///   not publicly routable (IPv4-mapped IPv6 addresses are checked as IPv4)
///
/// * `TOO_MANY_IPS`: more than 50 addresses were requested
///
/// * `INVALID_LOOKUP_TYPE`: `lookup_type` is not one of the values above
#[utoipa::path(
    get,
    path = "/geoip/lookup/{lookup_type}/{ip_addresses}",
    operation_id = "lookup",
    tag = "GeoIP",
    responses(
        (status = 200, description = "Ok", body = LookupResponseModel),
        (status = 400, description = "Invalid request, see `error.code`")
    ),
    params(
        ("lookup_type" = String, Path, description = "Type of the lookup", example = "city"),
        ("ip_addresses" = String, Path, description = "List of ip addresses separated by comma", example = "4.2.2.4")
    )
)]
#[get("/geoip/lookup/{lookup_type}/{ip_addresses}")]
async fn handle(data: web::Data<MaxmindDB>, path: web::Path<(String, String)>) -> impl Responder {
    let (lookup_type, ip_addresses) = path.into_inner();

    // Bail out before parsing anything if there are too many entries
    let ip_addresses: Vec<&str> = ip_addresses.split(',').take(MAX_IP_ADDRESSES + 1).collect();

    if ip_addresses.len() > MAX_IP_ADDRESSES {
        return bad_request(
            "Too many IP Addresses".to_string(),
            "TOO_MANY_IPS".to_string(),
        );
    }

    let ip_addresses: Vec<IpAddr> = match ip_addresses
        .into_iter()
        .map(str::trim)
        .map(parse_ip_address)
        .collect()
    {
        Ok(ip_addresses) => ip_addresses,
        Err((message, code)) => return bad_request(message, code),
    };

    let db = data.current();

    let results: LookupResult = match lookup_type.as_str() {
        "anonymous_ip" => LookupResult::AnonymousIp(db.lookup(&ip_addresses)),
        "asn" => LookupResult::Asn(db.lookup(&ip_addresses)),
        "city" => LookupResult::City(db.lookup(&ip_addresses)),
        "connection_type" => LookupResult::ConnectionType(db.lookup(&ip_addresses)),
        "country" => LookupResult::Country(db.lookup(&ip_addresses)),
        "density_income" => LookupResult::DensityIncome(db.lookup(&ip_addresses)),
        "enterprise" => LookupResult::Enterprise(db.lookup(&ip_addresses)),
        "isp" => LookupResult::Isp(db.lookup(&ip_addresses)),
        _ => {
            return bad_request(
                "invalid lookup_type".to_string(),
                "INVALID_LOOKUP_TYPE".to_string(),
            );
        }
    };

    HttpResponse::Ok()
        .insert_header(CacheControl(vec![
            CacheDirective::Public,
            CacheDirective::MaxAge(CACHE_MAX_AGE_SECS),
        ]))
        .json(LookupResponseModel {
            results,
            database_build_epoch: db.build_epoch(),
        })
}

/// Returns the error message and code on failure
fn parse_ip_address(ip_address: &str) -> Result<IpAddr, (String, String)> {
    let ip: IpAddr = ip_address.parse().map_err(|_| {
        (
            format!("Invalid IP Address {ip_address:?}"),
            "INVALID_IP".to_string(),
        )
    })?;

    if ip.is_special_ip() {
        return Err((
            format!("IP Address is part of a special list and not allowed: {ip}"),
            "SPECIAL_IP".to_string(),
        ));
    }

    Ok(ip)
}
