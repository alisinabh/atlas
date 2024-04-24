use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Serialize)]
pub struct Subdivision {
    iso_code: Option<String>,
    names: Option<BTreeMap<String, String>>,
}

impl Subdivision {
    pub fn from_maxmind(mm_sub: Option<maxminddb::geoip2::city::Subdivision>) -> Option<Self> {
        mm_sub.map(|sub| Self {
            iso_code: sub.iso_code.map(|s| s.to_string()),
            names: sub.names.map(|m| {
                m.iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect()
            }),
        })
    }
}
