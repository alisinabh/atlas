use crate::models::{HealthCheckModel, LookupResponseModel, LookupResult};
use crate::services;
use serde_json::json;
use utoipa::openapi::{schema::AdditionalProperties, Array, ObjectBuilder};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Atlas GeoIP",
        description = "Atlas GeoIP Service API Documentation [Github Repo](https://github.com/alisinabh/atlas-rs)"
    ),
    paths(services::healthcheck::handle, services::lookup::handle),
    components(schemas(LookupResponseModel, LookupResult, HealthCheckModel)),
    tags(
        (name = "GeoIP", description = "IP GeoLocation Endpoints"),
        (name = "Health", description = "Healthcheck Endpoints")
    )
)]
struct ApiDoc;

pub fn api_doc() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

// API Documentation for structs

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
                            .item(LookupResult::enterprise_schema())
                            .item(LookupResult::anonymous_ip_schema())
                            .item(LookupResult::asn_schema())
                            .item(LookupResult::connection_type_schema())
                            .item(LookupResult::country_schema())
                            .item(LookupResult::density_income_schema())
                            .item(LookupResult::isp_schema())
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

    fn anonymous_ip_schema() -> ObjectBuilder {
        let mut obj = ObjectBuilder::new()
            .schema_type(utoipa::openapi::SchemaType::Object)
            .title(Some("AnonymousIpLookupResult"));

        const BOOLEAN_FIELDS: &'static [&'static str] = &[
            "is_anonymous",
            "is_anonymous_vpn",
            "is_hosting_provider",
            "is_public_proxy",
            "is_residential_proxy",
            "is_tor_exit_node",
        ];

        for key in BOOLEAN_FIELDS {
            obj = obj.property(
                key.to_owned(),
                ObjectBuilder::new()
                    .schema_type(utoipa::openapi::SchemaType::Boolean)
                    .example(Some(serde_json::Value::Bool(false))),
            )
        }

        obj
    }

    fn asn_schema() -> ObjectBuilder {
        ObjectBuilder::new()
            .title(Some("AsnLookupResult"))
            .schema_type(utoipa::openapi::SchemaType::Object)
            .property(
                "autonomous_system_number",
                ObjectBuilder::new()
                    .schema_type(utoipa::openapi::SchemaType::Integer)
                    .example(Some(serde_json::json!(13335))),
            )
            .property(
                "autonomous_system_organization",
                ObjectBuilder::new()
                    .schema_type(utoipa::openapi::SchemaType::String)
                    .example(Some("Cloudflare".into())),
            )
    }

    fn connection_type_schema() -> ObjectBuilder {
        ObjectBuilder::new()
            .title(Some("ConnectionTypeLookupResult"))
            .schema_type(utoipa::openapi::SchemaType::Object)
            .property(
                "connection_type",
                ObjectBuilder::new()
                    .schema_type(utoipa::openapi::SchemaType::String)
                    .example(Some("LTE".into())),
            )
    }

    fn country_schema() -> ObjectBuilder {
        ObjectBuilder::new()
            .title(Some("CountryLookupResult"))
            .schema_type(utoipa::openapi::SchemaType::Object)
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
                            .schema_type(utoipa::openapi::SchemaType::Integer)
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
                            .schema_type(utoipa::openapi::SchemaType::Integer)
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
                Array::new(
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

    fn enterprise_schema() -> ObjectBuilder {
        ObjectBuilder::new()
            .title(Some("EnterpriseLookupResult"))
            .schema_type(utoipa::openapi::SchemaType::Object)
            .property(
                "city",
                ObjectBuilder::new()
                    .property(
                        "confidence",
                        ObjectBuilder::new().schema_type(utoipa::openapi::SchemaType::Integer),
                    )
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
                    .property(
                        "confidence",
                        ObjectBuilder::new().schema_type(utoipa::openapi::SchemaType::Integer),
                    )
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
                            .schema_type(utoipa::openapi::SchemaType::Integer)
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
                            .schema_type(utoipa::openapi::SchemaType::Integer)
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
                ObjectBuilder::new()
                    .property(
                        "confidence",
                        ObjectBuilder::new().schema_type(utoipa::openapi::SchemaType::Integer),
                    )
                    .property(
                        "code",
                        ObjectBuilder::new()
                            .schema_type(utoipa::openapi::SchemaType::String)
                            .example(Some("27127".into())),
                    ),
            )
            .property(
                "registered_country",
                ObjectBuilder::new()
                    .property(
                        "confidence",
                        ObjectBuilder::new().schema_type(utoipa::openapi::SchemaType::Integer),
                    )
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
                    .property(
                        "confidence",
                        ObjectBuilder::new().schema_type(utoipa::openapi::SchemaType::Integer),
                    )
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
                Array::new(
                    ObjectBuilder::new()
                        .property(
                            "confidence",
                            ObjectBuilder::new().schema_type(utoipa::openapi::SchemaType::Integer),
                        )
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

    fn density_income_schema() -> ObjectBuilder {
        ObjectBuilder::new()
            .schema_type(utoipa::openapi::SchemaType::Object)
            .title(Some("DenityIncomeLookupResult"))
            .property(
                "average_income",
                ObjectBuilder::new().schema_type(utoipa::openapi::SchemaType::Integer),
            )
            .property(
                "population_density",
                ObjectBuilder::new().schema_type(utoipa::openapi::SchemaType::Integer),
            )
    }

    fn isp_schema() -> ObjectBuilder {
        ObjectBuilder::new()
            .schema_type(utoipa::openapi::SchemaType::Object)
            .title(Some("IspLookupResult"))
            .property(
                "autonomous_system_number",
                ObjectBuilder::new()
                    .schema_type(utoipa::openapi::SchemaType::Integer)
                    .example(Some(serde_json::json!(13335))),
            )
            .property(
                "autonomous_system_organization",
                ObjectBuilder::new()
                    .schema_type(utoipa::openapi::SchemaType::String)
                    .example(Some("Cloudflare".into())),
            )
            .property(
                "isp",
                ObjectBuilder::new()
                    .schema_type(utoipa::openapi::SchemaType::String)
                    .example(Some(json!("Rogers"))),
            )
            .property(
                "mobile_country_code",
                ObjectBuilder::new()
                    .schema_type(utoipa::openapi::SchemaType::String)
                    .example(Some(json!("+98"))),
            )
            .property(
                "mobile_network_code",
                ObjectBuilder::new()
                    .schema_type(utoipa::openapi::SchemaType::String)
                    .example(Some(json!("C101"))),
            )
            .property(
                "organization",
                ObjectBuilder::new()
                    .schema_type(utoipa::openapi::SchemaType::String)
                    .example(Some(json!("ACME"))),
            )
    }
}
