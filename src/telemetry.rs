use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

/// Output JSON in production only
pub fn get_subcriber(debug: bool) -> impl tracing::Subscriber + Send + Sync {
    let env_filter = if debug {
        String::from("trace")
    } else {
        String::from("info")
    };

    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));

    let stdout_log = tracing_subscriber::fmt::layer().pretty();
    let subscriber = Registry::default().with(env_filter).with(stdout_log);

    let json_log = if debug {
        None
    } else {
        let json_log = tracing_subscriber::fmt::layer().json();
        Some(json_log)
    };

    subscriber.with(json_log)
}

pub fn init_subscriber(subscriber: impl tracing::Subscriber + Send + Sync) {
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");
}
