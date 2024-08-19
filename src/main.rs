use std::{fs::File, io, process};

use actix_web::{get, http::KeepAlive, middleware, App, HttpResponse, HttpServer};
use askama::Template;
use log::{debug, error, info, LevelFilter};
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode, WriteLogger};

/// The local IP address of the server.
const HOST_IP: &str = "0.0.0.0"; // Local connection
/// The port that the server will listen on.
const PORT: u16 = 8099;
// const API_RS: &str = "127.0.0.1:8090"; // Communication with the signal generator
/// The name of the database that will be used.

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
            File::create("aj_quiz.log").unwrap(),
        ),
    ]) {
        Ok(()) => debug!("Logger initialized"),
        Err(err) => {
            error!("Error initializing logger: {err:#?}");
            process::exit(1);
        }
    }
    info!("Launched on PORT: {PORT}");

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .service(
                actix_files::Files::new("/static", "./static")
                    .show_files_listing()
                    .use_last_modified(true),
            )
            .service(index)
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

#[derive(Template)]
#[template(path = "index.html")]
struct Index<'a> {
    title: &'a str,
}

#[get("/")]
async fn index() -> HttpResponse {
    let template = Index { title: "AJ Quiz" };

    let body = match template.render() {
        Ok(body) => body,
        Err(err) => {
            error!("Error rendering template: {err:#?}");
            return HttpResponse::InternalServerError().finish();
        }
    };

    HttpResponse::Ok().body(body)
}
