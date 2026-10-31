mod common;

use actix_http::body::MessageBody;
use actix_web::dev::{Service, ServiceResponse};
use actix_web::http::{StatusCode, header};
use actix_web::{App, test, web::Data};
use atlas_rs::config::DownloadConfig;
use atlas_rs::maxmind_db::MaxmindDB;
use serde_json::Value;

struct SetupResult<T> {
    service: T,
    app_data: Data<MaxmindDB>,
}

async fn setup_variant(
    variant: &str,
) -> SetupResult<
    impl Service<
        actix_http::Request,
        Error = actix_web::Error,
        Response = ServiceResponse<impl MessageBody>,
    >,
> {
    let app_data = atlas_rs::init_db("tests-data/", variant, DownloadConfig::default())
        .await
        .unwrap();
    let service = test::init_service(
        App::new()
            .app_data(app_data.clone())
            .service(atlas_rs::services::lookup::handle),
    )
    .await;

    SetupResult { service, app_data }
}

async fn setup() -> SetupResult<
    impl Service<
        actix_http::Request,
        Error = actix_web::Error,
        Response = ServiceResponse<impl MessageBody>,
    >,
> {
    setup_variant(common::CITY_VARIANT).await
}

/// Performs a lookup and returns the status code and the JSON body.
async fn lookup<S, B>(service: &S, uri: &str) -> (StatusCode, Value)
where
    S: Service<actix_http::Request, Response = ServiceResponse<B>, Error = actix_web::Error>,
    B: MessageBody,
{
    let req = test::TestRequest::get().uri(uri).to_request();
    let resp = test::call_service(service, req).await;
    let status = resp.status();
    let body = test::read_body(resp).await;
    (status, serde_json::from_slice(&body).unwrap())
}

async fn assert_error_code(uri: &str, expected_code: &str) {
    let setup = setup().await;
    let (status, resp) = lookup(&setup.service, uri).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{uri}: {resp}");
    assert_eq!(resp["error"]["code"], expected_code, "{uri}: {resp}");
}

#[actix_web::test]
async fn test_lookup_endpoint() {
    let setup = setup().await;
    let (status, resp) = lookup(&setup.service, "/geoip/lookup/city/214.78.120.1").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        resp["database_build_epoch"],
        setup.app_data.current().build_epoch()
    );
    assert_eq!(
        resp["results"]["214.78.120.1"]["city"]["geoname_id"],
        5391811
    );
    assert_eq!(
        resp["results"]["214.78.120.1"]["city"]["names"]["en"],
        "San Diego"
    );
}

#[actix_web::test]
async fn test_lookup_sets_cache_control() {
    let setup = setup().await;
    let req = test::TestRequest::get()
        .uri("/geoip/lookup/city/214.78.120.1")
        .to_request();
    let resp = test::call_service(&setup.service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let cache_control = resp.headers().get(header::CACHE_CONTROL).unwrap();
    assert!(cache_control.to_str().unwrap().contains("max-age="));
}

#[actix_web::test]
async fn test_multiple_ips() {
    let setup = setup().await;
    let (status, resp) = lookup(
        &setup.service,
        "/geoip/lookup/city/214.78.120.1,214.78.120.2,4.2.2.4",
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert!(resp["results"].get("214.78.120.1").is_some());
    assert!(resp["results"].get("214.78.120.2").is_some());
    assert!(resp["results"].get("4.2.2.4").is_some());
}

#[actix_web::test]
async fn test_whitespace_around_ips_is_trimmed() {
    let setup = setup().await;
    let (status, resp) = lookup(
        &setup.service,
        "/geoip/lookup/city/%20214.78.120.1%20,%204.2.2.4",
    )
    .await;

    assert_eq!(status, StatusCode::OK, "{resp}");
    assert!(resp["results"].get("214.78.120.1").is_some());
    assert!(resp["results"].get("4.2.2.4").is_some());
}

#[actix_web::test]
async fn test_duplicate_ips_are_merged() {
    let setup = setup().await;
    let (status, resp) = lookup(
        &setup.service,
        "/geoip/lookup/city/214.78.120.1,214.78.120.1",
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["results"].as_object().unwrap().len(), 1);
}

#[actix_web::test]
async fn test_non_existing_ip() {
    let setup = setup().await;
    let (status, resp) = lookup(&setup.service, "/geoip/lookup/city/1.1.1.1").await;

    assert_eq!(status, StatusCode::OK);
    assert!(resp["results"].get("1.1.1.1").unwrap().is_null());
}

#[actix_web::test]
async fn test_ipv6_lookup() {
    let setup = setup_variant("GeoIP2-Country-Test").await;
    let (status, resp) = lookup(&setup.service, "/geoip/lookup/country/2001:218::1").await;

    assert_eq!(status, StatusCode::OK, "{resp}");
    assert_eq!(resp["results"]["2001:218::1"]["country"]["iso_code"], "JP");
}

#[actix_web::test]
async fn test_ipv4_mapped_public_ip_is_allowed() {
    let setup = setup().await;
    let (status, resp) = lookup(&setup.service, "/geoip/lookup/city/::ffff:214.78.120.1").await;

    assert_eq!(status, StatusCode::OK, "{resp}");
    assert!(resp["results"].get("::ffff:214.78.120.1").is_some());
}

#[actix_web::test]
async fn test_special_ip() {
    assert_error_code("/geoip/lookup/city/192.168.1.1", "SPECIAL_IP").await;
}

#[actix_web::test]
async fn test_special_ips_are_rejected() {
    for ip in [
        "127.0.0.1",
        "10.0.0.1",
        "169.254.1.1",
        "100.64.0.1",
        "0.1.2.3",
        "240.0.0.1",
        "255.255.255.255",
        "224.0.0.1",
        "::1",
        "::",
        "fe80::1",
        "fc00::1",
        "ff02::1",
        "::ffff:192.168.1.1",
        "::ffff:127.0.0.1",
    ] {
        assert_error_code(&format!("/geoip/lookup/city/{ip}"), "SPECIAL_IP").await;
    }
}

#[actix_web::test]
async fn test_special_ip_in_list_rejects_whole_request() {
    assert_error_code("/geoip/lookup/city/214.78.120.1,10.0.0.1", "SPECIAL_IP").await;
}

#[actix_web::test]
async fn test_invalid_ip() {
    assert_error_code("/geoip/lookup/city/192.168.1.", "INVALID_IP").await;
}

#[actix_web::test]
async fn test_trailing_comma_is_invalid() {
    assert_error_code("/geoip/lookup/city/214.78.120.1,", "INVALID_IP").await;
}

#[actix_web::test]
async fn test_invalid_lookup_type() {
    assert_error_code("/geoip/lookup/nope/214.78.120.1", "INVALID_LOOKUP_TYPE").await;
}

#[actix_web::test]
async fn test_accepts_max_ips() {
    let setup = setup().await;
    let ip_addresses = (1..=50)
        .map(|i| format!("1.1.1.{i}"))
        .collect::<Vec<_>>()
        .join(",");

    let (status, resp) = lookup(
        &setup.service,
        &format!("/geoip/lookup/city/{ip_addresses}"),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["results"].as_object().unwrap().len(), 50);
}

#[actix_web::test]
async fn test_rejects_too_many_ips() {
    let ip_addresses = (1..=51)
        .map(|i| format!("1.1.1.{i}"))
        .collect::<Vec<_>>()
        .join(",");

    assert_error_code(
        &format!("/geoip/lookup/city/{ip_addresses}"),
        "TOO_MANY_IPS",
    )
    .await;
}

#[actix_web::test]
async fn test_rejects_too_many_ips_even_if_invalid() {
    let ip_addresses = vec!["x"; 1000].join(",");

    assert_error_code(
        &format!("/geoip/lookup/city/{ip_addresses}"),
        "TOO_MANY_IPS",
    )
    .await;
}

/// (variant, lookup_type, ip, JSON pointer into the result, expected value)
const LOOKUP_TYPE_CASES: &[(&str, &str, &str, &str, &str)] = &[
    (
        "GeoIP2-Anonymous-IP-Test",
        "anonymous_ip",
        "81.2.69.1",
        "/is_tor_exit_node",
        "true",
    ),
    (
        "GeoLite2-ASN-Test",
        "asn",
        "1.128.0.1",
        "/autonomous_system_number",
        "1221",
    ),
    (
        "GeoIP2-City-Test",
        "city",
        "214.78.120.1",
        "/city/names/en",
        "\"San Diego\"",
    ),
    (
        "GeoIP2-Connection-Type-Test",
        "connection_type",
        "1.0.1.1",
        "/connection_type",
        "\"Cellular\"",
    ),
    (
        "GeoIP2-Country-Test",
        "country",
        "2001:218::1",
        "/country/iso_code",
        "\"JP\"",
    ),
    (
        "GeoIP2-DensityIncome-Test",
        "density_income",
        "5.83.124.1",
        "/average_income",
        "32323",
    ),
    (
        "GeoIP2-Enterprise-Test",
        "enterprise",
        "74.209.24.1",
        "/city/names/en",
        "\"Chatham\"",
    ),
    (
        "GeoIP2-ISP-Test",
        "isp",
        "4.0.0.1",
        "/isp",
        "\"Level 3 Communications\"",
    ),
];

#[actix_web::test]
async fn test_all_lookup_types() {
    for &(variant, lookup_type, ip, pointer, expected) in LOOKUP_TYPE_CASES {
        let setup = setup_variant(variant).await;
        let (status, resp) =
            lookup(&setup.service, &format!("/geoip/lookup/{lookup_type}/{ip}")).await;

        assert_eq!(status, StatusCode::OK, "{lookup_type}: {resp}");
        let result = &resp["results"][ip];
        let expected: Value = serde_json::from_str(expected).unwrap();
        assert_eq!(
            result.pointer(pointer),
            Some(&expected),
            "{lookup_type}: {result}"
        );
    }
}

#[actix_web::test]
async fn test_lookup_type_not_in_db_returns_null() {
    let setup = setup().await;
    let (status, resp) = lookup(&setup.service, "/geoip/lookup/isp/214.78.120.1").await;

    assert_eq!(status, StatusCode::OK);
    assert!(resp["results"]["214.78.120.1"].is_null(), "{resp}");
}
