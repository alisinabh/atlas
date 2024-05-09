use actix_web::{get, HttpResponse, Responder};

/// Returns 200 when GeoIP service is up and running
#[utoipa::path(
    get,
    path = "/health",
    operation_id = "healthcheck",
    tag = "Health",
    responses(
        (status = 200, description = "Ok", body = HealthCheckModel, content_type = "text/plain")
    ),
)]
#[get("/health")]
pub async fn handle() -> impl Responder {
    HttpResponse::Ok().content_type("text/plain").body("Ok")
}
