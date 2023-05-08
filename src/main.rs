use clap::Parser;
use color_eyre::Result;
use redacted_debug::RedactedDebug;
use std::path::PathBuf;
use tracing::trace;
use tracing_subscriber::prelude::*;

#[derive(RedactedDebug, Clone, Parser)]
struct Args {
    /// Don't perform any action
    #[arg(short = 'n', long)]
    dry: bool,
    /// Base path for images
    #[arg(short, long, env = "PWD")]
    path: PathBuf,
    /// Danbooru username
    #[arg(env = "DANBOORU_USERNAME", long)]
    #[redacted]
    danbooru_username: String,
    /// Danbooru apikey
    #[arg(env = "DANBOORU_APIKEY", long)]
    #[redacted]
    danbooru_apikey: String,
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let layer_filter = tracing_subscriber::EnvFilter::from_default_env()
        .add_directive("info".parse()?)
        .add_directive("plymouth_bot=trace".parse()?);

    let layer_fmt = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stderr)
        // .without_time()
        .with_line_number(true)
        .compact();

    tracing_subscriber::registry()
        .with(layer_filter)
        .with(layer_fmt)
        .init();

    let args = Args::parse();

    trace!("Tracing setup");
    trace!(?args);

    Ok(())
}
