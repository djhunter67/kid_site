use actix_session::Session;
use actix_web::{get, web, Error, HttpResponse};
use askama::Template;
use mongodb::Database;
use tracing::instrument;

use crate::endpoints::{adrian::school::Grade, login::validate_session, templates::CorbinLanding};

/// All things regarding grade, teachers, classes, and pictures
#[allow(clippy::future_not_send)]
#[get("/corbin")]
#[instrument(
    name = "Corbin",
    level = "info",
    target = "kid_data",
    skip(_client, session)
)]
pub async fn corbin(
    // data: web::Json<Grade>,
    _client: web::Data<Database>,
    session: Session,
) -> Result<HttpResponse, Error> {
    if let Some(http_resp) = validate_session(session) {
        return Ok(http_resp);
    }

    let grade = Grade {
        school_level: String::from("Day Care"),
        teacher: String::new(),
        class: String::new(),
        picture: String::new(),
    };

    let return_template = CorbinLanding {
        title: String::from("C.J."),
        name: String::from("Corbin J. Hunter"),
        age: 0,
        grade,
    }
    .render()
    .expect("Failed to render template");

    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(return_template))
}
