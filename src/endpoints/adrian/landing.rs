use actix_web::{post, web, Error, HttpResponse};
use mongodb::Database;
use tracing::instrument;

use crate::{endpoints::adrian::school::Grade, models::mongo::MongoRepo};

/// All things regarding grade, teachers, classes, and pictures

#[post("/grade")]
#[instrument(
    name = "Adrian",
    level = "info",
    target = "aj_studying",
    skip(data, client)
)]
pub async fn grades(
    data: web::Json<Grade>,
    client: web::Data<Database>,
) -> Result<HttpResponse, Error> {
    let conn = MongoRepo::new(&client.as_ref().to_owned());

    let Ok(grade) = serde_json::from_str::<Grade>(&data.to_string()) else {
        return Ok(HttpResponse::BadRequest().into());
    };

    let res = web::block(move || grade.save(&conn)).await;

    match res {
        Ok(()) => Ok(HttpResponse::Ok().json("Grade added")),
        Err(_) => Ok(HttpResponse::InternalServerError().into()),
    }
}
