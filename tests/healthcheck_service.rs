use actix_web::http::StatusCode;
use actix_web::{App, body::MessageBody, test, web};

#[actix_web::test]
async fn test_healthcheck_endpoint() {
    let app = test::init_service(App::new().service(atlas_rs::services::healthcheck::handle)).await;
    let req = test::TestRequest::get().uri("/health").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = resp.into_body().try_into_bytes().unwrap();
    assert_eq!(body, web::Bytes::from_static(b"Ok"));
}

async fn status_of(swagger_ui_enabled: bool, uri: &str) -> StatusCode {
    let app =
        test::init_service(App::new().configure(atlas_rs::configure_app(swagger_ui_enabled))).await;
    let req = test::TestRequest::get().uri(uri).to_request();
    test::call_service(&app, req).await.status()
}

#[actix_web::test]
async fn test_app_serves_healthcheck() {
    assert_eq!(status_of(false, "/health").await, StatusCode::OK);
}

#[actix_web::test]
async fn test_swagger_ui_disabled() {
    assert_eq!(
        status_of(false, "/swagger-ui/").await,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        status_of(false, "/api-docs/openapi.json").await,
        StatusCode::NOT_FOUND
    );
}

#[actix_web::test]
async fn test_swagger_ui_enabled() {
    assert_eq!(status_of(true, "/swagger-ui/").await, StatusCode::OK);
    assert!(status_of(true, "/swagger-ui").await.is_redirection());

    let app = test::init_service(App::new().configure(atlas_rs::configure_app(true))).await;
    let req = test::TestRequest::get()
        .uri("/api-docs/openapi.json")
        .to_request();
    let spec: serde_json::Value = test::call_and_read_body_json(&app, req).await;
    assert!(
        spec["paths"]
            .get("/geoip/lookup/{lookup_type}/{ip_addresses}")
            .is_some()
    );
    assert!(spec["paths"].get("/health").is_some());
}
