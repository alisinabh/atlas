use serde::Serialize;

#[derive(Serialize)]
pub struct Postal<'a> {
    code: Option<&'a str>,
}

impl<'a> Postal<'a> {
    pub fn from_maxmind(mm_postal: Option<maxminddb::geoip2::city::Postal<'a>>) -> Option<Self> {
        mm_postal.map(|postal| Self { code: postal.code })
    }
}
