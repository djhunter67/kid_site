use actix_web::{get, http::StatusCode, HttpResponse, Responder};
use askama::Template;
use tracing::{info, instrument, warn};

use super::templates::ErrorPage;

#[get("/health_check")]
#[instrument(name = "Health check", level = "info")]
pub async fn health_check() -> impl Responder {
    info!("Health check endpoint called.");
    HttpResponse::Ok().json("I'm alive!")
}

/// # Result
///   - `HttpResponse` with status code 404 and error message.
/// # Errors
///   - None
/// # Panics
///   - Failing to render error page.
#[must_use]
#[instrument(
    name = "Render error",
    level = "info",
    skip(message, err),
    fields(
	status_code = %status.as_u16(),
	status = %status.as_str(),
    )
)]
pub fn render_error<'a>(
    status: StatusCode,
    message: &'a str,
    err: Option<&'a str>,
) -> HttpResponse {
    info!("Rendering error page.");
    let error = ErrorPage {
        title: status.as_str(),
        code: status.as_u16(),
        error: err.unwrap_or(""),
        message,
    };

    warn!("Error Page error: {error}");

    let error_template = error.render().expect("Failed to render error page.");

    HttpResponse::build(status)
        .content_type("text/html; charset=utf-8")
        .body(error_template)
}
