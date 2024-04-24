use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Serialize)]
pub struct City {
    names: Option<BTreeMap<String, String>>,
}

impl City {
    pub fn from_maxmind(mm_city: Option<maxminddb::geoip2::city::City>) -> Option<Self> {
        mm_city.map(|city| Self {
            names: city.names.map(|m| {
                m.iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect()
            }),
        })
    }
}
