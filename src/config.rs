use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Default, Deserialize)]
pub struct ConfigFile {
    pub time: Option<TimeConfig>,
    pub repos: Option<ReposConfig>,
    pub language: Option<LanguageConfigFile>,
}

#[derive(Debug, Default, Deserialize)]
pub struct TimeConfig {
    pub timezone: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct ReposConfig {
    pub pinned: Option<Vec<String>>,
}

#[derive(Debug, Default, Deserialize)]
pub struct LanguageConfigFile {
    pub commits_limit: Option<u32>,
    pub commits_per_repo: Option<u32>,
    pub repos_limit: Option<u32>,
    pub top_n: Option<usize>,
    pub exclude: Option<Vec<String>>,
    pub types: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct LanguageConfig {
    /// Global cap on sampled commits across all repositories. 0 = unlimited.
    pub commits_limit: u32,
    /// Cap on sampled commits per repository. 0 = unlimited.
    pub commits_per_repo: u32,
    /// Max contributed repositories to sample (GraphQL caps the list at 100).
    /// 0 = unlimited.
    pub repos_limit: u32,
    pub top_n: usize,
    pub exclude: Vec<String>,
    /// Linguist language types to include (e.g. "programming", "markup").
    /// Default: ["programming"] only.
    pub types: Vec<String>,
}

impl Default for LanguageConfig {
    fn default() -> Self {
        Self {
            commits_limit: 5000,
            commits_per_repo: 1000,
            repos_limit: 100,
            top_n: 10,
            exclude: Vec::new(),
            types: vec!["programming".to_string()],
        }
    }
}

impl LanguageConfig {
    pub fn from_file(config: Option<&LanguageConfigFile>) -> Self {
        let defaults = Self::default();
        let commits_limit = config
            .and_then(|c| c.commits_limit)
            .unwrap_or(defaults.commits_limit);
        let commits_per_repo = config
            .and_then(|c| c.commits_per_repo)
            .unwrap_or(defaults.commits_per_repo);
        let repos_limit = config
            .and_then(|c| c.repos_limit)
            .unwrap_or(defaults.repos_limit);
        let top_n = config.and_then(|c| c.top_n).unwrap_or(defaults.top_n);
        let exclude = config
            .and_then(|c| c.exclude.clone())
            .unwrap_or_default()
            .into_iter()
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect();
        let types = config
            .and_then(|c| c.types.clone())
            .unwrap_or(defaults.types)
            .into_iter()
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect();
        Self {
            commits_limit,
            commits_per_repo,
            repos_limit,
            top_n,
            exclude,
            types,
        }
    }
}

pub fn load_config(path: &Path) -> Result<Option<ConfigFile>> {
    if !path.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read config {}", path.display()))?;
    let config: ConfigFile =
        toml::from_str(&content).context("failed to parse github-readme-stats.toml")?;
    Ok(Some(config))
}
