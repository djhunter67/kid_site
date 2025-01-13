use actix_web::{http::StatusCode, HttpResponse};
use askama::Template;
use tracing::{info, instrument};

use crate::endpoints::templates::ErrorPage;

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

    // warn!("Error Page error: {error}");

    let error_template = error.render().expect("Failed to render error page.");

    HttpResponse::NotFound()
        .content_type("text/html; charset=utf-8")
        .body(error_template)
}
