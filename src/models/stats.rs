use serde::Serialize;

use super::calendar::{ContributionCalendar, StreakStats};
use super::pinned::PinnedRepo;
use super::time_distribution::TimeDistribution;

/// Implements [[RFC-0001:C-DATA-MODEL]] and [[RFC-0002:C-GRAPHQL-CLIENT]]
#[derive(Debug, Serialize)]
pub struct UserStats {
    pub name: Option<String>,
    pub username: String,
    pub repos: u64,
    pub stars: u64,
    pub forks: u64,
    pub followers: u64,
    pub commits: u64,
    pub prs: u64,
    pub issues: u64,
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
}
