use actix_web::{get, HttpResponse};
use askama::Template;
use tracing::{debug, error, info, instrument};

use crate::endpoints::templates::Index;

#[get("/main")]
#[instrument(name = "Main page", level = "debug")]
pub async fn index() -> HttpResponse {
    info!("Rendering the index page");
    let template = Index { title: "Quiz site" };

    debug!("rendering the main page");
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
