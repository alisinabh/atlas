use actix_web::{body::MessageBody, test, web, App};

#[actix_web::test]
async fn test_healthcheck_endpoint() {
    let app = test::init_service(App::new().service(atlas_rs::services::healthcheck::handle)).await;
    let req = test::TestRequest::get().uri("/health").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = resp.into_body().try_into_bytes().unwrap();
    assert_eq!(body, web::Bytes::from_static(b"Ok"));
}
