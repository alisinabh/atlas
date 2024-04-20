use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Serialize)]
pub struct Subdivision<'a> {
    iso_code: Option<&'a str>,
    names: Option<BTreeMap<&'a str, &'a str>>,
}

impl<'a> Subdivision<'a> {
    pub fn from_maxmind(mm_sub: Option<maxminddb::geoip2::city::Subdivision<'a>>) -> Option<Self> {
        mm_sub.map(|sub| Self {
            iso_code: sub.iso_code,
            names: sub.names,
        })
    }
}
