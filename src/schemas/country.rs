use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Serialize)]
pub struct Country<'a> {
    iso_code: Option<&'a str>,
    names: Option<BTreeMap<&'a str, &'a str>>,
}

impl<'a> Country<'a> {
    pub fn from_maxmind(
        mm_country: Option<maxminddb::geoip2::country::Country<'a>>,
    ) -> Option<Self> {
        mm_country.map(|country| Self {
            iso_code: country.iso_code,
            names: country.names,
        })
    }
}
