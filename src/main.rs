use aj_studying::{settings, startup::Application, telemetry};
use std::io;
use tracing::{info, warn};

#[actix_web::main]
async fn main() -> io::Result<()> {
    // This is a macro that allows for multiple loggers to be used at once

    dotenv::dotenv().ok();

    let settings = settings::get().expect("Failed to get application settings");

    let subscriber = telemetry::get_subcriber(settings.clone().debug);
    telemetry::init_subscriber(subscriber);

    info!("Building the application");
    let application = Application::build(settings, None).await?;

    info!("Listening on port: {}", application.port());
    application.run_until_stopped().await?;
    warn!("Shutting down");

    Ok(())
}
