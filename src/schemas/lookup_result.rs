use serde::Serialize;
use std::collections::HashMap;
use std::net::IpAddr;

#[derive(Serialize)]
pub struct LookupResult<T: Serialize> {
    pub results: HashMap<IpAddr, Option<T>>,
    pub database_build_epoch: u64,
}
