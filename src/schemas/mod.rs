mod city;
mod city_result;
mod connection_type_result;
mod country;
mod postal;
mod subdivision;

use maxminddb::{MaxMindDBError, Reader};
use serde::Serialize;
use std::net::IpAddr;

pub use city_result::*;
pub use connection_type_result::*;

use city::*;
use country::*;
use postal::*;
use subdivision::*;

pub trait Lookupable {
    fn lookup<T: AsRef<[u8]>>(
        reader: &Reader<T>,
        ip: IpAddr,
    ) -> Result<LookupResult, MaxMindDBError>
    where
        Self: Sized;
}

pub enum LookupResult {
    City(CityResult),
    ConnectionType(ConnectionTypeResult),
}

impl Serialize for LookupResult {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            LookupResult::City(c) => c.serialize(serializer),
            LookupResult::ConnectionType(c) => c.serialize(serializer),
        }
    }
}
