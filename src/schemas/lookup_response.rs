use super::GeoLocation;
use serde::Serialize;
use std::collections::HashMap;
use std::net::IpAddr;

#[derive(Serialize)]
pub struct LookupResponse<'a> {
    pub results: HashMap<IpAddr, Option<GeoLocation<'a>>>,
    pub database_build_epoch: u64,
}
