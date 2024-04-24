use super::GeoLocation;
use serde::Serialize;
use std::collections::HashMap;
use std::net::IpAddr;

#[derive(Serialize)]
pub struct LookupResult {
    pub results: HashMap<IpAddr, Option<GeoLocation>>,
    pub database_build_epoch: u64,
}
