use actix_web::{get, HttpResponse, Responder};
use log::info;

#[get("/health_check")]
pub async fn health_check() -> impl Responder {
    info!("Health check endpoint called.");
    HttpResponse::Ok().json("I'm alive!")
}
