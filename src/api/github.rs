use anyhow::{Context, Result};
use chrono::Utc;
use octocrab::Octocrab;
use std::collections::HashSet;

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

impl GitHubClient {
    pub fn new(username: String, token: String, orgs: Vec<String>) -> Result<Self> {
        let octocrab = Octocrab::builder()
            .personal_token(token)
            .build()
            .context("failed to build GitHub client")?;
        Ok(Self {
            octocrab,
            username,
            orgs,
        })
    }

    pub async fn fetch_stats(&self) -> Result<UserStats> {
        let profile = self
            .octocrab
            .users(&self.username)
            .profile()
            .await
            .context("failed to fetch user profile")?;

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
            let batch: Vec<octocrab::models::Repository> = self
                .octocrab
                .get(
                    format!("/users/{}/repos", self.username),
                    Some(&[
                        ("per_page", "100"),
                        ("page", &page.to_string()),
                        ("type", "owner"),
                    ]),
                )
                .await
                .with_context(|| format!("failed to fetch user repos page {page}"))?;
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
            let batch: Vec<octocrab::models::Repository> = self
                .octocrab
                .get(
                    format!("/orgs/{org}/repos"),
                    Some(&[("per_page", "100"), ("page", &page.to_string())]),
                )
                .await
                .with_context(|| format!("failed to fetch {org} repos page {page}"))?;
            if batch.is_empty() {
                break;
            }
            all.extend(batch);
            page += 1;
        }
        Ok(all)
    }

    async fn count_commits(&self) -> Result<u64> {
        let result: SearchCount = self
            .octocrab
            .get(
                "/search/commits",
                Some(&[("q", &format!("author:{}", self.username))]),
            )
            .await
            .context("failed to search commits")?;
        Ok(result.total_count)
    }

    async fn count_prs(&self) -> Result<u64> {
        let result = self
            .octocrab
            .search()
            .issues_and_pull_requests(&format!("author:{} is:pr", self.username))
            .send()
            .await
            .context("failed to search PRs")?;
        Ok(result.total_count.unwrap_or(0) as u64)
    }

    async fn count_issues(&self) -> Result<u64> {
        let result = self
            .octocrab
            .search()
            .issues_and_pull_requests(&format!("author:{} is:issue", self.username))
            .send()
            .await
            .context("failed to search issues")?;
        Ok(result.total_count.unwrap_or(0) as u64)
    }
}
