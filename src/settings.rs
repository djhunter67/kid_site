use mongodb::options::ClientOptions;
use serde::Deserialize;

/// Global setting for exposing all preconfigured variables
#[derive(Deserialize, Clone)]
pub struct Settings {
    pub application: Application,
    pub debug: bool,
    pub mongo: Mongo,
    pub redis: Redis,
}

/// Redis setting for the entire application
#[derive(Deserialize, Clone, Debug)]
pub struct Redis {
    pub url: String,
    pub pool_max_open: u64,
    pub pool_max_idle: u64,
    pub pool_timeout_seconds: u64,
    pub pool_expire_seconds: u64,
}

/// Mongo setting for the entire application
#[derive(Deserialize, Clone, Debug)]
pub struct Mongo {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub db: String,
    pub collection: String,
    pub require_auth: bool,
}

/// Application's specific settings to expose `port`,
/// `host`, `protocol`, and possible URL of the application
/// during and after development
#[derive(Deserialize, Clone)]
pub struct Application {
    pub port: u16,
    pub host: String,
    pub base_url: String,
    pub protocol: String,
}

impl Mongo {
    #[must_use]
    /// # Returns
    ///   - Returns a `ClientOptions` for connecting to ``MongoDB``
    /// # Errors
    ///   - Panics if the ``MongoDB`` environment variables are not set
    /// # Panics
    ///   - Panics if the ``MongoDB`` environment variables are not set
    /// # Notes
    ///   - mongodb://<credentials>@127.0.0.1:27017/?directConnection=true&serverSelectionTimeoutMS=2000&appName=mongosh+2.3.0
    pub async fn mongo_options(&self) -> ClientOptions {
        let login_config = if self.require_auth {
            format!(
                "mongodb://{}:{}@{}:{}/{}?directConnection=true&serverSelectionTimeoutMS=2000&appName=mongosh+2.3.0",
                self.username, self.password, self.host, self.port, self.db
            )
        } else {
            format!(
                "mongodb://{}:{}/{}?directConnection=true&serverSelectionTimeoutMS=2000&appName=mongosh+2.3.0",
                self.host, self.port, self.db
            )
        };
        ClientOptions::parse(login_config)
            .await
            .expect("Unable to parse the MongoDB environment variables")
    }
}

/// The possible runtime environment for our application
pub enum Environment {
    Development,
    Production,
}

impl Environment {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Development => "development",
            Self::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "development" => Ok(Self::Development),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{other} is not a supported environment. Use either 'production' or 'development'."
            )),
        }
    }
}

/// # Result
///   - Returns a `Result` of `Settings` if successful
/// # Errors
///   - Returns a `config::ConfigError` if there is an error loading the settings
/// # Panics
///   - Panics if the current directory cannot be determined
/// # Notes
///   - Multipurpose function that helps detect the current environment the application
///     is running using the `APP_ENVIRONMENT` environment variable.
///
/// \\\
/// ``APP_ENVIRONMENT`` = development | production.
/// \\\
///
/// After detection, it loads the appropriate .yaml file
/// then it loads the environment variable that overrides whatever is set in the .yaml file.
/// For this to work, you the environment variable MUST be in uppercase and starts with `APP`,
/// a "_" separator then the category of settings,
/// followed by "__" separator,  and then the variable.
/// # Example
///   - ``APP__APPLICATION_PORT=5001`` for "port" to be set as "5001"
pub fn get() -> Result<Settings, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    let setting_directory = base_path.join("settings");

    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "development".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT.");
    let environment_filename = format!("{}.yaml", environment.as_str());
    let settings = config::Config::builder()
        .add_source(config::File::from(setting_directory.join("base.yaml")))
        .add_source(config::File::from(
            setting_directory.join(environment_filename),
        ))
        // Add in settings from environment variables (with a prefix of APP and '__' as seperator)
        // e.g `APP_APPLICATION__PORT_5001 would set `Setting.application.port`
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()?;

    settings.try_deserialize::<Settings>()
}
