use actix_web::{get, HttpResponse, Responder};

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Ok", body = HealthCheckModel, content_type = "text/plain")
    ),
)]
#[get("/health")]
pub async fn handle() -> impl Responder {
    HttpResponse::Ok().content_type("text/plain").body("Ok")
}
