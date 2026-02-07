use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

use crate::api::retry::send_with_retry;
use crate::config::LanguageConfig;
use crate::models::LanguageUsage;

use super::commit_search::CommitSample;

// Generated from data/languages.yml at build time
include!(concat!(env!("OUT_DIR"), "/languages.rs"));

pub(crate) async fn compute_language_usage(
    client: &reqwest::Client,
    commits: &[CommitSample],
    config: &LanguageConfig,
) -> Result<(Vec<LanguageUsage>, u64, u64)> {
    if config.commits_limit == 0 || commits.is_empty() {
        return Ok((Vec::new(), 0, 0));
    }

    let exclude: HashSet<String> = config.exclude.iter().cloned().collect();
    let allowed_types: HashSet<String> = config.types.iter().cloned().collect();
    let mut totals: HashMap<&'static str, u64> = HashMap::new();
    let mut sampled = 0u64;
    let mut failures = 0u64;
    let total = commits.len();
    let mut processed = 0usize;

    for commit in commits {
        if commit.repo_full_name.is_empty() || commit.sha.is_empty() {
            processed += 1;
            continue;
        }
        let url = format!(
            "https://api.github.com/repos/{}/commits/{}",
            commit.repo_full_name, commit.sha
        );
        let response = match send_with_retry(|| client.get(&url), "REST commit detail").await {
            Ok(response) => response,
            Err(_e) => {
                failures += 1;
                processed += 1;
                continue;
            }
        };
        let detail: CommitDetail = match response
            .json()
            .await
            .context("Failed to parse commit detail response")
        {
            Ok(detail) => detail,
            Err(_e) => {
                failures += 1;
                processed += 1;
                continue;
            }
        };
        sampled += 1;
        processed += 1;

        if processed.is_multiple_of(50) || processed == total {
            eprintln!("  language stats: {processed}/{total} commits processed");
        }

        for file in detail.files.unwrap_or_default() {
            let Some((lang, lang_type)) = language_for_path(&file.filename) else {
                continue;
            };
            if !allowed_types.contains(&lang_type.to_lowercase()) {
                continue;
            }
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

    eprintln!("  language stats: done ({sampled} sampled, {failures} skipped)");

    let total_changes: u64 = totals.values().sum();
    if total_changes == 0 {
        return Ok((Vec::new(), 0, sampled));
    }

    let mut usage: Vec<LanguageUsage> = totals
        .into_iter()
        .map(|(name, changes)| LanguageUsage {
            name: name.to_string(),
            changes,
            percent: (changes as f64 / total_changes as f64) * 100.0,
        })
        .collect();

    usage.sort_by(|a, b| b.changes.cmp(&a.changes));
    usage.truncate(config.top_n);

    Ok((usage, total_changes, sampled))
}

/// Returns (language_name, language_type) for a file path using the Linguist extension map.
fn language_for_path(path: &str) -> Option<(&'static str, &'static str)> {
    let ext = path.rsplit('.').next()?;
    let ext_lower = ext.to_lowercase();
    extension_to_language(&ext_lower)
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
