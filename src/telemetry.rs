use tracing_subscriber::prelude::*;

use crate::settings::logging::*;

pub fn init(settings: &Settings) {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| format!("{}", settings.level).into());

    match settings.format {
        Format::Full => {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(tracing_subscriber::fmt::layer())
                .init();
        }

        Format::Compact => {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(tracing_subscriber::fmt::layer().compact())
                .init();
        }

        Format::Pretty => {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(tracing_subscriber::fmt::layer().pretty())
                .init();
        }

        Format::Json => {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(tracing_subscriber::fmt::layer().json())
                .init();
        }
    }
}
