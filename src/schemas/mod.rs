mod city;
mod city_result;
mod connection_type_result;
mod country;
mod lookup_result;
mod postal;
mod subdivision;

use city::*;
pub use city_result::*;
pub use connection_type_result::*;
use country::*;
pub use lookup_result::*;
use postal::*;
use subdivision::*;

use maxminddb::Reader;
use std::net::IpAddr;

pub trait Lookupable {
    fn lookup<T: AsRef<[u8]>>(reader: &Reader<T>, ip: IpAddr) -> Option<Self>
    where
        Self: Sized;
}
