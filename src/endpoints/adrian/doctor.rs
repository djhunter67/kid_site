use actix_web::{get, http::header::Date, web, Error, HttpResponse};
use askama::Template;
use mongodb::Database;
use serde::Serialize;
use tracing::instrument;

use crate::{endpoints::templates::DoctorData, settings::Settings};

/// All things regarding grade, teachers, classes, and pictures

#[get("/doctor_data")]
#[instrument(
    name = "doctor data",
    level = "info",
    target = "kid_data",
    skip(_client, settings)
)]
pub async fn doctor_data(
    // data: web::Json<Grade>,
    _client: web::Data<Database>,
    settings: Settings,
) -> Result<HttpResponse, Error> {
    // let conn = MongoRepo::new(&client.as_ref().to_owned());

    // let Ok(grade) = serde_json::from_str::<Grade>(&data.to_string()) else {
    //     return Ok(HttpResponse::BadRequest().into());
    // };

    // let res = web::block(move || grade.save(&conn)).await;

    let grade = DoctorData {
        title: "Doctor Data",
        name: &settings.doctor.name,
        email: &settings.doctor.email,
        phone: &settings.doctor.phone,
        address: &settings.doctor.address,
        speciality: &settings.doctor.speciality,
        card_data: vec![
            DoctorCards {
                // Get the current date
                date: Date::now().to_string(),
                ..Default::default()
            },
            DoctorCards {
                date: Date::now().to_string(),
                ..Default::default()
            },
            DoctorCards {
                date: Date::now().to_string(),
                ..Default::default()
            },
            DoctorCards {
                date: Date::now().to_string(),
                ..Default::default()
            },
            DoctorCards {
                // Get the current date
                date: Date::now().to_string(),
                ..Default::default()
            },
            DoctorCards {
                date: Date::now().to_string(),
                ..Default::default()
            },
            DoctorCards {
                date: Date::now().to_string(),
                ..Default::default()
            },
            DoctorCards {
                date: Date::now().to_string(),
                ..Default::default()
            },
        ],
    };

    let return_template = grade.render().expect("Failed to render template");

    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(return_template))
}

#[derive(Debug, Default, Serialize)]
pub struct DoctorCards {
    pub date: String,
    pub description: String,
    pub image: String,
}
