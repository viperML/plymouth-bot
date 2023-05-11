use std::{borrow::Cow, collections::HashMap, time::Duration};

use color_eyre::{
    eyre::{bail, Context, ContextCompat},
    Result,
};

use serde::Deserialize;
use serde_json::{Number, Value};

use tokio::time::sleep;
use tracing::{trace, warn};

#[derive(redacted_debug::RedactedDebug)]
pub(crate) struct SauceNaoClient {
    #[redacted]
    api_key: String,
    client: reqwest::Client,
    long_remaining: u64,
    short_remaining: u64,
    pub slow: bool,
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
            long_remaining: 100,
            short_remaining: 4,
            slow: false,
        }
    }

    pub(crate) async fn tag<T: Into<Cow<'static, [u8]>> + 'static>(
        &mut self,
        contents: T,
    ) -> Result<Match> {
        if (!self.slow && self.short_remaining == 0) || (self.slow && self.short_remaining <= 3) {
            warn!("Short limit reached, sleeping");
            // Short timeout is 30 seconds, so sleep a bit more
            let timeout = Duration::from_secs(31);
            let divisions = 100;
            let step = timeout / divisions;

            let bar = indicatif::ProgressBar::new(divisions.into());
            for _ in 0..divisions {
                bar.inc(1);
                sleep(step).await;
            }
            bar.finish();
        };
        if self.long_remaining == 0 {
            bail!("Long limit reached!");
        }

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

        let short_rem = &parsed.header["short_remaining"];
        let short_rem = serde_json::from_value::<Number>(short_rem.clone())?
            .as_u64()
            .context("FIXME")?;
        trace!(?short_rem);
        self.short_remaining = short_rem;

        let long_rem = &parsed.header["long_remaining"];
        let long_rem = serde_json::from_value::<Number>(long_rem.clone())?
            .as_u64()
            .context("FIXME")?;
        trace!(?long_rem);
        self.long_remaining = long_rem;

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
