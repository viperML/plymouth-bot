use std::{collections::HashMap, fmt, path::Path, time::Duration};

use color_eyre::{
    eyre::{bail, Context, ContextCompat},
    Result,
};
use reqwest::Request;
use serde::Deserialize;
use serde_json::Value;
use tokio::time::sleep;
use tracing::{info, instrument, trace, warn};

#[derive(redacted_debug::RedactedDebug)]
pub(crate) struct SauceNaoClient {
    api_key: String,
    client: reqwest::Client,
}

impl SauceNaoClient {
    pub(crate) fn new<S: AsRef<str>>(api_key: S) -> Self {
        let api_key = api_key.as_ref();

        Self {
            api_key: api_key.to_string(),
            client: reqwest::Client::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[non_exhaustive]
struct SauceNaoResponse {
    header: Value,
    results: Vec<SauceNaoResult>,
}

#[derive(Debug, Deserialize)]
#[non_exhaustive]
struct SauceNaoResult {
    data: Value,
    header: HashMap<String, Value>,
}

pub(crate) async fn tag<P: AsRef<Path> + fmt::Debug>(
    file: P,
    args: &crate::Args,
    c: &SauceNaoClient,
) -> Result<()> {
    let file = file.as_ref();
    let contents = tokio::fs::read(file).await.context("Reading file")?;

    let url = reqwest::Url::parse_with_params(
        "https://saucenao.com/search.php",
        &[
            ("output_type", "2"),
            ("numres", "1"),
            ("minsim", "85"),
            ("db", "9"),
            ("api_key", &c.api_key),
        ],
    )?;

    let part = reqwest::multipart::Part::bytes(contents).file_name("file");
    let form = reqwest::multipart::Form::new().part("file", part);
    let req = c.client.post(url).multipart(form).build()?;

    let response = c.client.execute(req).await?;

    let decoded: SauceNaoResponse = response.json().await?;
    trace!(?decoded);
    // let pretty = serde_json::to_string_pretty(&decoded)?;
    // println!("{}", pretty);

    let first_result = decoded.results.get(0).context("Reading first result")?;
    info!(?first_result);
    let similarity = first_result
        .header
        .get("similarity")
        .context("Reading similarity")?;

    let similarity: f64 = if let Value::String(s) = similarity {
        s.parse()?
    } else {
        bail!("Couldn't parse similarit f64");
    };

    info!(?similarity);

    Ok(())
}

#[tokio::test]
async fn create_client() {
    SauceNaoClient::new("---");
}
