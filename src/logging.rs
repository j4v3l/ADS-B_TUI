use crate::config::Config;
use std::fs::{self, OpenOptions};
use std::path::Path;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::fmt::time::ChronoLocal;
use tracing_subscriber::{fmt, EnvFilter};

pub fn init(config: &Config) -> Option<WorkerGuard> {
    if !config.log_enabled {
        return None;
    }

    let level = if config.log_level.trim().is_empty() {
        "info"
    } else {
        config.log_level.trim()
    };
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(level))
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let (writer, guard) = if config.log_file.trim().is_empty() {
        tracing_appender::non_blocking(std::io::stderr())
    } else {
        let path = Path::new(config.log_file.trim());
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                let _ = fs::create_dir_all(parent);
            }
        }
        match OpenOptions::new()
            .create(true)
            .append(true)
            .open(&config.log_file)
        {
            Ok(file) => tracing_appender::non_blocking(file),
            Err(_) => tracing_appender::non_blocking(std::io::stderr()),
        }
    };

    let subscriber = fmt::Subscriber::builder()
        .with_env_filter(filter)
        .with_writer(writer)
        .with_ansi(false)
        .with_level(true)
        .with_target(true)
        .with_timer(ChronoLocal::new("%Y-%m-%d %H:%M:%S%.3f".to_string()))
        .compact()
        .finish();

    let _ = tracing::subscriber::set_global_default(subscriber);
    Some(guard)
}
