use serde::Serialize;
use std::collections::HashMap;
use std::net::IpAddr;
use utoipa::{
    openapi::{schema::AdditionalProperties, ObjectBuilder},
    ToSchema,
};

use maxminddb::geoip2::{
    AnonymousIp, Asn, City, ConnectionType, Country, DensityIncome, Enterprise, Isp,
};

type LookupHashMap<T> = HashMap<IpAddr, Option<T>>;

pub enum LookupResult<'a> {
    AnonymousIp(LookupHashMap<AnonymousIp>),
    Asn(LookupHashMap<Asn<'a>>),
    City(LookupHashMap<City<'a>>),
    ConnectionType(LookupHashMap<ConnectionType<'a>>),
    Country(LookupHashMap<Country<'a>>),
    DensityIncome(LookupHashMap<DensityIncome>),
    Enterprise(LookupHashMap<Enterprise<'a>>),
    Isp(LookupHashMap<Isp<'a>>),
}

impl<'a> Serialize for LookupResult<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::AnonymousIp(anonymous_ip) => anonymous_ip.serialize(serializer),
            Self::Asn(asn) => asn.serialize(serializer),
            Self::City(city) => city.serialize(serializer),
            Self::ConnectionType(connection_type) => connection_type.serialize(serializer),
            Self::Country(country) => country.serialize(serializer),
            Self::DensityIncome(density_income) => density_income.serialize(serializer),
            Self::Enterprise(enterprise) => enterprise.serialize(serializer),
            Self::Isp(isp) => isp.serialize(serializer),
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct LookupResponseModel<'a> {
    pub results: LookupResult<'a>,
    pub database_build_epoch: u64,
}

pub struct HealthCheckModel;

impl<'__s> utoipa::ToSchema<'__s> for HealthCheckModel {
    fn schema() -> (
        &'__s str,
        utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
    ) {
        (
            "HealthCheckModel",
            ObjectBuilder::new()
                .schema_type(utoipa::openapi::SchemaType::String)
                .example(Some("Ok".into()))
                .into(),
        )
    }
}

impl<'__s> utoipa::ToSchema<'__s> for LookupResult<'__s> {
    fn schema() -> (
        &'__s str,
        utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
    ) {
        (
            "LookupResult",
            utoipa::openapi::ObjectBuilder::new()
                .property(
                    "{ip_address}",
                    utoipa::openapi::Schema::OneOf(
                        utoipa::openapi::schema::OneOfBuilder::new()
                            .item(LookupResult::city_schema())
                            .into(),
                    ),
                )
                .into(),
        )
    }
}

impl<'a> LookupResult<'a> {
    fn localized_string() -> ObjectBuilder {
        ObjectBuilder::new()
            .schema_type(utoipa::openapi::SchemaType::Object)
            .title(Some("LocalizedString"))
            .property(
                "{language_code}",
                ObjectBuilder::new().schema_type(utoipa::openapi::SchemaType::String),
            )
            .additional_properties(Some(AdditionalProperties::FreeForm(true)))
    }

    fn geoname_id() -> ObjectBuilder {
        ObjectBuilder::new()
            .schema_type(utoipa::openapi::SchemaType::Integer)
            .example(Some(serde_json::Value::Number(1234.into())))
    }

    fn city_schema() -> ObjectBuilder {
        ObjectBuilder::new()
            .title(Some("CityLookupResult"))
            .schema_type(utoipa::openapi::SchemaType::Object)
            .property(
                "city",
                ObjectBuilder::new()
                    .property("geoname_id", Self::geoname_id())
                    .property(
                        "names",
                        Self::localized_string()
                            .example(serde_json::json!({"en": "San Diego"}).into()),
                    ),
            )
            .property(
                "continent",
                ObjectBuilder::new()
                    .property(
                        "code",
                        ObjectBuilder::new()
                            .schema_type(utoipa::openapi::SchemaType::String)
                            .example(Some("NA".into())),
                    )
                    .property("geoname_id", Self::geoname_id())
                    .property(
                        "names",
                        Self::localized_string()
                            .example(serde_json::json!({"en": "North America"}).into()),
                    ),
            )
            .property(
                "country",
                ObjectBuilder::new()
                    .property("geoname_id", Self::geoname_id())
                    .property(
                        "is_in_european_union",
                        ObjectBuilder::new().schema_type(utoipa::openapi::SchemaType::Boolean),
                    )
                    .property(
                        "iso_code",
                        ObjectBuilder::new()
                            .schema_type(utoipa::openapi::SchemaType::String)
                            .example(Some("US".into())),
                    )
                    .property(
                        "names",
                        Self::localized_string()
                            .example(serde_json::json!({"en": "United States"}).into()),
                    ),
            )
            .property(
                "location",
                ObjectBuilder::new()
                    .property(
                        "accuracy_radius",
                        ObjectBuilder::new()
                            .schema_type(utoipa::openapi::SchemaType::Number)
                            .example(Some(200.into())),
                    )
                    .property(
                        "latitude",
                        ObjectBuilder::new()
                            .schema_type(utoipa::openapi::SchemaType::Number)
                            .example(Some((30.0406).into())),
                    )
                    .property(
                        "longitude",
                        ObjectBuilder::new()
                            .schema_type(utoipa::openapi::SchemaType::Number)
                            .example(Some((-80.26).into())),
                    )
                    .property(
                        "metro_code",
                        ObjectBuilder::new()
                            .schema_type(utoipa::openapi::SchemaType::Number)
                            .example(Some(518.into())),
                    )
                    .property(
                        "time_zone",
                        ObjectBuilder::new()
                            .schema_type(utoipa::openapi::SchemaType::String)
                            .example(Some("America/New_York".into())),
                    ),
            )
            .property(
                "postal",
                ObjectBuilder::new().property(
                    "code",
                    ObjectBuilder::new()
                        .schema_type(utoipa::openapi::SchemaType::String)
                        .example(Some("27127".into())),
                ),
            )
            .property(
                "registered_country",
                ObjectBuilder::new()
                    .property("geoname_id", Self::geoname_id())
                    .property(
                        "is_in_european_union",
                        ObjectBuilder::new().schema_type(utoipa::openapi::SchemaType::Boolean),
                    )
                    .property(
                        "iso_code",
                        ObjectBuilder::new()
                            .schema_type(utoipa::openapi::SchemaType::String)
                            .example(Some("US".into())),
                    )
                    .property(
                        "names",
                        Self::localized_string()
                            .example(serde_json::json!({"en": "United States"}).into()),
                    ),
            )
            .property(
                "represented_country",
                ObjectBuilder::new()
                    .property("geoname_id", Self::geoname_id())
                    .property(
                        "is_in_european_union",
                        ObjectBuilder::new().schema_type(utoipa::openapi::SchemaType::Boolean),
                    )
                    .property(
                        "iso_code",
                        ObjectBuilder::new()
                            .schema_type(utoipa::openapi::SchemaType::String)
                            .example(Some("US".into())),
                    )
                    .property(
                        "names",
                        Self::localized_string()
                            .example(serde_json::json!({"en": "United States"}).into()),
                    )
                    .property(
                        "type",
                        ObjectBuilder::new().schema_type(utoipa::openapi::SchemaType::String),
                    ),
            )
            .property(
                "subdivisions",
                ObjectBuilder::new()
                    .property("geoname_id", Self::geoname_id())
                    .property(
                        "iso_code",
                        ObjectBuilder::new()
                            .schema_type(utoipa::openapi::SchemaType::String)
                            .example(Some("NC".into())),
                    )
                    .property(
                        "names",
                        Self::localized_string()
                            .example(serde_json::json!({"en": "North Carolina"}).into()),
                    ),
            )
            .property(
                "traits",
                ObjectBuilder::new()
                    .property(
                        "is_anonymous_proxy",
                        ObjectBuilder::new().schema_type(utoipa::openapi::SchemaType::Boolean),
                    )
                    .property(
                        "is_anycast",
                        ObjectBuilder::new().schema_type(utoipa::openapi::SchemaType::Boolean),
                    )
                    .property(
                        "is_satellite_provider",
                        ObjectBuilder::new().schema_type(utoipa::openapi::SchemaType::Boolean),
                    ),
            )
    }
}
