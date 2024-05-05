use actix_web::HttpResponse;
use serde::Serialize;

pub mod healthcheck;
pub mod lookup;

#[derive(Serialize)]
struct Error {
    message: String,
    code: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: Error,
}

pub fn bad_request(message: String, code: String) -> HttpResponse {
    HttpResponse::BadRequest().json(ErrorResponse {
        error: Error { message, code },
    })
}
