mod endpoints;
mod models;

use actix_files::NamedFile;
use actix_web::{
    get,
    http::KeepAlive,
    middleware::{self},
    web::Data,
    App, HttpResponse, HttpServer, Responder,
};
use endpoints::{
    index::index,
    login::{login, register, registration, submit_login},
    users::{create_user, delete_user, get_user, get_users, update_user},
};
use log::{debug, error, info, LevelFilter};
use models::mongo::MongoRepo;
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode, WriteLogger};
use std::{fs::File, io, path::PathBuf, process};

/// The local IP address of the server.
const HOST_IP: &str = "0.0.0.0"; // Local connection
/// The port that the server will listen on.
const PORT: u16 = 8099;

#[actix_web::main]
async fn main() -> io::Result<()> {
    // This is a macro that allows for multiple loggers to be used at once
    match CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Debug,
            Config::default(),
            TerminalMode::Stdout,
            ColorChoice::Always,
        ),
        WriteLogger::new(
            LevelFilter::Debug,
            Config::default(),
            match File::create("aj_quiz.log") {
                Ok(file) => file,
                Err(err) => {
                    error!("Error creating log file: {err:#?}");
                    File::create("/tmp/aj_quiz.log").expect("Error creating backup log file")
                }
            },
        ),
    ]) {
        Ok(()) => debug!("Logger initialized"),
        Err(err) => {
            error!("Error initializing logger: {err:#?}");
            process::exit(1);
        }
    }
    info!("Launched on PORT: {PORT}");

    let db = MongoRepo::init().await;
    let db_data = Data::new(db);

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .app_data(db_data.clone())
            // .service(
            //     actix_files::Files::new("/static", ".")
            //         .index_file("base.html")
            //         .prefer_utf8(true)
            //         .show_files_listing()
            //         .use_last_modified(true),
            // )
            .service(favicon)
            .service(stylesheet)
            .service(source_map)
            .service(htmx)
            .service(response_targets)
            .service(login)
            .service(index)
            .service(submit_login)
            .service(registration)
            .service(register)
            // Database operations
            .service(create_user)
            .service(get_user)
            .service(update_user)
            .service(delete_user)
            .service(get_users)
    })
    .keep_alive(KeepAlive::Os) // Keep the connection alive; OS handled
    .bind((HOST_IP, PORT))
    .unwrap_or_else(|_| {
        error!("Error binding to port {}.", PORT);
        process::exit(1); // This is expected behavior if the port is already in use
    })
    .disable_signals() // Disable the signals to allow the OS to handle the signals
    .shutdown_timeout(3)
    .workers(2)
    .run()
    .await
}

#[get("/favicon")]
async fn favicon() -> impl Responder {
    let file = include_str!("../static/imgs/education.svg");
    HttpResponse::Ok().content_type("icon").body(file)
}

#[get("/stylesheet")]
async fn stylesheet() -> impl Responder {
    let file = include_str!("../static/css/style.css");
    HttpResponse::Ok().content_type("text/css").body(file)
}

#[get("/style.css.map")]
async fn source_map() -> impl Responder {
    let file = include_str!("../static/css/style.css.map");
    HttpResponse::Ok()
        .content_type("application/json")
        .body(file)
}

#[get("/htmx")]
async fn htmx() -> Result<NamedFile, actix_web::Error> {
    let path: PathBuf = ["static", "assets", "htmx", "htmx.min.js"].iter().collect();
    match NamedFile::open(path) {
        Ok(file) => Ok(file),
        Err(err) => {
            error!("Error opening file: {err:#?}");
            Err(actix_web::error::ErrorInternalServerError(err))
        }
    }
}

#[get("/response-targets")]
async fn response_targets() -> Result<NamedFile, actix_web::Error> {
    let pash: PathBuf = ["static", "assets", "htmx", "response-targets.js"]
        .iter()
        .collect();
    match NamedFile::open(pash) {
        Ok(file) => Ok(file),
        Err(err) => {
            error!("Error opening file: {err:#?}");
            Err(actix_web::error::ErrorInternalServerError(err))
        }
    }
}
