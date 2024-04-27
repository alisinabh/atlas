use serde::Serialize;
use std::collections::HashMap;
use std::net::IpAddr;

use maxminddb::geoip2::{
    AnonymousIp, Asn, City, ConnectionType, Country, DensityIncome, Enterprise, Isp,
};

type LookupHashMap<T> = HashMap<IpAddr, Option<T>>;

pub enum LookupResult<'a> {
    AnonymousIp(LookupHashMap<AnonymousIp>),
    Asn(LookupHashMap<Asn<'a>>),
    City(LookupHashMap<City<'a>>),
    ConnectionType(LookupHashMap<ConnectionType<'a>>),
    Country(LookupHashMap<Country<'a>>),
    DensityIncome(LookupHashMap<DensityIncome>),
    Enterprise(LookupHashMap<Enterprise<'a>>),
    Isp(LookupHashMap<Isp<'a>>),
}

impl<'a> Serialize for LookupResult<'a> {
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
pub struct LookupResponseModel<'a> {
    pub results: LookupResult<'a>,
    pub database_build_epoch: u64,
}
