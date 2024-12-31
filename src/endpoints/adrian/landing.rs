use actix_web::{get, web, Error, HttpResponse};
use askama::Template;
use mongodb::Database;
use tracing::instrument;

use crate::endpoints::{adrian::school::Grade, templates::AdrianLanding};

/// All things regarding grade, teachers, classes, and pictures

#[get("/adrian")]
#[instrument(name = "Adrian", level = "info", target = "kid_data", skip(_client))]
pub async fn adrian(
    // data: web::Json<Grade>,
    _client: web::Data<Database>,
) -> Result<HttpResponse, Error> {
    // let conn = MongoRepo::new(&client.as_ref().to_owned());

    // let Ok(grade) = serde_json::from_str::<Grade>(&data.to_string()) else {
    //     return Ok(HttpResponse::BadRequest().into());
    // };

    // let res = web::block(move || grade.save(&conn)).await;

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
