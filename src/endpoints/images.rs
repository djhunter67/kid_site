use std::path::PathBuf;

use actix_files::NamedFile;
use actix_web::get;
use log::{error, info};

#[get("/english_image")]
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
