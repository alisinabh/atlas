use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Serialize)]
pub struct Country {
    iso_code: Option<String>,
    names: Option<BTreeMap<String, String>>,
}

impl Country {
    pub fn from_maxmind(mm_country: Option<maxminddb::geoip2::country::Country>) -> Option<Self> {
        mm_country.map(|country| Self {
            iso_code: country.iso_code.map(|s| s.to_string()),
            names: country.names.map(|m| {
                m.iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect()
            }),
        })
    }
}
