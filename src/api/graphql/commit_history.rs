use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use std::collections::HashSet;

use super::client::{COMMITS_QUERY, GRAPHQL_ENDPOINT, GraphQLRequest};
use super::models::CommitsResponse;
use super::retry::send_with_retry;
use crate::config::LanguageConfig;
use crate::models::LanguageUsage;

// Generated from data/languages.yml at build time
include!(concat!(env!("OUT_DIR"), "/languages.rs"));

#[derive(Debug)]
pub(crate) struct SampledCommit {
    pub authored_date: DateTime<Utc>,
    pub additions: u64,
    pub deletions: u64,
}

/// Commit history of a single repository, paired with the repository's
/// language byte distribution (language name -> bytes).
#[derive(Debug)]
pub(crate) struct RepoHistory {
    pub languages: Vec<(String, u64)>,
    pub commits: Vec<SampledCommit>,
}

/// Fetch the user's commit history for each contributed repository via GraphQL.
/// Repositories are queried sequentially, most-contributed first, honoring
/// `repos_limit`, `commits_per_repo`, and `commits_limit` from the config
/// (0 means unlimited for each). Per-repo failures are logged and skipped.
pub(crate) async fn fetch_commit_history(
    client: &reqwest::Client,
    repos: &[(String, String)],
    author_id: &str,
    since: &DateTime<Utc>,
    config: &LanguageConfig,
) -> Vec<RepoHistory> {
    let repos_limit = limit_or_unlimited(config.repos_limit);
    let per_repo_limit = limit_or_unlimited(config.commits_per_repo);
    let mut remaining = limit_or_unlimited(config.commits_limit);

    let mut histories = Vec::new();

    for (owner, name) in repos.iter().take(repos_limit) {
        if remaining == 0 {
            break;
        }
        let repo_limit = per_repo_limit.min(remaining);
        match fetch_repo_history(client, owner, name, author_id, since, repo_limit).await {
            Ok(Some(history)) => {
                eprintln!(
                    "  commit history: {owner}/{name}: {} commits",
                    history.commits.len()
                );
                remaining = remaining.saturating_sub(history.commits.len());
                histories.push(history);
            }
            Ok(None) => {}
            Err(e) => {
                eprintln!("warning: failed to fetch commit history for {owner}/{name}: {e:#}")
            }
        }
    }

    histories
}

fn limit_or_unlimited(limit: u32) -> usize {
    if limit == 0 {
        usize::MAX
    } else {
        limit as usize
    }
}

/// Fetch one repository's languages and paged commit history (default branch,
/// authored by the user, since `since`), at most `limit` commits.
async fn fetch_repo_history(
    client: &reqwest::Client,
    owner: &str,
    name: &str,
    author_id: &str,
    since: &DateTime<Utc>,
    limit: usize,
) -> Result<Option<RepoHistory>> {
    let mut after: Option<String> = None;
    let mut languages: Option<Vec<(String, u64)>> = None;
    let mut commits = Vec::new();

    loop {
        let variables = serde_json::json!({
            "owner": owner,
            "name": name,
            "since": since.to_rfc3339(),
            "authorId": author_id,
            "after": after,
        });
        let request = GraphQLRequest {
            query: COMMITS_QUERY,
            variables,
        };

        let response = send_with_retry(
            || client.post(GRAPHQL_ENDPOINT).json(&request),
            "GraphQL commit history request",
        )
        .await?;

        let gql_response: CommitsResponse = response
            .json()
            .await
            .context("Failed to parse commit history GraphQL response")?;

        if let Some(errors) = gql_response.errors {
            let msgs: Vec<_> = errors.iter().map(|e| e.message.as_str()).collect();
            anyhow::bail!("GraphQL errors: {}", msgs.join(", "));
        }

        let Some(repo) = gql_response.data.and_then(|d| d.repository) else {
            return Ok(None);
        };

        if languages.is_none() {
            languages = Some(
                repo.languages
                    .edges
                    .iter()
                    .map(|e| (e.node.name.clone(), e.size))
                    .collect(),
            );
        }

        let Some(history) = repo
            .default_branch_ref
            .and_then(|branch| branch.target.history)
        else {
            break;
        };

        let page_info = history.page_info;
        for node in history.nodes {
            if commits.len() >= limit {
                break;
            }
            commits.push(SampledCommit {
                authored_date: node.authored_date,
                additions: node.additions,
                deletions: node.deletions,
            });
        }

        if commits.len() >= limit || !page_info.has_next_page {
            break;
        }
        after = page_info.end_cursor;
        if after.is_none() {
            break;
        }
    }

    Ok(Some(RepoHistory {
        languages: languages.unwrap_or_default(),
        commits,
    }))
}

/// Compute language usage by attributing each commit's log-compressed weight
/// (`ln(1 + additions + deletions)`) across its repository's languages,
/// proportional to each language's byte share. Log compression keeps large
/// squashed commits and generated-code changes from dominating, while
/// preserving ordering. Returns (usage, total weight, commits sampled).
///
/// Note: `LanguageUsage.changes` and the returned total are weights on a
/// log scale, not raw line counts.
pub(crate) fn compute_language_usage(
    histories: &[RepoHistory],
    config: &LanguageConfig,
) -> (Vec<LanguageUsage>, u64, u64) {
    if histories.is_empty() {
        return (Vec::new(), 0, 0);
    }

    let exclude: HashSet<&str> = config.exclude.iter().map(String::as_str).collect();
    let allowed_types: HashSet<&str> = config.types.iter().map(String::as_str).collect();
    let mut totals: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
    let mut sampled = 0u64;

    for history in histories {
        let filtered: Vec<(&str, u64)> = history
            .languages
            .iter()
            .filter(|(name, _)| {
                if exclude.contains(name.to_lowercase().as_str()) {
                    return false;
                }
                language_type(&name.to_lowercase())
                    .is_some_and(|lang_type| allowed_types.contains(lang_type))
            })
            .map(|(name, size)| (name.as_str(), *size))
            .collect();
        let total_size: u64 = filtered.iter().map(|(_, size)| size).sum();

        for commit in &history.commits {
            sampled += 1;
            let changes = commit.additions + commit.deletions;
            if changes == 0 || total_size == 0 {
                continue;
            }
            // Log compression: a 5000-line squash merge counts ~8.5,
            // a 100-line commit ~4.6, a 5-line fix ~1.8.
            let weight = (changes as f64 + 1.0).ln();
            for (name, size) in &filtered {
                *totals.entry((*name).to_string()).or_insert(0.0) +=
                    weight * (*size as f64 / total_size as f64);
            }
        }
    }

    let rounded: Vec<(String, u64)> = totals
        .into_iter()
        .map(|(name, changes)| (name, changes.round() as u64))
        .collect();
    let total_changes: u64 = rounded.iter().map(|(_, changes)| changes).sum();
    if total_changes == 0 {
        return (Vec::new(), 0, sampled);
    }

    let mut usage: Vec<LanguageUsage> = rounded
        .into_iter()
        .map(|(name, changes)| LanguageUsage {
            name,
            changes,
            percent: (changes as f64 / total_changes as f64) * 100.0,
        })
        .collect();

    usage.sort_by_key(|u| std::cmp::Reverse(u.changes));
    usage.truncate(config.top_n);

    (usage, total_changes, sampled)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn commit(additions: u64, deletions: u64) -> SampledCommit {
        SampledCommit {
            authored_date: DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            additions,
            deletions,
        }
    }

    #[test]
    fn single_language_repo_attributes_all_weight() {
        let histories = vec![RepoHistory {
            languages: vec![("Rust".to_string(), 1000)],
            commits: vec![commit(60, 40)],
        }];
        let (usage, total, sampled) =
            compute_language_usage(&histories, &LanguageConfig::default());
        // weight = ln(1 + 100) ≈ 4.62, rounds to 5
        assert_eq!(sampled, 1);
        assert_eq!(total, 5);
        assert_eq!(usage.len(), 1);
        assert_eq!(usage[0].name, "Rust");
        assert_eq!(usage[0].changes, 5);
        assert!((usage[0].percent - 100.0).abs() < 1e-9);
    }

    #[test]
    fn weight_is_split_proportionally_to_language_bytes() {
        let histories = vec![RepoHistory {
            languages: vec![("Rust".to_string(), 750), ("Python".to_string(), 250)],
            commits: vec![commit(100, 0)],
        }];
        let (usage, total, _) = compute_language_usage(&histories, &LanguageConfig::default());
        // ln(101) ≈ 4.62 -> Rust 3.46 -> 3, Python 1.15 -> 1
        assert_eq!(total, 4);
        assert_eq!(usage[0].name, "Rust");
        assert_eq!(usage[0].changes, 3);
        assert_eq!(usage[1].name, "Python");
        assert_eq!(usage[1].changes, 1);
    }

    #[test]
    fn log_weight_compresses_large_commits() {
        // One 5000-line squash merge vs. eight 5-line fixes in another repo.
        let histories = vec![
            RepoHistory {
                languages: vec![("Rust".to_string(), 1000)],
                commits: vec![commit(5000, 0)],
            },
            RepoHistory {
                languages: vec![("Python".to_string(), 1000)],
                commits: (0..8).map(|_| commit(5, 0)).collect(),
            },
        ];
        let (usage, _, _) = compute_language_usage(&histories, &LanguageConfig::default());
        // ln(5001) ≈ 8.52 -> 9,  8 * ln(6) ≈ 14.33 -> 14
        assert_eq!(usage[0].name, "Python");
        assert_eq!(usage[0].changes, 14);
        assert_eq!(usage[1].name, "Rust");
        assert_eq!(usage[1].changes, 9);
    }

    #[test]
    fn excluded_and_non_programming_languages_are_filtered_before_split() {
        let histories = vec![RepoHistory {
            languages: vec![
                ("Rust".to_string(), 500),
                ("Markdown".to_string(), 500), // prose type, filtered by default
            ],
            commits: vec![commit(100, 0)],
        }];
        let (usage, total, _) = compute_language_usage(&histories, &LanguageConfig::default());
        assert_eq!(total, 5);
        assert_eq!(usage.len(), 1);
        assert_eq!(usage[0].name, "Rust");
    }

    #[test]
    fn exclude_list_applies_case_insensitively() {
        let config = LanguageConfig {
            exclude: vec!["rust".to_string()],
            ..Default::default()
        };
        let histories = vec![RepoHistory {
            languages: vec![("Rust".to_string(), 900), ("Python".to_string(), 100)],
            commits: vec![commit(100, 0)],
        }];
        let (usage, _, _) = compute_language_usage(&histories, &config);
        assert_eq!(usage.len(), 1);
        assert_eq!(usage[0].name, "Python");
        assert_eq!(usage[0].changes, 5);
    }

    #[test]
    fn zero_change_commits_are_counted_but_not_attributed() {
        let histories = vec![RepoHistory {
            languages: vec![("Rust".to_string(), 1000)],
            commits: vec![commit(0, 0)],
        }];
        let (usage, total, sampled) =
            compute_language_usage(&histories, &LanguageConfig::default());
        assert_eq!(sampled, 1);
        assert_eq!(total, 0);
        assert!(usage.is_empty());
    }

    #[test]
    fn top_n_truncates_results() {
        let config = LanguageConfig {
            top_n: 1,
            ..Default::default()
        };
        let histories = vec![RepoHistory {
            languages: vec![("Rust".to_string(), 500), ("Python".to_string(), 500)],
            commits: vec![commit(100, 0)],
        }];
        let (usage, _, _) = compute_language_usage(&histories, &config);
        assert_eq!(usage.len(), 1);
    }
}
