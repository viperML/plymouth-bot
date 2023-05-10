use color_eyre::{Result, eyre::bail, Report};
use tracing::{instrument, debug};

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
            client: reqwest::Client::new(),
        }
    }
}

impl DanbooruClient<'_> {
    #[instrument(ret, err, level = "debug")]
    pub(crate) async fn fav(&self, id: u64) -> Result<()> {
        let url = reqwest::Url::parse_with_params(
            "https://danbooru.donmai.us/favorites",
            &[("post_id", id.to_string())],
        )?;

        let req = self.client
            .post(url)
            .basic_auth(&self.username, Some(&self.apikey)).build()?;

        debug!(?req);
        let response = self.client.execute(req).await?;
        debug!(?response);

        if !response.status().is_success() {
            bail!("Bad response: {:?}", response.status());
        }

        Ok(())
    }
}
