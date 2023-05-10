use std::{borrow::Cow, collections::HashMap, fmt, path::Path};

use color_eyre::{
    eyre::{bail, Context, ContextCompat},
    Result,
};

use serde::Deserialize;
use serde_json::Value;

use tracing::trace;

use crate::GetSauce;

#[derive(redacted_debug::RedactedDebug)]
pub(crate) struct SauceNaoClient {
    api_key: String,
    client: reqwest::Client,
}

#[derive(Debug)]
pub(crate) struct Match {
    pub similarity: f64,
    pub danbooru_id: u64,
}

impl SauceNaoClient {
    pub(crate) fn new<S: AsRef<str>>(api_key: S) -> Self {
        let api_key = api_key.as_ref();

        Self {
            api_key: api_key.to_string(),
            client: reqwest::Client::new(),
        }
    }

    pub(crate) async fn tag<T: Into<Cow<'static, [u8]>> + 'static>(
        &self,
        contents: T,
    ) -> Result<Match> {
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

        let sim = &parsed.results[0].header["similarity"];
        let similarity = if let Value::String(s) = sim {
            s.parse()?
        } else {
            bail!("Similarity wasn't a string!");
        };

        let danbooru_id = if let Value::Number(n) = &parsed.results[0].data["danbooru_id"] {
            n.as_u64().context("FIXME")?
        } else {
            bail!("ID wsn't a number")
        };

        let my_match = Match {
            similarity,
            danbooru_id,
        };

        Ok(my_match)
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
