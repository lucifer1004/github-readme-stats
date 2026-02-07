use chrono::{DateTime, Utc};
use serde::Deserialize;

// ===== User query response types =====

#[derive(Debug, Deserialize)]
pub(crate) struct UserResponse {
    pub(crate) data: Option<UserDataRoot>,
    pub(crate) errors: Option<Vec<GraphQLError>>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct GraphQLError {
    pub(crate) message: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct UserDataRoot {
    pub(crate) user: Option<UserNode>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UserNode {
    pub(crate) id: String, // GitHub node ID for author filtering
    pub(crate) name: Option<String>,
    pub(crate) login: String,
    pub(crate) bio: Option<String>,
    pub(crate) company: Option<String>,
    pub(crate) location: Option<String>,
    pub(crate) website_url: Option<String>,
    pub(crate) twitter_username: Option<String>,
    pub(crate) avatar_url: Option<String>,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) followers: CountNode,
    pub(crate) organizations: CountNode,
    pub(crate) repositories: RepositoriesNode,
    pub(crate) contributions_collection: ContributionsCollectionNode,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CountNode {
    pub(crate) total_count: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RepositoriesNode {
    pub(crate) total_count: u64,
    pub(crate) nodes: Vec<RepoStatsNode>,
    pub(crate) page_info: PageInfoNode,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RepoStatsNode {
    pub(crate) stargazer_count: u64,
    pub(crate) fork_count: u64,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PageInfoNode {
    pub(crate) has_next_page: bool,
    pub(crate) end_cursor: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ContributionsCollectionNode {
    pub(crate) total_commit_contributions: u64,
    pub(crate) total_pull_request_contributions: u64,
    pub(crate) total_issue_contributions: u64,
    pub(crate) total_repository_contributions: u64,
    pub(crate) restricted_contributions_count: u64,
    pub(crate) contribution_years: Vec<i32>,
    pub(crate) first_issue_contribution: Option<FirstContributionNode>,
    pub(crate) first_pull_request_contribution: Option<FirstContributionNode>,
    pub(crate) first_repository_contribution: Option<FirstContributionNode>,
    pub(crate) contribution_calendar: CalendarNode,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FirstContributionNode {
    pub(crate) occurred_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CalendarNode {
    pub(crate) total_contributions: u64,
    pub(crate) weeks: Vec<WeekNode>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WeekNode {
    pub(crate) contribution_days: Vec<DayNode>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DayNode {
    pub(crate) date: String,
    pub(crate) contribution_count: u64,
    pub(crate) contribution_level: String,
}

// ===== Repo query response types =====

#[derive(Debug, Deserialize)]
pub(crate) struct RepoResponse {
    pub(crate) data: Option<RepoDataRoot>,
    pub(crate) errors: Option<Vec<GraphQLError>>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RepoDataRoot {
    pub(crate) repository: Option<RepoNode>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RepoNode {
    pub(crate) name: String,
    pub(crate) description: Option<String>,
    pub(crate) stargazer_count: u64,
    pub(crate) fork_count: u64,
    pub(crate) is_archived: bool,
    pub(crate) is_fork: bool,
    pub(crate) is_template: bool,
    pub(crate) disk_usage: u64,
    pub(crate) watchers: CountNode,
    pub(crate) issues: CountNode,
    pub(crate) pull_requests: CountNode,
    pub(crate) releases: CountNode,
    pub(crate) license_info: Option<LicenseNode>,
    pub(crate) repository_topics: RepositoryTopicsNode,
    pub(crate) primary_language: Option<LanguageNode>,
    pub(crate) default_branch_ref: Option<DefaultBranchRefNode>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct LanguageNode {
    pub(crate) name: String,
    pub(crate) color: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct DefaultBranchRefNode {
    pub(crate) name: String,
    pub(crate) target: CommitTarget,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LicenseNode {
    pub(crate) spdx_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RepositoryTopicsNode {
    pub(crate) nodes: Vec<RepositoryTopicNode>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RepositoryTopicNode {
    pub(crate) topic: TopicNode,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TopicNode {
    pub(crate) name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CommitTarget {
    pub(crate) history: Option<HistoryNode>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HistoryNode {
    pub(crate) nodes: Vec<CommitNode>,
    pub(crate) total_count: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CommitNode {
    pub(crate) committed_date: DateTime<Utc>,
    pub(crate) additions: u64,
    pub(crate) deletions: u64,
}
