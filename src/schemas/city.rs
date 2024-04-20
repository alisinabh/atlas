use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Serialize)]
pub struct City<'a> {
    names: Option<BTreeMap<&'a str, &'a str>>,
}

impl<'a> City<'a> {
    pub fn from_maxmind(mm_city: Option<maxminddb::geoip2::city::City<'a>>) -> Option<Self> {
        mm_city.map(|city| Self { names: city.names })
    }
}
