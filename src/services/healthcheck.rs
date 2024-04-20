use actix_web::{get, HttpResponse, Responder};

#[get("/health")]
pub async fn handle() -> impl Responder {
    HttpResponse::Ok().body("Ok")
}
