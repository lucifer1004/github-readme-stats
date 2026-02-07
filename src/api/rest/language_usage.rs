use anyhow::{Context, Result};
use hyperpolyglot::Detection;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

use crate::api::retry::send_with_retry;
use crate::config::LanguageConfig;
use crate::models::LanguageUsage;

use super::commit_search::CommitSample;

pub(crate) async fn compute_language_usage(
    client: &reqwest::Client,
    commits: &[CommitSample],
    config: &LanguageConfig,
) -> Result<(Vec<LanguageUsage>, u64, u64)> {
    if config.commits_limit == 0 || commits.is_empty() {
        return Ok((Vec::new(), 0, 0));
    }

    let exclude: std::collections::HashSet<String> = config.exclude.iter().cloned().collect();
    let mut totals: HashMap<String, u64> = HashMap::new();
    let mut sampled = 0u64;

    for commit in commits {
        if commit.repo_full_name.is_empty() || commit.sha.is_empty() {
            continue;
        }
        let url = format!(
            "https://api.github.com/repos/{}/commits/{}",
            commit.repo_full_name, commit.sha
        );
        let response = send_with_retry(|| client.get(&url), "REST commit detail").await?;
        let detail: CommitDetail = response
            .json()
            .await
            .context("Failed to parse commit detail response")?;
        sampled += 1;

        for file in detail.files.unwrap_or_default() {
            let Some(lang) = language_for_path(&file.filename) else {
                continue;
            };
            if exclude.contains(&lang.to_lowercase()) {
                continue;
            }
            let changes = file
                .changes
                .or_else(|| Some(file.additions + file.deletions))
                .unwrap_or(0) as u64;
            if changes == 0 {
                continue;
            }
            *totals.entry(lang).or_insert(0) += changes;
        }
    }

    let total_changes: u64 = totals.values().sum();
    if total_changes == 0 {
        return Ok((Vec::new(), 0, sampled));
    }

    let mut usage: Vec<LanguageUsage> = totals
        .into_iter()
        .map(|(name, changes)| LanguageUsage {
            name,
            changes,
            percent: (changes as f64 / total_changes as f64) * 100.0,
        })
        .collect();

    usage.sort_by(|a, b| b.changes.cmp(&a.changes));
    usage.truncate(config.top_n);

    Ok((usage, total_changes, sampled))
}

fn language_for_path(path: &str) -> Option<String> {
    let detected = hyperpolyglot::detect(Path::new(path)).ok()??;
    let language = match detected {
        Detection::Filename(name)
        | Detection::Extension(name)
        | Detection::Shebang(name)
        | Detection::Heuristics(name)
        | Detection::Classifier(name) => name,
    };
    Some(language.to_string())
}

#[derive(Debug, Deserialize)]
struct CommitDetail {
    files: Option<Vec<CommitFile>>,
}

#[derive(Debug, Deserialize)]
struct CommitFile {
    filename: String,
    additions: u32,
    deletions: u32,
    changes: Option<u32>,
}
