use std::path::PathBuf;

use actix_files::NamedFile;
use actix_web::get;
use tracing::{error, info, instrument};

#[get("/english_image")]
#[instrument(name = "English image", level = "info", target = "kid_data")]
async fn english_image() -> Result<NamedFile, actix_web::Error> {
    info!("Serving english_image.png");
    let path: PathBuf = ["static", "imgs", "english.png"].iter().collect();
    match NamedFile::open(path) {
        Ok(file) => Ok(file),
        Err(err) => {
            error!("Error opening file: {err:#?}");
            Err(actix_web::error::ErrorInternalServerError(err))
        }
    }
}

#[get("/science_image")]
#[instrument(name = "Science image", level = "info", target = "kid_data")]
async fn science_image() -> Result<NamedFile, actix_web::Error> {
    info!("Serving science_image.png");
    let path: PathBuf = ["static", "imgs", "science.png"].iter().collect();
    match NamedFile::open(path) {
        Ok(file) => Ok(file),
        Err(err) => {
            error!("Error opening file: {err:#?}");
            Err(actix_web::error::ErrorInternalServerError(err))
        }
    }
}

#[get("/math_image")]
#[instrument(name = "Math image", level = "info", target = "kid_data")]
async fn math_image() -> Result<NamedFile, actix_web::Error> {
    info!("Serving math_image.png");
    let path: PathBuf = ["static", "imgs", "math.jpeg"].iter().collect();
    match NamedFile::open(path) {
        Ok(file) => Ok(file),
        Err(err) => {
            error!("Error opening file: {err:#?}");
            Err(actix_web::error::ErrorInternalServerError(err))
        }
    }
}

#[get("/social_studies_image")]
#[instrument(name = "Social studies image", level = "info", target = "kid_data")]
async fn social_studies_image() -> Result<NamedFile, actix_web::Error> {
    info!("Serving social_studies_image.png");
    let path: PathBuf = ["static", "imgs", "social_studies.jpeg"].iter().collect();
    match NamedFile::open(path) {
        Ok(file) => Ok(file),
        Err(err) => {
            error!("Error opening file: {err:#?}");
            Err(actix_web::error::ErrorInternalServerError(err))
        }
    }
}

#[get("/dental_image")]
#[instrument(name = "Dental image", level = "info", target = "kid_data")]
async fn dental_image() -> Result<NamedFile, actix_web::Error> {
    info!("Serving math_image.png");
    let path: PathBuf = ["static", "imgs", "dental_img.gif"].iter().collect();
    match NamedFile::open(path) {
        Ok(file) => Ok(file),
        Err(err) => {
            error!("Error opening file: {err:#?}");
            Err(actix_web::error::ErrorInternalServerError(err))
        }
    }
}
