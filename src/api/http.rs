use anyhow::{Context, Result};
use reqwest::Client;
use reqwest::header::{ACCEPT, AUTHORIZATION, HeaderMap, HeaderValue, USER_AGENT};
use std::time::Duration;

pub const DEFAULT_TIMEOUT_SECS: u64 = 30;
pub const DEFAULT_USER_AGENT: &str = "github-readme-stats";
pub const GITHUB_ACCEPT: &str = "application/vnd.github+json";

pub fn build_github_client(token: &str) -> Result<Client> {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static(DEFAULT_USER_AGENT));
    headers.insert(ACCEPT, HeaderValue::from_static(GITHUB_ACCEPT));
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", token))
            .context("invalid GitHub token for Authorization header")?,
    );

    Client::builder()
        .default_headers(headers)
        .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
        .build()
        .context("failed to build HTTP client")
}
