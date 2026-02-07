use serde::Serialize;

/// Implements [[RFC-0002:C-PINNED-REPOS]]
#[derive(Debug, Clone, Serialize)]
pub struct PinnedRepo {
    pub name: String,
    pub description: Option<String>,
    pub stars: u64,
    pub forks: u64,
    pub watchers: u64,
    pub issues: u64,
    pub pull_requests: u64,
    pub releases: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topics: Option<Vec<String>>,
    pub language: Option<String>,
    pub language_color: Option<String>,
    pub is_archived: bool,
    pub is_fork: bool,
    pub is_template: bool,
    pub disk_usage: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_branch: Option<String>,
    pub recent_additions: u64,
    pub recent_deletions: u64,
    pub recent_commits: u64,
    pub last_commit_date: Option<String>,
}
