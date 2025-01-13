use std::str::FromStr;

use actix_web::{
    get,
    http::{header::Date, StatusCode},
    web::{self, Data, Json},
    Error, HttpResponse,
};
use askama::Template;
use mongodb::{bson::oid::ObjectId, Database};
use serde::{Deserialize, Serialize};
use tracing::{error, instrument, warn};

use crate::{
    endpoints::{
        error::render_error,
        templates::{DoctorData, DoctorVisit},
    },
    models::mongo::MongoRepo,
    settings::Settings,
};

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
    pub db_id: ObjectId,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Appointment {
    pub date: String,
    pub notes: String,
    pub purpose: String,
}

#[get("/doctor_card/{id}")]
#[instrument(
    name = "recorded appointment",
    level = "info",
    target = "kid_data",
    skip(id, pool)
)]
pub async fn doctor_card(id: web::Path<String>, pool: Data<Database>) -> HttpResponse {
    let con = MongoRepo::new(&pool.clone().as_ref().to_owned(), None);

    warn!("ID passed: {}", ObjectId::from_str(&id).expect(""));
    match con
        .get_user(
            Some(ObjectId::from_str(id.as_str()).expect("Passed in data is not an Object ID")),
            None,
        )
        .await
    {
        Ok(res) => {
            warn!("DB user: {res}");
            // Describe and implement the web view for the doctor visit
            let template = DoctorVisit {
                date: &Date::now().to_string(),
                notes: vec![],
                purpose: "Annual checkup",
            };

            return HttpResponse::Ok()
                .content_type("text/html")
                .body(template.render().expect("Unable to render template"));
        }
        Err(err) => {
            error!("Unable to find the data for the ID passed in");
            return render_error(
                StatusCode::NOT_FOUND,
                "ID lookup failed",
                Some(&err.to_string()),
            );
        }
    };
}

#[get("/visit_notes/{id}")]
#[instrument(
    name = "appointment form",
    level = "info",
    target = "kid_data",
    skip(id, pool)
)]
pub async fn add_doctor_visit(
    id: web::Path<String>,
    pool: Data<Database>,
    appointment: Json<Appointment>,
) -> HttpResponse {
    let _con: MongoRepo = MongoRepo::new(pool.as_ref(), Some("Visits"));

    warn!("ID passed: {id}");

    // Save the form data to the Database

    HttpResponse::Ok().finish()
}
