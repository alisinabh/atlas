use actix_web::HttpResponse;
use serde::Serialize;

pub mod healthcheck;
pub mod lookup;

#[derive(Serialize)]
struct JsonError {
    message: String,
}

pub fn bad_request(message: String) -> HttpResponse {
    HttpResponse::BadRequest().json(JsonError { message })
}
