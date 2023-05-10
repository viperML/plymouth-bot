use clap::Parser;
use color_eyre::{
    eyre::{Context, ContextCompat},
    Report, Result,
};
use futures::{stream::FuturesUnordered, StreamExt};
use redacted_debug::RedactedDebug;

use std::{
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
};
use tokio::{select, sync::oneshot::Sender};
use tracing::{debug, info, trace};
use tracing_subscriber::prelude::*;

mod danbooru;
mod saucenao;

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
    /// Reduce the short_remaining timeout for easier debugging
    #[arg(short, long)]
    slow: bool,
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
    setup()?;
    let args = Args::parse();

    let config_folders = Folders::new(&args.path);
    debug!(?config_folders);

    let input_files: Vec<_> = std::fs::read_dir(config_folders.input)
        .context("Reading input folder")?
        .into_iter()
        .flatten()
        .map(|elem| elem.path())
        .collect();

    debug!(?input_files);

    let (tx_sauce, mut rx_sauce) = tokio::sync::mpsc::unbounded_channel::<GetSauce>();

    let mut futs = FuturesUnordered::new();

    let danbooru_client = Rc::new(danbooru::DanbooruClient::new(
        &args.danbooru_username,
        &args.danbooru_apikey,
    ));

    for file in input_files {
        let tx_sauce = tx_sauce.clone();
        let danbooru_client = danbooru_client.clone();
        let task = async move {
            let (tx_similarity, rx_sim) = tokio::sync::oneshot::channel();

            let file_contents = tokio::fs::read(file).await?;

            let cmd = GetSauce {
                file_contents,
                responder: tx_similarity,
            };

            tx_sauce.send(cmd).unwrap();
            let _match = rx_sim.await.unwrap();
            info!(?_match);

            danbooru_client.fav(_match.danbooru_id).await?;

            Ok::<_, Report>(())
        };
        futs.push(task);
    }

    let mut sauce_client = saucenao::SauceNaoClient::new(&args.saucenao_apikey);
    sauce_client.slow = args.slow;

    loop {
        select! {
            Some(msg) = rx_sauce.recv() => {
                let resp = sauce_client.tag(msg.file_contents).await?;
                msg.responder.send(resp).unwrap();
            }
            n = futs.next() => match n {
                None => break,
                Some(_) => {}
            }
        }
    }

    Ok(())
}

#[derive(Debug)]
struct GetSauce {
    file_contents: Vec<u8>,
    responder: Sender<saucenao::Match>,
}

fn setup() -> Result<()> {
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

    trace!("Tracing setup");
    Ok(())
}
