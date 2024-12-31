use actix_web::{get, web, Error, HttpResponse};
use askama::Template;
use mongodb::Database;
use tracing::instrument;

use crate::endpoints::{adrian::school::Grade, templates::CorbinLanding};

/// All things regarding grade, teachers, classes, and pictures

#[get("/corbin")]
#[instrument(name = "Corbin", level = "info", target = "aj_studying", skip(_client))]
pub async fn corbin(
    // data: web::Json<Grade>,
    _client: web::Data<Database>,
) -> Result<HttpResponse, Error> {
    // let conn = MongoRepo::new(&client.as_ref().to_owned());

    // let Ok(grade) = serde_json::from_str::<Grade>(&data.to_string()) else {
    //     return Ok(HttpResponse::BadRequest().into());
    // };

    // let res = web::block(move || grade.save(&conn)).await;

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
