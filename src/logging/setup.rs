//! Logging initialization and configuration

use crate::error::Result;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Initialize logging with the specified log level
pub fn init_logging(log_level: &str) -> Result<()> {
    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(log_level))
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer().with_writer(std::io::stdout))
        .init();

    Ok(())
}
