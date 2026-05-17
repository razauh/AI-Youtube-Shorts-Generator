use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use std::time::Duration;

const APP_USER_AGENT: &str = "ai-youtube-shorts-generator/1.0";

pub fn default_http_client() -> Result<reqwest::Client, String> {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static(APP_USER_AGENT));
    reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(15))
        .timeout(Duration::from_secs(120))
        .default_headers(headers)
        .build()
        .map_err(|e| e.to_string())
}
