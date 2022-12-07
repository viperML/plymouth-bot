use std::{path::Path};

use anyhow::bail;
use log::info;

use crate::danbooru::DanbooruId;

#[derive(thiserror::Error, Debug)]
pub enum SauceError {
    #[error("Didn't get any matches")]
    NoMatch,
    #[error("Timed out")]
    TimedOut,
    #[error("Other error")]
    OtherError,
}

pub struct SaucenaoClient {
    api_key: String,
    pub short_remaining: u64,
    long_remaining: u64,
}

impl SaucenaoClient {
    pub fn new(api_key: &str) -> SaucenaoClient {
        SaucenaoClient {
            api_key: api_key.to_string(),
            short_remaining: 4,
            long_remaining: 100,
        }
    }

    fn build_url(&self) -> anyhow::Result<reqwest::Url> {
        let result = reqwest::Url::parse_with_params(
            "https://saucenao.com/search.php",
            &[
                ("output_type", "2"),
                ("numres", "1"),
                ("minsim", "85"),
                ("db", "9"),
                ("api_key", &self.api_key),
            ],
        )?;
        Ok(result)
    }

    fn build_request<P: AsRef<Path>>(
        &self,
        path: P,
        client: &reqwest::blocking::Client,
    ) -> anyhow::Result<reqwest::blocking::RequestBuilder> {
        let url = self.build_url()?;
        let file_bytes = std::fs::read(&path)?;
        // let file_name = path.as_ref().file_name().unwrap().to_str().unwrap();
        let part = reqwest::blocking::multipart::Part::bytes(file_bytes).file_name("file");
        let form = reqwest::blocking::multipart::Form::new().part("file", part);
        Ok(client.post(url).multipart(form))
    }

    pub fn tag_image<P: AsRef<Path>>(
        &mut self,
        path: &P,
        minimum_similarity: f32,
    ) -> anyhow::Result<DanbooruId> {
        let client = reqwest::blocking::Client::new();
        let request = self.build_request(path, &client)?;
        let result = request.send()?;

        let decoded = result.json::<serde_json::Value>()?;

        let first_match = decoded["results"][0].clone();
        let header = decoded["header"].clone();

        let similarity = first_match
            .get("header")
            .and_then(|v| v.get("similarity"))
            .ok_or(SauceError::OtherError)?
            .as_str()
            .unwrap()
            .parse::<f32>()
            .unwrap();

        info!("Similarity: {similarity}");

        self.short_remaining = header
            .get("short_remaining")
            .ok_or(SauceError::OtherError)?
            .as_u64()
            .unwrap();

        info!("Short remaining: {}", self.short_remaining);

        self.long_remaining = header
            .get("long_remaining")
            .ok_or(SauceError::OtherError)?
            .as_u64()
            .unwrap();

        info!("Long remaining: {}", self.long_remaining);

        if self.long_remaining == 0 {
            bail!(SauceError::TimedOut);
        }

        if similarity < minimum_similarity {
            bail!(SauceError::NoMatch)
        } else {
            let id_raw = first_match
                .get("data")
                .and_then(|x| x.get("danbooru_id"))
                .ok_or(SauceError::NoMatch)?;

            let id = DanbooruId::from(id_raw);
            info!("DanbooruId: {id:?}");
            Ok(id)
        }
    }
}
