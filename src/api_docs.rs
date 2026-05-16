use crate::models::{HealthCheckModel, LookupResponseModel, LookupResult};
use crate::services;
use serde_json::json;
use std::borrow::Cow;
use utoipa::openapi::{
    Array, ObjectBuilder,
    schema::{AdditionalProperties, Type},
};
use utoipa::{OpenApi, PartialSchema};

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

impl PartialSchema for HealthCheckModel {
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        ObjectBuilder::new()
            .schema_type(Type::String)
            .examples(["Ok"])
            .into()
    }
}

impl utoipa::ToSchema for HealthCheckModel {
    fn name() -> Cow<'static, str> {
        Cow::Borrowed("HealthCheckModel")
    }
}

impl PartialSchema for LookupResult<'_> {
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        utoipa::openapi::ObjectBuilder::new()
            .additional_properties(Some(AdditionalProperties::RefOr(
                utoipa::openapi::RefOr::T(LookupResult::one_of_lookup_schema()),
            )))
            .into()
    }
}

impl utoipa::ToSchema for LookupResult<'_> {
    fn name() -> Cow<'static, str> {
        Cow::Borrowed("LookupResult")
    }
}

impl LookupResult<'_> {
    fn localized_string() -> ObjectBuilder {
        ObjectBuilder::new()
            .schema_type(Type::Object)
            .title(Some("LocalizedString"))
            .additional_properties(Some(AdditionalProperties::RefOr(
                utoipa::openapi::RefOr::T(utoipa::openapi::Schema::Object(
                    ObjectBuilder::new().schema_type(Type::String).into(),
                )),
            )))
    }

    fn geoname_id() -> ObjectBuilder {
        ObjectBuilder::new()
            .schema_type(Type::Integer)
            .examples([serde_json::Value::Number(1234.into())])
    }

    fn one_of_lookup_schema() -> utoipa::openapi::Schema {
        let builder = utoipa::openapi::schema::OneOfBuilder::new()
            .item(LookupResult::city_schema())
            .item(LookupResult::enterprise_schema())
            .item(LookupResult::anonymous_ip_schema())
            .item(LookupResult::asn_schema())
            .item(LookupResult::connection_type_schema())
            .item(LookupResult::country_schema())
            .item(LookupResult::density_income_schema())
            .item(LookupResult::isp_schema());

        utoipa::openapi::Schema::OneOf(builder.into())
    }

    fn anonymous_ip_schema() -> ObjectBuilder {
        let mut obj = ObjectBuilder::new()
            .schema_type(Type::Object)
            .title(Some("AnonymousIpLookupResult"));

        const BOOLEAN_FIELDS: &[&str] = &[
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
                    .schema_type(Type::Boolean)
                    .examples([serde_json::Value::Bool(false)]),
            )
        }

        obj
    }

    fn asn_schema() -> ObjectBuilder {
        ObjectBuilder::new()
            .title(Some("AsnLookupResult"))
            .schema_type(Type::Object)
            .property(
                "autonomous_system_number",
                ObjectBuilder::new()
                    .schema_type(Type::Integer)
                    .examples([serde_json::json!(13335)]),
            )
            .property(
                "autonomous_system_organization",
                ObjectBuilder::new()
                    .schema_type(Type::String)
                    .examples(["Cloudflare"]),
            )
    }

    fn connection_type_schema() -> ObjectBuilder {
        ObjectBuilder::new()
            .title(Some("ConnectionTypeLookupResult"))
            .schema_type(Type::Object)
            .property(
                "connection_type",
                ObjectBuilder::new()
                    .schema_type(Type::String)
                    .examples(["LTE"]),
            )
    }

    fn country_schema() -> ObjectBuilder {
        ObjectBuilder::new()
            .title(Some("CountryLookupResult"))
            .schema_type(Type::Object)
            .property(
                "continent",
                ObjectBuilder::new()
                    .property(
                        "code",
                        ObjectBuilder::new()
                            .schema_type(Type::String)
                            .examples(["NA"]),
                    )
                    .property("geoname_id", Self::geoname_id())
                    .property(
                        "names",
                        Self::localized_string()
                            .examples([serde_json::json!({"en": "North America"})]),
                    ),
            )
            .property(
                "country",
                ObjectBuilder::new()
                    .property("geoname_id", Self::geoname_id())
                    .property(
                        "is_in_european_union",
                        ObjectBuilder::new().schema_type(Type::Boolean),
                    )
                    .property(
                        "iso_code",
                        ObjectBuilder::new()
                            .schema_type(Type::String)
                            .examples(["US"]),
                    )
                    .property(
                        "names",
                        Self::localized_string()
                            .examples([serde_json::json!({"en": "United States"})]),
                    ),
            )
            .property(
                "registered_country",
                ObjectBuilder::new()
                    .property("geoname_id", Self::geoname_id())
                    .property(
                        "is_in_european_union",
                        ObjectBuilder::new().schema_type(Type::Boolean),
                    )
                    .property(
                        "iso_code",
                        ObjectBuilder::new()
                            .schema_type(Type::String)
                            .examples(["US"]),
                    )
                    .property(
                        "names",
                        Self::localized_string()
                            .examples([serde_json::json!({"en": "United States"})]),
                    ),
            )
            .property(
                "represented_country",
                ObjectBuilder::new()
                    .property("geoname_id", Self::geoname_id())
                    .property(
                        "is_in_european_union",
                        ObjectBuilder::new().schema_type(Type::Boolean),
                    )
                    .property(
                        "iso_code",
                        ObjectBuilder::new()
                            .schema_type(Type::String)
                            .examples(["US"]),
                    )
                    .property(
                        "names",
                        Self::localized_string()
                            .examples([serde_json::json!({"en": "United States"})]),
                    )
                    .property("type", ObjectBuilder::new().schema_type(Type::String)),
            )
            .property(
                "traits",
                ObjectBuilder::new()
                    .property(
                        "is_anonymous_proxy",
                        ObjectBuilder::new().schema_type(Type::Boolean),
                    )
                    .property(
                        "is_anycast",
                        ObjectBuilder::new().schema_type(Type::Boolean),
                    )
                    .property(
                        "is_satellite_provider",
                        ObjectBuilder::new().schema_type(Type::Boolean),
                    ),
            )
    }

    fn city_schema() -> ObjectBuilder {
        ObjectBuilder::new()
            .title(Some("CityLookupResult"))
            .schema_type(Type::Object)
            .property(
                "city",
                ObjectBuilder::new()
                    .property("geoname_id", Self::geoname_id())
                    .property(
                        "names",
                        Self::localized_string().examples([serde_json::json!({"en": "San Diego"})]),
                    ),
            )
            .property(
                "continent",
                ObjectBuilder::new()
                    .property(
                        "code",
                        ObjectBuilder::new()
                            .schema_type(Type::String)
                            .examples(["NA"]),
                    )
                    .property("geoname_id", Self::geoname_id())
                    .property(
                        "names",
                        Self::localized_string()
                            .examples([serde_json::json!({"en": "North America"})]),
                    ),
            )
            .property(
                "country",
                ObjectBuilder::new()
                    .property("geoname_id", Self::geoname_id())
                    .property(
                        "is_in_european_union",
                        ObjectBuilder::new().schema_type(Type::Boolean),
                    )
                    .property(
                        "iso_code",
                        ObjectBuilder::new()
                            .schema_type(Type::String)
                            .examples(["US"]),
                    )
                    .property(
                        "names",
                        Self::localized_string()
                            .examples([serde_json::json!({"en": "United States"})]),
                    ),
            )
            .property(
                "location",
                ObjectBuilder::new()
                    .property(
                        "accuracy_radius",
                        ObjectBuilder::new()
                            .schema_type(Type::Integer)
                            .examples([200]),
                    )
                    .property(
                        "latitude",
                        ObjectBuilder::new()
                            .schema_type(Type::Number)
                            .examples([(30.0406)]),
                    )
                    .property(
                        "longitude",
                        ObjectBuilder::new()
                            .schema_type(Type::Number)
                            .examples([(-80.26)]),
                    )
                    .property(
                        "metro_code",
                        ObjectBuilder::new()
                            .schema_type(Type::Integer)
                            .examples([518]),
                    )
                    .property(
                        "time_zone",
                        ObjectBuilder::new()
                            .schema_type(Type::String)
                            .examples(["America/New_York"]),
                    ),
            )
            .property(
                "postal",
                ObjectBuilder::new().property(
                    "code",
                    ObjectBuilder::new()
                        .schema_type(Type::String)
                        .examples(["27127"]),
                ),
            )
            .property(
                "registered_country",
                ObjectBuilder::new()
                    .property("geoname_id", Self::geoname_id())
                    .property(
                        "is_in_european_union",
                        ObjectBuilder::new().schema_type(Type::Boolean),
                    )
                    .property(
                        "iso_code",
                        ObjectBuilder::new()
                            .schema_type(Type::String)
                            .examples(["US"]),
                    )
                    .property(
                        "names",
                        Self::localized_string()
                            .examples([serde_json::json!({"en": "United States"})]),
                    ),
            )
            .property(
                "represented_country",
                ObjectBuilder::new()
                    .property("geoname_id", Self::geoname_id())
                    .property(
                        "is_in_european_union",
                        ObjectBuilder::new().schema_type(Type::Boolean),
                    )
                    .property(
                        "iso_code",
                        ObjectBuilder::new()
                            .schema_type(Type::String)
                            .examples(["US"]),
                    )
                    .property(
                        "names",
                        Self::localized_string()
                            .examples([serde_json::json!({"en": "United States"})]),
                    )
                    .property("type", ObjectBuilder::new().schema_type(Type::String)),
            )
            .property(
                "subdivisions",
                Array::new(
                    ObjectBuilder::new()
                        .property("geoname_id", Self::geoname_id())
                        .property(
                            "iso_code",
                            ObjectBuilder::new()
                                .schema_type(Type::String)
                                .examples(["NC"]),
                        )
                        .property(
                            "names",
                            Self::localized_string()
                                .examples([serde_json::json!({"en": "North Carolina"})]),
                        ),
                ),
            )
            .property(
                "traits",
                ObjectBuilder::new()
                    .property(
                        "is_anonymous_proxy",
                        ObjectBuilder::new().schema_type(Type::Boolean),
                    )
                    .property(
                        "is_anycast",
                        ObjectBuilder::new().schema_type(Type::Boolean),
                    )
                    .property(
                        "is_satellite_provider",
                        ObjectBuilder::new().schema_type(Type::Boolean),
                    ),
            )
    }

    fn enterprise_schema() -> ObjectBuilder {
        ObjectBuilder::new()
            .title(Some("EnterpriseLookupResult"))
            .schema_type(Type::Object)
            .property(
                "city",
                ObjectBuilder::new()
                    .property(
                        "confidence",
                        ObjectBuilder::new().schema_type(Type::Integer),
                    )
                    .property("geoname_id", Self::geoname_id())
                    .property(
                        "names",
                        Self::localized_string().examples([serde_json::json!({"en": "San Diego"})]),
                    ),
            )
            .property(
                "continent",
                ObjectBuilder::new()
                    .property(
                        "code",
                        ObjectBuilder::new()
                            .schema_type(Type::String)
                            .examples(["NA"]),
                    )
                    .property("geoname_id", Self::geoname_id())
                    .property(
                        "names",
                        Self::localized_string()
                            .examples([serde_json::json!({"en": "North America"})]),
                    ),
            )
            .property(
                "country",
                ObjectBuilder::new()
                    .property(
                        "confidence",
                        ObjectBuilder::new().schema_type(Type::Integer),
                    )
                    .property("geoname_id", Self::geoname_id())
                    .property(
                        "is_in_european_union",
                        ObjectBuilder::new().schema_type(Type::Boolean),
                    )
                    .property(
                        "iso_code",
                        ObjectBuilder::new()
                            .schema_type(Type::String)
                            .examples(["US"]),
                    )
                    .property(
                        "names",
                        Self::localized_string()
                            .examples([serde_json::json!({"en": "United States"})]),
                    ),
            )
            .property(
                "location",
                ObjectBuilder::new()
                    .property(
                        "accuracy_radius",
                        ObjectBuilder::new()
                            .schema_type(Type::Integer)
                            .examples([200]),
                    )
                    .property(
                        "latitude",
                        ObjectBuilder::new()
                            .schema_type(Type::Number)
                            .examples([(30.0406)]),
                    )
                    .property(
                        "longitude",
                        ObjectBuilder::new()
                            .schema_type(Type::Number)
                            .examples([(-80.26)]),
                    )
                    .property(
                        "metro_code",
                        ObjectBuilder::new()
                            .schema_type(Type::Integer)
                            .examples([518]),
                    )
                    .property(
                        "time_zone",
                        ObjectBuilder::new()
                            .schema_type(Type::String)
                            .examples(["America/New_York"]),
                    ),
            )
            .property(
                "postal",
                ObjectBuilder::new()
                    .property(
                        "confidence",
                        ObjectBuilder::new().schema_type(Type::Integer),
                    )
                    .property(
                        "code",
                        ObjectBuilder::new()
                            .schema_type(Type::String)
                            .examples(["27127"]),
                    ),
            )
            .property(
                "registered_country",
                ObjectBuilder::new()
                    .property(
                        "confidence",
                        ObjectBuilder::new().schema_type(Type::Integer),
                    )
                    .property("geoname_id", Self::geoname_id())
                    .property(
                        "is_in_european_union",
                        ObjectBuilder::new().schema_type(Type::Boolean),
                    )
                    .property(
                        "iso_code",
                        ObjectBuilder::new()
                            .schema_type(Type::String)
                            .examples(["US"]),
                    )
                    .property(
                        "names",
                        Self::localized_string()
                            .examples([serde_json::json!({"en": "United States"})]),
                    ),
            )
            .property(
                "represented_country",
                ObjectBuilder::new()
                    .property(
                        "confidence",
                        ObjectBuilder::new().schema_type(Type::Integer),
                    )
                    .property("geoname_id", Self::geoname_id())
                    .property(
                        "is_in_european_union",
                        ObjectBuilder::new().schema_type(Type::Boolean),
                    )
                    .property(
                        "iso_code",
                        ObjectBuilder::new()
                            .schema_type(Type::String)
                            .examples(["US"]),
                    )
                    .property(
                        "names",
                        Self::localized_string()
                            .examples([serde_json::json!({"en": "United States"})]),
                    )
                    .property("type", ObjectBuilder::new().schema_type(Type::String)),
            )
            .property(
                "subdivisions",
                Array::new(
                    ObjectBuilder::new()
                        .property(
                            "confidence",
                            ObjectBuilder::new().schema_type(Type::Integer),
                        )
                        .property("geoname_id", Self::geoname_id())
                        .property(
                            "iso_code",
                            ObjectBuilder::new()
                                .schema_type(Type::String)
                                .examples(["NC"]),
                        )
                        .property(
                            "names",
                            Self::localized_string()
                                .examples([serde_json::json!({"en": "North Carolina"})]),
                        ),
                ),
            )
            .property(
                "traits",
                ObjectBuilder::new()
                    .property(
                        "is_anonymous_proxy",
                        ObjectBuilder::new().schema_type(Type::Boolean),
                    )
                    .property(
                        "is_anycast",
                        ObjectBuilder::new().schema_type(Type::Boolean),
                    )
                    .property(
                        "is_satellite_provider",
                        ObjectBuilder::new().schema_type(Type::Boolean),
                    ),
            )
    }

    fn density_income_schema() -> ObjectBuilder {
        ObjectBuilder::new()
            .schema_type(Type::Object)
            .title(Some("DenityIncomeLookupResult"))
            .property(
                "average_income",
                ObjectBuilder::new().schema_type(Type::Integer),
            )
            .property(
                "population_density",
                ObjectBuilder::new().schema_type(Type::Integer),
            )
    }

    fn isp_schema() -> ObjectBuilder {
        ObjectBuilder::new()
            .schema_type(Type::Object)
            .title(Some("IspLookupResult"))
            .property(
                "autonomous_system_number",
                ObjectBuilder::new()
                    .schema_type(Type::Integer)
                    .examples([serde_json::json!(13335)]),
            )
            .property(
                "autonomous_system_organization",
                ObjectBuilder::new()
                    .schema_type(Type::String)
                    .examples(["Cloudflare"]),
            )
            .property(
                "isp",
                ObjectBuilder::new()
                    .schema_type(Type::String)
                    .examples([json!("Rogers")]),
            )
            .property(
                "mobile_country_code",
                ObjectBuilder::new()
                    .schema_type(Type::String)
                    .examples([json!("+98")]),
            )
            .property(
                "mobile_network_code",
                ObjectBuilder::new()
                    .schema_type(Type::String)
                    .examples([json!("C101")]),
            )
            .property(
                "organization",
                ObjectBuilder::new()
                    .schema_type(Type::String)
                    .examples([json!("ACME")]),
            )
    }
}
