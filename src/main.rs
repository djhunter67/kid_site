use aj_studying::{settings, startup::Application};
use log::{debug, error, info, warn, LevelFilter};
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode, WriteLogger};
use std::{fs::File, io, process::exit};

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
            LevelFilter::Trace,
            Config::default(),
            File::create("scan_yam.log")?,
        ),
    ]) {
        Ok(()) => debug!("Logger initialized."),
        Err(e) => {
            error!("Error initializing logger: {e:?}");
            exit(1);
        }
    }

    info!("Loading env variables");
    dotenv::dotenv().ok();

    info!("Init settings");
    let settings = settings::get().expect("Failed to get application settings");

    info!("Building the application");
    let application = Application::build(settings, None).await?;

    info!("Listening on port: {}", application.port());
    application.run_until_stopped().await?;
    warn!("Shutting down");

    Ok(())
}
