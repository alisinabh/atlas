use serde::Serialize;

#[derive(Serialize)]
pub struct Postal {
    code: Option<String>,
}

impl Postal {
    pub fn from_maxmind(mm_postal: Option<maxminddb::geoip2::city::Postal>) -> Option<Self> {
        mm_postal.map(|postal| Self {
            code: postal.code.map(|s| s.to_string()),
        })
    }
}
