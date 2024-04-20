use super::{City, Country, Postal, Subdivision};
use serde::Serialize;

#[derive(Serialize)]
pub struct GeoLocation<'a> {
    city: Option<City<'a>>,
    country: Option<Country<'a>>,
    postal: Option<Postal<'a>>,
    subdivisions: Option<Vec<Subdivision<'a>>>,
}

impl<'a> GeoLocation<'a> {
    pub fn from_maxmind(mm_city: Option<maxminddb::geoip2::City<'a>>) -> Option<Self> {
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
