use super::{City, Country, LookupResult, Lookupable, Postal, Subdivision};
use maxminddb::MaxMindDBError;
use serde::Serialize;

#[derive(Serialize)]
pub struct CityResult {
    city: Option<City>,
    country: Option<Country>,
    postal: Option<Postal>,
    subdivisions: Option<Vec<Subdivision>>,
}

impl CityResult {
    pub fn from_maxmind(city: maxminddb::geoip2::City) -> LookupResult {
        LookupResult::City(Self {
            city: City::from_maxmind(city.city),
            country: Country::from_maxmind(city.country),
            postal: Postal::from_maxmind(city.postal),
            subdivisions: city.subdivisions.map(|sub| {
                sub.into_iter()
                    .map(|sub| Subdivision::from_maxmind(Some(sub)).unwrap())
                    .collect()
            }),
        })
    }
}

impl Lookupable for CityResult {
    fn lookup<T: AsRef<[u8]>>(
        reader: &maxminddb::Reader<T>,
        ip: std::net::IpAddr,
    ) -> Result<LookupResult, MaxMindDBError> {
        reader
            .lookup::<maxminddb::geoip2::City>(ip)
            .map(Self::from_maxmind)
    }
}
