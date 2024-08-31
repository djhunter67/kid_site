use actix_web::{http::KeepAlive, middleware, web::Data, App, HttpServer};

use crate::{endpoints::{index::index, login::{login, register, registration, submit_login}, templates::{favicon, htmx, response_targets, source_map, stylesheet}, users::{create_user, delete_user, get_user, get_users, update_user}}, models::mongo::MongoRepo};

pub struct Application {
    port: u16,
    server: actix_web::dev::Server,
}

impl Application {
    pub async fn build(settings: crate::settings::Settings) -> Result<Self, std::io::Error> {
        let address = format!(
            "{}:{}",
            settings.application.host, settings.application.port
        );

        let listener = std::net::TcpListener::bind(&address)?;
        let port = listener.local_addr()?.port();
        let server = run(listener).await?;

        Ok(Self { port, server })
    }

    #[must_use]
    pub const fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

async fn run(listener: std::net::TcpListener) -> Result<actix_web::dev::Server, std::io::Error> {
    let db = MongoRepo::init().await;
    let db_data = Data::new(db);

    let server = HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .app_data(db_data.clone())
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
    .disable_signals() // Disable the signals to allow the OS to handle the signals
    .shutdown_timeout(3)
    .workers(2)
    .listen(listener)?
    .run();

    Ok(server)
}
