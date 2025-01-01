use actix_session::SessionExt;
use actix_web::{get, web, Error, HttpRequest, HttpResponse};
use askama::Template;
use mongodb::Database;
use tracing::{instrument, warn};

use crate::endpoints::{adrian::school::Grade, templates::AdrianLanding};

/// All things regarding grade, teachers, classes, and pictures

#[get("/adrian")]
#[instrument(
    name = "Adrian",
    level = "info",
    target = "kid_data",
    skip(_client, req)
)]
pub async fn adrian(
    req: HttpRequest,
    // data: web::Json<Grade>,
    _client: web::Data<Database>,
) -> Result<HttpResponse, Error> {
    // let conn = MongoRepo::new(&client.as_ref().to_owned());

    // let Ok(grade) = serde_json::from_str::<Grade>(&data.to_string()) else {
    //     return Ok(HttpResponse::BadRequest().into());
    // };

    // let res = web::block(move || grade.save(&conn)).await;

    let session = req.get_session();

    warn!("Session Entries: {:#?}", session.entries());

    // if let Ok(username) = session.get::<String>("id") {
    // warn!("Cookie: {username:#?}");
    // }

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
