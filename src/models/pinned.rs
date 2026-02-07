use serde::Serialize;

/// Implements [[RFC-0002:C-PINNED-REPOS]]
#[derive(Debug, Clone, Serialize)]
pub struct PinnedRepo {
    pub name: String,
    pub description: Option<String>,
    pub stars: u64,
    pub forks: u64,
    pub language: Option<String>,
    pub language_color: Option<String>,
    pub recent_additions: u64,
    pub recent_deletions: u64,
    pub recent_commits: u64,
    pub last_commit_date: Option<String>,
}
