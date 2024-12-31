use actix_web::{get, web, Error, HttpResponse};
use askama::Template;
use mongodb::Database;
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
        age: "Kid age",
        name: &settings.doctor.name.clone(),
        email: &settings.doctor.email.clone(),
        phone: &settings.doctor.phone.clone(),
        address: &settings.doctor.address.clone(),
        speciality: &settings.doctor.speciality.clone(),
    };

    let return_template = grade.render().expect("Failed to render template");

    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(return_template))
}
