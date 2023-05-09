use std::{fmt, path::Path, time::Duration};

use tokio::time::sleep;
use tracing::{instrument, warn};

#[instrument(ret, level = "debug")]
pub(crate) async fn tag<P: AsRef<Path> + fmt::Debug>(file: P) {
    warn!("ZZZ");
    sleep(Duration::from_secs(2)).await;
    warn!("Awake");
}
