use std::net;

use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{
    http::KeepAlive,
    web::{scope, Data},
    App, HttpServer,
};
use log::{debug, error, info, warn};
use mongodb::Database;

use crate::{
    endpoints::{
        health::health_check,
        index::index,
        login::{login, login_user},
        register::{register, registration},
        templates::{
            english_image, favicon, htmx, response_targets, science_image, source_map, stylesheet,
        },
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
        info!("Buidling the main application");
        let connection_pool = if let Some(pool) = test_pool {
            pool
        } else {
            get_connection_pool(&settings.mongo).await
        };

        let address = format!(
            "{}:{}",
            settings.application.host, settings.application.port
        );

        debug!("Binding the TCP port: {address}");
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
        info!("Running until stopped");
        self.server.await
    }
}

/// # Result
///  - `Ok(Database)` if the connection pool was successfully created
/// # Errors
///  - `mongodb::error::Error` if the connection pool could not be created
/// # Panics
///  - If the connection pool could not be created
async fn get_connection_pool(settings: &settings::Mongo) -> mongodb::Database {
    info!("Get mongo connection pool");
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

#[allow(clippy::unused_async)]
async fn run(
    listener: std::net::TcpListener,
    db_pool: mongodb::Database,
    settings: Settings,
) -> Result<actix_web::dev::Server, std::io::Error> {
    // For each session
    let secret_key = actix_web::cookie::Key::from(settings.secret.hmac_secret.as_bytes());
    info!("Obtaining the cookie secret");

    // Connect to the MongoDB database
    let db_data = Data::new(db_pool);
    info!("Processed DB connection pool for distribution");

    // Redis connection pool
    let cfg = deadpool_redis::Config::from_url(settings.redis.url);
    let redis_pool = match cfg.create_pool(Some(deadpool_redis::Runtime::Tokio1)) {
        Ok(pool) => pool,
        Err(err) => {
            error!("Failed to connect to Redis: {err}\nExiting...");
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "Failed to connect to Redis",
            ));
        }
    };

    info!("Established Redis connection pool");
    let redis_pool = Data::new(redis_pool);

    let server = HttpServer::new(move || {
        App::new()
            .wrap(if settings.debug {
                warn!("Debug mode");
                SessionMiddleware::builder(CookieSessionStore::default(), secret_key.clone())
                    .cookie_http_only(true)
                    .cookie_same_site(actix_web::cookie::SameSite::None)
                    .cookie_secure(true)
                    .build()
            } else {
                // TODO: Add a secure cookie
                warn!("Production mode");
                SessionMiddleware::new(CookieSessionStore::default(), secret_key.clone())
            })
            // .wrap(middleware::Logger::default())
            .app_data(db_data.clone())
            .app_data(redis_pool.clone())
            .service(favicon)
            .service(stylesheet)
            .service(source_map)
            .service(htmx)
            .service(response_targets)
            .service(english_image)
            .service(science_image)
            .service(login)
            .service(index)
            .service(login_user)
            .service(registration)
            .service(register)
            .service(
                scope("/v1")
                    .service(create)
                    .service(get_user)
                    .service(update_user)
                    .service(delete_user)
                    .service(get_users),
            )
            .service(health_check)
    })
    .keep_alive(KeepAlive::Os) // Keep the connection alive; OS handled
    .disable_signals() // Disable the signals to allow the OS to handle the signals
    .workers(1)
    .shutdown_timeout(3)
    .listen(listener)?
    .run();

    Ok(server)
}
