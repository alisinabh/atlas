use actix_http::body::MessageBody;
use actix_web::dev::{Service, ServiceResponse};
use actix_web::{test, web::Data, App};
use atlas_rs::maxmind_db::MaxmindDB;

struct SetupResult<T> {
    service: T,
    app_data: Data<MaxmindDB>,
}

async fn setup() -> SetupResult<
    impl Service<
        actix_http::Request,
        Error = actix_web::Error,
        Response = ServiceResponse<impl MessageBody>,
    >,
> {
    let app_data = atlas_rs::init_db("tests-data/", "GeoIP2-City-Test")
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

#[actix_web::test]
async fn test_lookup_endpoint() {
    let setup = setup().await;
    let req = test::TestRequest::get()
        .uri("/geoip/lookup/city/214.78.120.1")
        .to_request();
    let resp: serde_json::Value = test::call_and_read_body_json(&setup.service, req).await;
    let db = setup.app_data.db.read().await;
    assert_eq!(resp["database_build_epoch"], db.build_epoch());
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
async fn test_multiple_ips() {
    let setup = setup().await;
    let req = test::TestRequest::get()
        .uri("/geoip/lookup/city/214.78.120.1,214.78.120.2,4.2.2.4")
        .to_request();

    let resp: serde_json::Value = test::call_and_read_body_json(&setup.service, req).await;

    assert!(resp["results"].get("214.78.120.1").is_some());
    assert!(resp["results"].get("214.78.120.2").is_some());
    assert!(resp["results"].get("4.2.2.4").is_some());
}

#[actix_web::test]
async fn test_non_existing_ip() {
    let setup = setup().await;
    let req = test::TestRequest::get()
        .uri("/geoip/lookup/city/1.1.1.1")
        .to_request();

    let resp: serde_json::Value = test::call_and_read_body_json(&setup.service, req).await;

    assert!(resp["results"].get("1.1.1.1").unwrap().is_null());
}

#[actix_web::test]
async fn test_special_ip() {
    let setup = setup().await;
    let req = test::TestRequest::get()
        .uri("/geoip/lookup/city/192.168.1.1")
        .to_request();

    let resp: serde_json::Value = test::call_and_read_body_json(&setup.service, req).await;

    assert_eq!(
        resp["error"]["code"].as_str().unwrap(),
        "SPECIAL_IP".to_string()
    );
}

#[actix_web::test]
async fn test_invalid_ip() {
    let setup = setup().await;
    let req = test::TestRequest::get()
        .uri("/geoip/lookup/city/192.168.1.")
        .to_request();

    let resp: serde_json::Value = test::call_and_read_body_json(&setup.service, req).await;

    assert_eq!(
        resp["error"]["code"].as_str().unwrap(),
        "INVALID_IP".to_string()
    );
}

#[actix_web::test]
async fn test_rejects_too_many_ips() {
    let setup = setup().await;

    let ip_addresses = (1..=51)
        .map(|i| format!("1.1.1.{i}"))
        .collect::<Vec<_>>()
        .join(",");

    let req = test::TestRequest::get()
        .uri(format!("/geoip/lookup/city/{ip_addresses}").as_ref())
        .to_request();

    let resp: serde_json::Value = test::call_and_read_body_json(&setup.service, req).await;

    assert_eq!(
        resp["error"]["code"].as_str().unwrap(),
        "TOO_MANY_IPS".to_string()
    );
}
