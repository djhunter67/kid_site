use actix_session::Session;
use actix_web::{get, web, Error, HttpResponse};
use askama::Template;
use mongodb::Database;
use tracing::{instrument, warn};

use crate::endpoints::{adrian::school::Grade, login::validate_session, templates::AdrianLanding};

/// All things regarding grade, teachers, classes, and pictures
#[allow(clippy::future_not_send)]
#[get("/adrian")]
#[instrument(
    name = "Adrian",
    level = "info",
    target = "kid_data",
    skip(_client, session)
)]
pub async fn adrian(session: Session, _client: web::Data<Database>) -> Result<HttpResponse, Error> {
    // let pool = MongoRepo::new(&client.as_ref().to_owned());

    warn!(
        "Adrian endpont Session entries: {}",
        session.entries().is_empty()
    );

    if let Some(http_resp) = validate_session(session) {
        return Ok(http_resp);
    }

    let grade = Grade {
        school_level: String::from("elementary"),
        teacher: String::from("Kim Cates"),
        class: String::new(),
        picture: String::new(),
    };

    let return_template = AdrianLanding {
        title: String::from("A.J."),
        name: String::from("Adrian J. Hunter"),
        age: 7,
        grade,
    }
    .render()
    .expect("Failed to render template");

    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(return_template))
}
