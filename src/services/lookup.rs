use super::bad_request;
use crate::maxmind_db::MaxmindDB;
use crate::network_utils::SpecialIPCheck;

use maxminddb::geoip2::{
    AnonymousIp, Asn, City, ConnectionType, Country, DensityIncome, Enterprise, Isp,
};

use actix_web::{get, web, HttpResponse, Responder};
use serde::Serialize;
use std::collections::HashMap;
use std::net::IpAddr;

type LookupRes<T> = HashMap<IpAddr, Option<T>>;

enum LookupType<'a> {
    AnonymousIp(LookupRes<AnonymousIp>),
    Asn(LookupRes<Asn<'a>>),
    City(LookupRes<City<'a>>),
    ConnectionType(LookupRes<ConnectionType<'a>>),
    Country(LookupRes<Country<'a>>),
    DensityIncome(LookupRes<DensityIncome>),
    Enterprise(LookupRes<Enterprise<'a>>),
    Isp(LookupRes<Isp<'a>>),
}

impl<'a> Serialize for LookupType<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::AnonymousIp(anonymous_ip) => anonymous_ip.serialize(serializer),
            Self::Asn(asn) => asn.serialize(serializer),
            Self::City(city) => city.serialize(serializer),
            Self::ConnectionType(connection_type) => connection_type.serialize(serializer),
            Self::Country(country) => country.serialize(serializer),
            Self::DensityIncome(density_income) => density_income.serialize(serializer),
            Self::Enterprise(enterprise) => enterprise.serialize(serializer),
            Self::Isp(isp) => isp.serialize(serializer),
        }
    }
}

#[derive(Serialize)]
struct LookupResponseModel<'a> {
    pub results: LookupType<'a>,
    pub database_build_epoch: u64,
}

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

    let db_inner = data.db.read().await;

    let results: LookupType = match lookup_type.as_str() {
        "anonymous_ip" => LookupType::AnonymousIp(db_inner.lookup(ip_addresses).await),
        "asn" => LookupType::Asn(db_inner.lookup(ip_addresses).await),
        "city" => LookupType::City(db_inner.lookup(ip_addresses).await),
        "connection_type" => LookupType::ConnectionType(db_inner.lookup(ip_addresses).await),
        "country" => LookupType::Country(db_inner.lookup(ip_addresses).await),
        "density_income" => LookupType::DensityIncome(db_inner.lookup(ip_addresses).await),
        "enterprise" => LookupType::Enterprise(db_inner.lookup(ip_addresses).await),
        "isp" => LookupType::Isp(db_inner.lookup(ip_addresses).await),
        _ => return bad_request("invalid lookup_type".to_string()),
    };

    HttpResponse::Ok().json(LookupResponseModel {
        results,
        database_build_epoch: db_inner.build_epoch(),
    })
}
