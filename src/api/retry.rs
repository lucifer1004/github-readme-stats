use anyhow::{Context, Result};
use reqwest::{StatusCode, header::HeaderMap};
use std::cmp::min;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::sleep;

const RETRY_MAX_ATTEMPTS: usize = 3;
const RETRY_BASE_DELAY_MS: u64 = 500;
const RETRY_MAX_DELAY_MS: u64 = 5000;

pub(crate) async fn send_with_retry<F>(
    mut make_request: F,
    context: &str,
) -> Result<reqwest::Response>
where
    F: FnMut() -> reqwest::RequestBuilder,
{
    let mut attempt = 0;
    loop {
        let response = make_request().send().await;
        match response {
            Ok(resp) => {
                if resp.status().is_success() {
                    return Ok(resp);
                }

                if attempt < RETRY_MAX_ATTEMPTS && should_retry(resp.status(), resp.headers()) {
                    let wait = retry_delay(attempt, resp.status(), resp.headers());
                    sleep(wait).await;
                    attempt += 1;
                    continue;
                }

                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                anyhow::bail!("{context} failed with status {status}: {body}");
            }
            Err(err) => {
                if attempt < RETRY_MAX_ATTEMPTS {
                    let wait = retry_delay(
                        attempt,
                        StatusCode::INTERNAL_SERVER_ERROR,
                        &HeaderMap::new(),
                    );
                    sleep(wait).await;
                    attempt += 1;
                    continue;
                }
                return Err(err).with_context(|| format!("{context} failed to send request"));
            }
        }
    }
}

fn should_retry(status: StatusCode, headers: &HeaderMap) -> bool {
    if status == StatusCode::TOO_MANY_REQUESTS {
        return true;
    }
    if status.is_server_error() {
        return true;
    }
    if status == StatusCode::FORBIDDEN {
        if let Some(remaining) = header_value(headers, "x-ratelimit-remaining")
            && remaining == "0" {
                return true;
            }
        if headers.get("retry-after").is_some() {
            return true;
        }
    }
    false
}

pub(crate) fn retry_delay(attempt: usize, status: StatusCode, headers: &HeaderMap) -> Duration {
    if let Some(secs) = retry_after_seconds(status, headers) {
        return Duration::from_secs(secs);
    }
    let backoff = RETRY_BASE_DELAY_MS.saturating_mul(2u64.saturating_pow(attempt as u32));
    Duration::from_millis(min(backoff, RETRY_MAX_DELAY_MS))
}

fn retry_after_seconds(status: StatusCode, headers: &HeaderMap) -> Option<u64> {
    if status == StatusCode::TOO_MANY_REQUESTS || status == StatusCode::FORBIDDEN {
        if let Some(value) = header_value(headers, "retry-after")
            && let Ok(secs) = value.parse::<u64>() {
                return Some(secs);
            }
        if let Some(value) = header_value(headers, "x-ratelimit-reset")
            && let Ok(epoch) = value.parse::<u64>() {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                if epoch > now {
                    return Some(epoch - now);
                }
            }
    }
    None
}

fn header_value(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::header::HeaderValue;

    #[test]
    fn retry_delay_prefers_retry_after() {
        let mut headers = HeaderMap::new();
        headers.insert("retry-after", HeaderValue::from_static("3"));
        let delay = retry_delay(0, StatusCode::TOO_MANY_REQUESTS, &headers);
        assert_eq!(delay.as_secs(), 3);
    }
}
