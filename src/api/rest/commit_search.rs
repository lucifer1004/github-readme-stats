use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::api::retry::send_with_retry;

const REST_API_BASE: &str = "https://api.github.com";
const REST_COMMITS_PAGE_SIZE: usize = 100;

#[derive(Debug, Clone)]
pub(crate) struct CommitSample {
    pub repo_full_name: String,
    pub sha: String,
    pub authored_at: Option<DateTime<Utc>>,
}

/// Fetch recent commits for a user via the commit search API.
pub(crate) async fn fetch_commit_sample(
    client: &reqwest::Client,
    username: &str,
    limit: u32,
) -> Result<Vec<CommitSample>> {
    if limit == 0 {
        return Ok(Vec::new());
    }

    let mut commits = Vec::new();
    let mut page = 1u32;
    let max_pages = (limit as usize).div_ceil(REST_COMMITS_PAGE_SIZE) as u32;

    while page <= max_pages && (commits.len() as u32) < limit {
        let url = format!(
            "{}/search/commits?q=author:{}&sort=author-date&order=desc&per_page={}&page={}",
            REST_API_BASE, username, REST_COMMITS_PAGE_SIZE, page
        );

        let response = send_with_retry(|| client.get(&url), "REST commit search").await?;

        let search_result: CommitSearchResponse = response
            .json()
            .await
            .context("Failed to parse commit search response")?;

        let CommitSearchResponse { items } = search_result;
        let items_len = items.len();
        if items_len == 0 {
            break;
        }

        for item in items {
            if commits.len() as u32 >= limit {
                break;
            }
            if let (Some(repo), Some(commit), Some(sha)) = (item.repository, item.commit, item.sha)
            {
                let authored_at = commit
                    .author
                    .as_ref()
                    .and_then(|a| DateTime::parse_from_rfc3339(&a.date).ok())
                    .map(|dt| dt.with_timezone(&Utc));
                commits.push(CommitSample {
                    repo_full_name: repo.full_name,
                    sha,
                    authored_at,
                });
            }
        }

        if items_len < REST_COMMITS_PAGE_SIZE {
            break;
        }
        page += 1;
    }

    Ok(commits)
}

#[derive(Debug, Deserialize)]
struct CommitSearchResponse {
    items: Vec<CommitSearchItem>,
}

#[derive(Debug, Deserialize)]
struct CommitSearchItem {
    commit: Option<CommitInfo>,
    repository: Option<RepoInfo>,
    sha: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RepoInfo {
    full_name: String,
}

#[derive(Debug, Deserialize)]
struct CommitInfo {
    author: Option<CommitAuthor>,
}

#[derive(Debug, Deserialize)]
struct CommitAuthor {
    date: String,
}
