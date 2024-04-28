use actix_web::{test, App};

#[actix_web::test]
async fn test_lookup_endpoint() {
    let app_data = atlas_rs::init_db("tests-data/", "GeoIP2-City-Test").await;
    let app = test::init_service(
        App::new()
            .app_data(app_data.clone())
            .service(atlas_rs::services::lookup::handle),
    )
    .await;
    let req = test::TestRequest::get()
        .uri("/geoip/lookup/city/214.78.120.1")
        .to_request();
    let resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;
    let db = app_data.db.read().await;
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
