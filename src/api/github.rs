use anyhow::{Context, Result};
use chrono::Utc;
use octocrab::Octocrab;
use serde::Serialize;
use std::collections::HashSet;
use std::time::Duration;
use tokio::time::sleep;

use crate::api::http;
use crate::models::UserStats;

/// Implements [[RFC-0001:C-API-CLIENT]]
pub struct GitHubClient {
    octocrab: Octocrab,
    username: String,
    orgs: Vec<String>,
}

#[derive(serde::Deserialize)]
struct SearchCount {
    total_count: u64,
}

#[derive(Serialize)]
struct RepoParams {
    per_page: u32,
    page: u32,
    #[serde(rename = "type")]
    kind: String,
}

#[derive(Serialize)]
struct OrgRepoParams {
    per_page: u32,
    page: u32,
}

#[derive(Serialize)]
struct SearchParams {
    q: String,
}

impl GitHubClient {
    pub fn new(username: String, token: String, orgs: Vec<String>) -> Result<Self> {
        let octocrab = Octocrab::builder()
            .personal_token(token)
            .set_connect_timeout(Some(Duration::from_secs(http::DEFAULT_TIMEOUT_SECS)))
            .set_read_timeout(Some(Duration::from_secs(http::DEFAULT_TIMEOUT_SECS)))
            .build()
            .context("failed to build GitHub client")?;
        Ok(Self {
            octocrab,
            username,
            orgs,
        })
    }

    pub async fn fetch_stats(&self) -> Result<UserStats> {
        let username = self.username.clone();
        let profile = self
            .retry("fetch user profile", || {
                let username = username.clone();
                async move { self.octocrab.users(&username).profile().await }
            })
            .await?;

        let repos = self.fetch_all_repos().await?;
        let stars: u64 = repos
            .iter()
            .map(|r| r.stargazers_count.unwrap_or(0) as u64)
            .sum();
        let forks: u64 = repos
            .iter()
            .map(|r| r.forks_count.unwrap_or(0) as u64)
            .sum();

        let (commits, prs, issues) =
            tokio::try_join!(self.count_commits(), self.count_prs(), self.count_issues(),)?;

        let age = Utc::now().signed_duration_since(profile.created_at);

        Ok(UserStats {
            name: profile.name.clone(),
            username: self.username.clone(),
            bio: None,
            company: None,
            location: None,
            website_url: None,
            twitter_username: None,
            avatar_url: None,
            organizations: None,
            repos: repos.len() as u64,
            stars,
            forks,
            followers: profile.followers,
            commits,
            prs,
            issues,
            total_repository_contributions: None,
            restricted_contributions: None,
            contribution_years: None,
            first_issue_contribution: None,
            first_pull_request_contribution: None,
            first_repository_contribution: None,
            account_age_years: (age.num_days() / 365) as u64,
            account_age_days: age.num_days() as u64,
            // REST API doesn't fetch extended data; use GraphQL for these
            contribution_calendar: None,
            streaks: None,
            pinned_repos: None,
            time_distribution: None,
            language_usage: None,
            language_total_changes: None,
            language_sampled_commits: None,
        })
    }

    async fn fetch_all_repos(&self) -> Result<Vec<octocrab::models::Repository>> {
        let mut seen = HashSet::new();
        let mut all = Vec::new();

        for repo in self.fetch_user_repos().await? {
            if let Some(ref name) = repo.full_name
                && seen.insert(name.clone())
            {
                all.push(repo);
            }
        }

        for org in &self.orgs {
            match self.fetch_org_repos(org).await {
                Ok(repos) => {
                    for repo in repos {
                        if let Some(ref name) = repo.full_name
                            && seen.insert(name.clone())
                        {
                            all.push(repo);
                        }
                    }
                }
                Err(e) => eprintln!("warning: skipping org {org}: {e:#}"),
            }
        }

        Ok(all)
    }

    async fn fetch_user_repos(&self) -> Result<Vec<octocrab::models::Repository>> {
        let mut all = Vec::new();
        let mut page: u32 = 1;
        loop {
            let path = format!("/users/{}/repos", self.username);
            let params = RepoParams {
                per_page: 100,
                page,
                kind: "owner".to_string(),
            };
            let batch: Vec<octocrab::models::Repository> = self
                .retry(&format!("fetch user repos page {page}"), || {
                    self.octocrab.get(path.clone(), Some(&params))
                })
                .await?;
            if batch.is_empty() {
                break;
            }
            all.extend(batch);
            page += 1;
        }
        Ok(all)
    }

    async fn fetch_org_repos(&self, org: &str) -> Result<Vec<octocrab::models::Repository>> {
        let mut all = Vec::new();
        let mut page: u32 = 1;
        loop {
            let path = format!("/orgs/{org}/repos");
            let params = OrgRepoParams {
                per_page: 100,
                page,
            };
            let batch: Vec<octocrab::models::Repository> = self
                .retry(&format!("fetch {org} repos page {page}"), || {
                    self.octocrab.get(path.clone(), Some(&params))
                })
                .await?;
            if batch.is_empty() {
                break;
            }
            all.extend(batch);
            page += 1;
        }
        Ok(all)
    }

    async fn count_commits(&self) -> Result<u64> {
        let params = SearchParams {
            q: format!("author:{}", self.username),
        };
        let result: SearchCount = self
            .retry("search commits", || {
                self.octocrab.get("/search/commits", Some(&params))
            })
            .await?;
        Ok(result.total_count)
    }

    async fn count_prs(&self) -> Result<u64> {
        let query = format!("author:{} is:pr", self.username);
        let result = self
            .retry("search PRs", || {
                self.octocrab
                    .search()
                    .issues_and_pull_requests(&query)
                    .send()
            })
            .await?;
        Ok(result.total_count.unwrap_or(0) as u64)
    }

    async fn count_issues(&self) -> Result<u64> {
        let query = format!("author:{} is:issue", self.username);
        let result = self
            .retry("search issues", || {
                self.octocrab
                    .search()
                    .issues_and_pull_requests(&query)
                    .send()
            })
            .await?;
        Ok(result.total_count.unwrap_or(0) as u64)
    }

    async fn retry<T, F, Fut>(&self, context: &str, mut f: F) -> Result<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, octocrab::Error>>,
    {
        let mut attempt = 0usize;
        loop {
            match f().await {
                Ok(value) => return Ok(value),
                Err(err) if attempt < 3 => {
                    let delay = 200u64.saturating_mul(2u64.saturating_pow(attempt as u32));
                    sleep(Duration::from_millis(delay)).await;
                    attempt += 1;
                }
                Err(err) => {
                    return Err(err).with_context(|| format!("{context} failed after retries"));
                }
            }
        }
    }
}
