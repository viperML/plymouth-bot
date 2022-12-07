#[derive(Debug)]
pub struct DanbooruId {
    pub id: u64, // serde uses u64
}

impl From<&serde_json::Value> for DanbooruId {
    fn from(val: &serde_json::Value) -> DanbooruId {
        DanbooruId {
            id: val.as_u64().unwrap(),
        }
    }
}

impl std::fmt::Display for DanbooruId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

#[derive(Debug)]
pub struct DanbooruClient {
    username: String,
    api_key: String,
}

impl DanbooruClient {
    pub fn new(username: &str, api_key: &str) -> DanbooruClient {
        DanbooruClient {
            username: username.to_string(),
            api_key: api_key.to_string(),
        }
    }

    fn build_request(
        &self,
        id: &DanbooruId,
        client: &reqwest::blocking::Client,
    ) -> reqwest::blocking::RequestBuilder {
        let url = reqwest::Url::parse_with_params(
            "https://danbooru.donmai.us/favorites",
            &[("post_id", id.to_string())],
        )
        .unwrap();

        client
            .post(url)
            .basic_auth(&self.username, Some(&self.api_key))
    }

    pub fn fav_post(&self, id: &DanbooruId) -> reqwest::Result<reqwest::blocking::Response> {
        let client = reqwest::blocking::Client::new();
        let request = self.build_request(id, &client);
        request.send()
    }
}
