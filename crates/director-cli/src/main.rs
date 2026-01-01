use clap::{Parser, ValueEnum};
use director_core::export::render_export;
use director_core::scripting::register_rhai_api;
use director_core::DefaultAssetLoader;
use rhai::Engine;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{error, info};
use tracing_subscriber::{fmt, EnvFilter};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the Rhai script
    #[arg(value_name = "SCRIPT")]
    script: PathBuf,

    /// Output video path
    #[arg(value_name = "OUTPUT")]
    output: Option<PathBuf>,

    /// Log level
    #[arg(long, value_enum, default_value_t = LogLevel::Info)]
    log_level: LogLevel,

    /// Log format
    #[arg(long, value_enum, default_value_t = LogFormat::Pretty)]
    log_format: LogFormat,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Error => write!(f, "error"),
            LogLevel::Warn => write!(f, "warn"),
            LogLevel::Info => write!(f, "info"),
            LogLevel::Debug => write!(f, "debug"),
            LogLevel::Trace => write!(f, "trace"),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum LogFormat {
    Pretty,
    Json,
}

fn main() {
    let cli = Cli::parse();

    // Initialize Logging
    let filter = EnvFilter::builder()
        .with_default_directive(cli.log_level.to_string().parse().unwrap())
        .from_env_lossy();

    let subscriber_builder = fmt::Subscriber::builder()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .with_target(false); // Clean output for humans (default)

    match cli.log_format {
        LogFormat::Json => {
            // For JSON, we might want target included, but user said strict NDJSON.
            // subscriber.json() enables JSON formatting.
            // We need to finish building.
            subscriber_builder.json().init();
        }
        LogFormat::Pretty => {
            subscriber_builder.pretty().init();
        }
    }

    let script_path = cli.script;
    let output_path = if let Some(out) = cli.output {
        out
    } else {
        let mut p = script_path.clone();
        p.set_extension("mp4");
        p
    };

    info!("Initializing Director Engine...");
    info!("Script: {:?}", script_path);
    info!("Output: {:?}", output_path);

    let script = match fs::read_to_string(&script_path) {
        Ok(s) => s,
        Err(e) => {
            error!("Error reading script file: {}", e);
            return;
        }
    };

    let mut engine = Engine::new();
    register_rhai_api(&mut engine, Arc::new(DefaultAssetLoader));

    match engine.eval::<director_core::scripting::MovieHandle>(&script) {
        Ok(movie) => {
            info!("Script evaluated successfully. Starting render...");
            let mut director = movie.director.lock().unwrap();
            match render_export(&mut director, output_path, None, None) {
                Ok(_) => info!("Render complete."),
                Err(e) => {
                    error!("Render failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            error!("Script Error: {}", e);
            std::process::exit(1);
        }
    }
}
