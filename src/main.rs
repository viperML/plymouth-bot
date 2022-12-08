pub mod danbooru;
pub mod saucenao;

use anyhow::{anyhow, bail, Context};
use danbooru::DanbooruId;
use log::{error, info, warn};
use saucenao::SaucenaoClient;

use clap::Parser;

use std::fmt::Debug;
use std::fs::{self};
use std::path::Path;

use crate::danbooru::DanbooruClient;

const CAG_INPUT: &str = "CAG_INPUT";

const EXTENSIONS: &[&str] = &["png", "jpg", "jpeg"];
const MINIMUM_SIMILARITY: f32 = 70.0;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = 1)]
    /// Number of files to process
    items: usize,
    #[arg(short = 'n', long)]
    /// Don't move any file
    dry: bool,
}

fn main() {
    match real_main() {
        Ok(_) => {
            std::process::exit(0);
        }
        Err(error) => {
            for chain_elem in error.chain().rev() {
                error!("{}", chain_elem);
            }
            std::process::exit(1);
        }
    }
}

fn real_main() -> anyhow::Result<()> {
    setup_logging()?;
    let args = Args::parse();

    let files = fs::read_dir(CAG_INPUT)
        .context("Failed to open CAG_INPUT")?
        .map(|entry| entry.map(|direntry| direntry.path()))
        .collect::<Result<Vec<_>, _>>()
        .context("Failed to get CAG_INPUT items")?;

    let files = files
        .iter()
        .filter(|elem| match elem.extension().and_then(|o_s| o_s.to_str()) {
            None => false,
            Some(ext) => EXTENSIONS.contains(&ext),
        })
        .collect::<Vec<_>>();

    let saucenao_apikey =
        std::env::var("SAUCENAO_APIKEY").context("Failed to get saucenao credentials")?;
    let mut saucer = SaucenaoClient::new(&saucenao_apikey);
    info!("param MINIMUM_SIMILARITY: {MINIMUM_SIMILARITY}");

    let danbooru_client = DanbooruClient::new(
        &std::env::var("DANBOORU_USERNAME").context("Failed to get Danbooru credentials")?,
        &std::env::var("DANBOORU_APIKEY").context("Failed to get Danbooru credentials")?,
    );

    for &path in files.iter().take(args.items) {
        let file_name = path
            .file_name()
            .ok_or_else(|| anyhow!("Failed to get failname for {:?}", path))?;

        info!("Processing {:?}", path);

        if saucer.short_remaining == 0 {
            warn!("Short limit reached, sleeping 30 seconds");
            let duration = std::time::Duration::from_secs(31);
            std::thread::sleep(duration);
        }

        match saucer.tag_image(path, MINIMUM_SIMILARITY) {
            Ok(id) => {
                info!("Adding to favourites!");
                dispatch(path, Some((&id, &danbooru_client)), args.dry)?;
            }
            Err(error) => {
                if error.chain().any(|cause| {
                    matches!(cause.downcast_ref(), Some(saucenao::SauceError::NoMatch))
                }) {
                    dispatch(path, None, args.dry)?;
                } else {
                    bail!(error);
                };
            }
        }
    }

    Ok(())
}

fn setup_logging() -> anyhow::Result<()> {
    let colors = fern::colors::ColoredLevelConfig::new()
        .info(fern::colors::Color::Green)
        .debug(fern::colors::Color::BrightBlue);

    let loglevel = if cfg!(debug_assertions) {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };

    let logfile = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .append(false)
        .truncate(true)
        .open("plymouth-bot.log")?;

    fern::Dispatch::new()
        .level(loglevel)
        .chain(std::io::stdout())
        .chain(logfile)
        .format(move |out, message, record| {
            out.finish(std::format_args!(
                "[{}][{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                colors.color(record.level()),
                message
            ))
        })
        .apply()?;

    Ok(())
}

fn dispatch<P: AsRef<Path>>(
    path: P,
    target: Option<(&DanbooruId, &DanbooruClient)>,
    dry: bool,
) -> anyhow::Result<()> {
    match target {
        None => {
            if !dry {
                info!("Nosaucing file");
                organize_file(path, false)?;
            }
        }
        Some((id, client)) => {
            client.fav_post(id)?;
            if !dry {
                info!("Saucing file");
                organize_file(path, true)?;
            }
        }
    }

    Ok(())
}

fn organize_file<P: AsRef<Path>>(path: P, sauced: bool) -> Result<(), std::io::Error> {
    let file_name = path.as_ref().file_name().unwrap();

    let new_path = if sauced {
        Path::new("CAG_SAUCE").join(file_name)
    } else {
        Path::new("CAG_NOSAUCE").join(file_name)
    };

    info!("{:?} -> {:?}", path.as_ref(), new_path);

    std::fs::rename(path.as_ref(), new_path)
}
