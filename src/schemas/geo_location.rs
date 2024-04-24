use super::{City, Country, Postal, Subdivision};
use serde::Serialize;

#[derive(Serialize)]
pub struct GeoLocation {
    city: Option<City>,
    country: Option<Country>,
    postal: Option<Postal>,
    subdivisions: Option<Vec<Subdivision>>,
}

impl GeoLocation {
    pub fn from_maxmind(mm_city: Option<maxminddb::geoip2::City>) -> Option<Self> {
        mm_city.map(|city| Self {
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
