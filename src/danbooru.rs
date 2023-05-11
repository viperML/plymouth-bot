use color_eyre::{eyre::bail, Result};

use tracing::{instrument, trace};

use crate::Folders;

#[derive(redacted_debug::RedactedDebug)]
pub(crate) struct DanbooruClient<'a> {
    #[redacted]
    pub(crate) username: &'a str,
    #[redacted]
    pub(crate) apikey: &'a str,
    client: reqwest::Client,
}

impl<'a> DanbooruClient<'a> {
    pub(crate) fn new(username: &'a str, apikey: &'a str) -> Self {
        Self {
            username,
            apikey,
            client: reqwest::ClientBuilder::new()
                .user_agent("github.com/viperML/plymouth-bot")
                .build()
                .unwrap(),
        }
    }
}

impl DanbooruClient<'_> {
    #[instrument(ret, err, level = "debug", skip(self))]
    pub(crate) async fn fav(&self, id: u64) -> Result<()> {
        let url = reqwest::Url::parse_with_params(
            "https://danbooru.donmai.us/favorites",
            &[("post_id", id.to_string())],
        )?;

        let req = self
            .client
            .post(url)
            .basic_auth(&self.username, Some(&self.apikey))
            .header(reqwest::header::CONTENT_TYPE, "application/json");

        let response = req.send().await?;
        let status = response.status();
        let text = response.text().await?;
        trace!(?text);

        if !status.is_success() {
            bail!("Bad response: {:?}", status);
        }

        Ok(())
    }
}
