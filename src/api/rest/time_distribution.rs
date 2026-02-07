use anyhow::{Context, Result};
use chrono::{DateTime, Datelike, FixedOffset, Timelike};
use serde::Deserialize;

use crate::api::retry::send_with_retry;
use crate::models::TimeDistribution;

const REST_API_BASE: &str = "https://api.github.com";
const REST_COMMITS_PAGE_SIZE: usize = 100;
const REST_COMMITS_MAX_PAGES: u32 = 10;

/// Fetch commit time distribution using REST API search
/// Returns a 24×7 grid of commit counts by hour and weekday
pub(crate) async fn fetch_time_distribution(
    client: &reqwest::Client,
    username: &str,
    timezone_offset: FixedOffset,
) -> Result<TimeDistribution> {
    let tz_str = format!(
        "{:+03}:{:02}",
        timezone_offset.local_minus_utc() / 3600,
        (timezone_offset.local_minus_utc().abs() % 3600) / 60
    );
    let mut distribution = TimeDistribution::new(tz_str);

    // Track earliest and latest commit dates
    let mut earliest: Option<DateTime<FixedOffset>> = None;
    let mut latest: Option<DateTime<FixedOffset>> = None;

    // Fetch up to REST_COMMITS_MAX_PAGES * REST_COMMITS_PAGE_SIZE commits
    for page in 1..=REST_COMMITS_MAX_PAGES {
        let url = format!(
            "{}/search/commits?q=author:{}&sort=author-date&order=desc&per_page={}&page={}",
            REST_API_BASE, username, REST_COMMITS_PAGE_SIZE, page
        );

        let response = send_with_retry(|| client.get(&url), "REST commit search").await?;

        let search_result: CommitSearchResponse = response
            .json()
            .await
            .context("Failed to parse commit search response")?;

        if search_result.items.is_empty() {
            break; // No more results
        }

        for item in &search_result.items {
            if let Some(ref commit) = item.commit
                && let Some(ref author) = commit.author
            {
                // Parse the date and convert to user's timezone
                if let Ok(utc_time) = DateTime::parse_from_rfc3339(&author.date) {
                    let local_time = utc_time.with_timezone(&timezone_offset);
                    let hour = local_time.hour() as u8;
                    // weekday(): Mon=0, Tue=1, ..., Sun=6
                    let weekday = local_time.weekday().num_days_from_monday() as u8;
                    distribution.add(hour, weekday);

                    // Track date range
                    match earliest {
                        None => earliest = Some(local_time),
                        Some(e) if local_time < e => earliest = Some(local_time),
                        _ => {}
                    }
                    match latest {
                        None => latest = Some(local_time),
                        Some(l) if local_time > l => latest = Some(local_time),
                        _ => {}
                    }
                }
            }
        }

        // If we got fewer than REST_COMMITS_PAGE_SIZE, we've reached the end
        if search_result.items.len() < REST_COMMITS_PAGE_SIZE {
            break;
        }
    }

    // Set date range
    distribution.earliest_date = earliest.map(|d| d.format("%Y-%m-%d").to_string());
    distribution.latest_date = latest.map(|d| d.format("%Y-%m-%d").to_string());

    distribution.finalize();
    Ok(distribution)
}

// REST API response types for commit search
#[derive(Debug, Deserialize)]
struct CommitSearchResponse {
    items: Vec<CommitSearchItem>,
}

#[derive(Debug, Deserialize)]
struct CommitSearchItem {
    commit: Option<CommitInfo>,
}

#[derive(Debug, Deserialize)]
struct CommitInfo {
    author: Option<CommitAuthor>,
}

#[derive(Debug, Deserialize)]
struct CommitAuthor {
    date: String,
}
