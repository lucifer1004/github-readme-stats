use serde::Serialize;

use super::calendar::{ContributionCalendar, StreakStats};
use super::language::LanguageUsage;
use super::pinned::PinnedRepo;
use super::time_distribution::TimeDistribution;

/// Implements [[RFC-0001:C-DATA-MODEL]] and [[RFC-0002:C-GRAPHQL-CLIENT]]
#[derive(Debug, Serialize)]
pub struct UserStats {
    pub name: Option<String>,
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bio: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub twitter_username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organizations: Option<u64>,
    pub repos: u64,
    pub stars: u64,
    pub forks: u64,
    pub followers: u64,
    pub commits: u64,
    pub prs: u64,
    pub issues: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_repository_contributions: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restricted_contributions: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contribution_years: Option<Vec<i32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_issue_contribution: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_pull_request_contribution: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_repository_contribution: Option<String>,
    pub account_age_years: u64,
    pub account_age_days: u64,

    // RFC-0002 additions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contribution_calendar: Option<ContributionCalendar>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub streaks: Option<StreakStats>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pinned_repos: Option<Vec<PinnedRepo>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_distribution: Option<TimeDistribution>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language_usage: Option<Vec<LanguageUsage>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language_total_changes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language_sampled_commits: Option<u64>,
}
