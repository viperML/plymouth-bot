use clap::Parser;
use color_eyre::{eyre::Context, Result};
use futures::StreamExt;
use redacted_debug::RedactedDebug;
use std::{
    collections::VecDeque,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use tokio::task::JoinHandle;
use tracing::{debug, info, trace, warn};
use tracing_subscriber::prelude::*;

mod handler;

#[derive(RedactedDebug, Clone, Parser)]
struct Args {
    /// Don't perform any action
    #[arg(short = 'n', long)]
    dry: bool,
    /// Base path for images
    #[arg(short, long, default_value = ".")]
    path: PathBuf,
    /// Danbooru username
    #[arg(env = "DANBOORU_USERNAME", long)]
    #[redacted]
    danbooru_username: String,
    /// Danbooru apikey
    #[arg(env = "DANBOORU_APIKEY", long)]
    #[redacted]
    danbooru_apikey: String,
    /// SauceNao apikey
    #[redacted]
    #[arg(env = "SAUCENAO_APIKEY", long)]
    saucenao_apikey: String,
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

    let args = Arc::new(Args::parse());

    trace!("Tracing setup");
    trace!(?args);

    let folders = Folders::new(&args.path);
    debug!(?folders);

    let input_files: VecDeque<_> = std::fs::read_dir(folders.input)
        .context("Reading input folder")?
        .into_iter()
        .flatten()
        .map(|elem| elem.path())
        .collect();

    debug!(?input_files);

    // let futs = tokio::stream::
    let futs = &mut futures::stream::FuturesUnordered::new();

    let saucenao_client = Arc::new(handler::SauceNaoClient::new(&args.saucenao_apikey));

    for f in input_files.iter().take(1) {
        let args = args.clone();
        let sc = saucenao_client.clone();
        let task = async move { handler::tag(f, &args, &sc).await };

        futs.push(task);
    }

    while let Some(x) = futs.next().await {
        match x {
            Ok(req) => info!(?req),
            Err(report) => warn!(?report),
        }
    }

    Ok(())
}
