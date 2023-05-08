use clap::Parser;
use color_eyre::Result;
use redacted_debug::RedactedDebug;
use std::{
    collections::VecDeque,
    path::{Path, PathBuf},
};
use tracing::{debug, trace};
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

#[derive(Debug, Clone)]
struct Folders {
    input: PathBuf,
    output_sauce: PathBuf,
    output_nosauce: PathBuf,
}

impl Folders {
    fn new<P: AsRef<Path>>(base: P) -> Self {
        let base = PathBuf::from(base.as_ref());
        Self {
            input: base.join("CAG_INPUT"),
            output_sauce: base.join("CAG_SAUCE"),
            output_nosauce: base.join("CAG_NOSAUCE"),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
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

    let folders = Folders::new(&args.path);
    debug!(?folders);

    let files: VecDeque<_> = std::fs::read_dir(folders.input)?
        .into_iter()
        .flatten()
        .map(|elem| elem.path())
        .collect();

    debug!(?files);

    Ok(())
}
