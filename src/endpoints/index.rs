use actix_web::{get, HttpResponse};
use askama::Template;
use tracing::error;

use crate::endpoints::templates::Index;

#[get("/main")]
pub async fn index() -> HttpResponse {
    let template = Index { title: "AJ Quiz" };

    let body = match template.render() {
        Ok(body) => body,
        Err(err) => {
            error!("Error rendering template: {err:#?}");
            return HttpResponse::InternalServerError().finish();
        }
    };

    HttpResponse::Ok()
        .content_type("text/html")
        .append_header(("Authorization", "Bearer token"))
        .body(body)
}
