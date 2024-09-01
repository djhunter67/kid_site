use std::net;

use actix_web::{http::KeepAlive, middleware, web::Data, App, HttpServer};
use mongodb::Database;
use tracing::error;

use crate::{
    endpoints::{
        health::health_check,
        index::index,
        login::{login, register, registration, submit_login},
        templates::{favicon, htmx, response_targets, source_map, stylesheet},
        users::{create, delete_user, get_user, get_users, update_user},
    },
    settings::{self, Settings},
};

pub struct Application {
    port: u16,
    server: actix_web::dev::Server,
}

impl Application {
    /// # Result
    ///  - `Ok(Application)` if the application was successfully built
    /// # Errors
    ///  - `std::io::Error` if the application could not be built
    /// # Panics
    ///  - If the application could not be built
    pub async fn build(
        settings: crate::settings::Settings,
        test_pool: Option<Database>,
    ) -> Result<Self, std::io::Error> {
        let connection_pool = if let Some(pool) = test_pool {
            pool
        } else {
            get_connection_pool(&settings.mongo).await
        };

        let address = format!(
            "{}:{}",
            settings.application.host, settings.application.port
        );

        let listener: net::TcpListener = net::TcpListener::bind(&address)?;
        let port = listener.local_addr()?.port();
        let server = run(listener, connection_pool, settings).await?;

        Ok(Self { port, server })
    }

    #[must_use]
    pub const fn port(&self) -> u16 {
        self.port
    }

    /// # Result
    ///  - `Ok(())` if the application was successfully started
    /// # Errors
    ///  - `std::io::Error` if the application could not be started
    /// # Panics
    ///  - If the application could not be started
    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

/// # Result
///  - `Ok(Database)` if the connection pool was successfully created
/// # Errors
///  - `mongodb::error::Error` if the connection pool could not be created
/// # Panics
///  - If the connection pool could not be created
pub async fn get_connection_pool(settings: &settings::Mongo) -> mongodb::Database {
    let mut client_options = settings.mongo_options().await;
    client_options.app_name = Some(settings.clone().db);

    let client = match mongodb::Client::with_options(client_options) {
        Ok(client) => client,
        Err(err) => {
            error!("Failed to connect to MongoDB: {err}\nExiting...");
            std::process::exit(1);
        }
    };
    client.database(&settings.db)
}

async fn run(
    listener: std::net::TcpListener,
    db_pool: mongodb::Database,
    settings: Settings,
) -> Result<actix_web::dev::Server, std::io::Error> {
    // Connect to the MongoDB database
    let db_data = Data::new(db_pool);

    // Redis connection pool
    let cfg = deadpool_redis::Config::from_url(settings.redis.url);
    let redis_pool = cfg
        .create_pool(Some(deadpool_redis::Runtime::Tokio1))
        .expect("Failed to create Redis pool");
    let redis_pool = Data::new(redis_pool);

    let server = HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .app_data(db_data.clone())
            .app_data(redis_pool.clone())
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
            .service(create)
            .service(get_user)
            .service(update_user)
            .service(delete_user)
            .service(get_users)
            .service(health_check)
    })
    .keep_alive(KeepAlive::Os) // Keep the connection alive; OS handled
    .disable_signals() // Disable the signals to allow the OS to handle the signals
    .shutdown_timeout(3)
    .workers(2)
    .listen(listener)?
    .run();

    Ok(server)
}
