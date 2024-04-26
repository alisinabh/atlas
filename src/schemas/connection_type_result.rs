use super::{LookupResult, Lookupable};
use maxminddb::MaxMindDBError;
use serde::Serialize;

#[derive(Serialize)]
pub struct ConnectionTypeResult {
    connection_type: Option<String>,
}

impl ConnectionTypeResult {
    pub fn from_maxmind(connection_type: maxminddb::geoip2::ConnectionType) -> LookupResult {
        LookupResult::ConnectionType(Self {
            connection_type: connection_type.connection_type.map(|s| s.to_string()),
        })
    }
}

impl Lookupable for ConnectionTypeResult {
    fn lookup<T: AsRef<[u8]>>(
        reader: &maxminddb::Reader<T>,
        ip: std::net::IpAddr,
    ) -> Result<LookupResult, MaxMindDBError> {
        reader
            .lookup::<maxminddb::geoip2::ConnectionType>(ip)
            .map(Self::from_maxmind)
    }
}
