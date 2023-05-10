use std::{collections::HashMap, fmt, path::Path, borrow::Cow};

use color_eyre::{
    eyre::{Context, ContextCompat},
    Result,
};


use serde::Deserialize;
use serde_json::Value;

use tracing::{trace};

use crate::GetSauce;

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

    pub(crate) async fn tag<T: Into<Cow<'static, [u8]>> + 'static>(&self, contents: T) -> Result<SauceNaoResponse> {
        // let contents = contents.as_ref();
        let url = reqwest::Url::parse_with_params(
            "https://saucenao.com/search.php",
            &[
                ("output_type", "2"),
                ("numres", "1"),
                ("minsim", "85"),
                ("db", "9"),
                ("api_key", &self.api_key),
            ],
        )?;
        let part = reqwest::multipart::Part::bytes(contents).file_name("file");
        let form = reqwest::multipart::Form::new().part("file", part);
        let req = self.client.post(url).multipart(form).build()?;

        let response = self.client.execute(req).await?;

        let raw: Value = response.json().await?;
        trace!(?raw);
        let parsed: SauceNaoResponse = serde_json::from_value(raw)?;

        Ok(parsed)
    }
}

#[derive(Debug, Deserialize)]
#[non_exhaustive]
pub struct SauceNaoResponse {
    pub header: Value,
    pub results: Vec<SauceNaoResult>,
}

#[derive(Debug, Deserialize)]
#[non_exhaustive]
pub struct SauceNaoResult {
    pub data: Value,
    pub header: HashMap<String, Value>,
}

pub(crate) async fn similarity<P: AsRef<Path> + fmt::Debug>(
    file: P,
    tx: tokio::sync::mpsc::UnboundedSender<GetSauce>,
    rx: tokio::sync::oneshot::Receiver<f64>,
    tx2: tokio::sync::oneshot::Sender<f64>,
) -> Result<f64> {
    let file = file.as_ref();
    let contents = tokio::fs::read(file).await.context("Reading file")?;

    let msg = GetSauce {
        file_contents: contents,
        responder: tx2,
    };

    tx.send(msg)?;
    let similarity = rx.await?;

    Ok(similarity)
}
