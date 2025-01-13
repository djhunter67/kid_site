use std::fmt::{self, Display, Formatter};

use actix_web::{post, web, Error, HttpResponse};
use mongodb::Database;
use serde::Deserialize;
use tracing::{info, instrument};

use crate::models::mongo::MongoRepo;

/// All things regarding grade, teachers, classes, and pictures

#[post("/grade")]
#[instrument(
    name = "Adrian",
    level = "info",
    target = "kid_data",
    skip(data, client)
)]
pub async fn grades_update(
    data: web::Json<Grade>,
    client: web::Data<Database>,
) -> Result<HttpResponse, Error> {
    let conn = MongoRepo::new(&client.as_ref().to_owned(), Some("Grades"));

    let Ok(grade) = serde_json::from_str::<Grade>(&data.to_string()) else {
        return Ok(HttpResponse::BadRequest().into());
    };

    let res = web::block(move || grade.save(&conn)).await;

    match res {
        Ok(()) => Ok(HttpResponse::Ok().json("Grade added")),
        Err(_) => Ok(HttpResponse::InternalServerError().into()),
    }
}

#[derive(Deserialize)]
pub struct Grade {
    pub school_level: String,
    pub teacher: String,
    pub class: String,
    pub picture: String,
}

impl Grade {
    pub fn save(&self, _conn: &MongoRepo) {
        info!("The grade: {}", self.school_level);
    }
}

impl Display for Grade {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "Grade: {}, Teacher: {}, Class: {}, Picture: {}",
            self.school_level, self.teacher, self.class, self.picture
        )
    }
}
