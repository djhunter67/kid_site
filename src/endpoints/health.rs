use actix_web::{get, HttpResponse, Responder};

#[tracing::instrument]
#[get("/health_check")]
pub async fn health_check() -> impl Responder {
    tracing::event!(target: "aj_studying", tracing::Level::DEBUG, "Health check endpoint called.");
    HttpResponse::Ok().json("I'm alive!")
}

