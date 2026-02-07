// Implements [[RFC-0001:C-CLI]] and [[RFC-0002:C-GRAPHQL-CLIENT]]

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use std::path::PathBuf;

mod api;
mod models;

#[derive(Debug, Clone, Copy, Default, ValueEnum)]
enum ApiMode {
    /// Use REST API (octocrab) - original behavior
    Rest,
    /// Use GraphQL API - single query, richer data
    #[default]
    Graphql,
}

#[derive(Parser)]
#[command(name = "github-stats", version, about = "Fetch GitHub stats to JSON")]
struct Cli {
    /// GitHub username
    username: String,

    /// Output JSON path
    #[arg(short, long, default_value = "stats.json")]
    output: PathBuf,

    /// API mode: rest (legacy) or graphql (default, richer data)
    #[arg(long, value_enum, default_value = "graphql")]
    api: ApiMode,
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse();

    let token =
        std::env::var("GHT").context("GHT environment variable not set (GitHub token required)")?;

    let stats = match cli.api {
        ApiMode::Rest => {
            let orgs: Vec<String> = std::env::var("ORGS")
                .unwrap_or_default()
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            let client = api::GitHubClient::new(cli.username, token, orgs)?;
            client.fetch_stats().await?
        }
        ApiMode::Graphql => {
            let pinned = std::env::var("PINNED_REPOS").ok();
            let timezone = std::env::var("TIMEZONE").ok();
            let client = api::GraphQLClient::new(token, pinned, timezone);
            client.fetch_stats(&cli.username).await?
        }
    };

    let json = serde_json::to_string_pretty(&stats).context("failed to serialize stats to JSON")?;

    std::fs::write(&cli.output, &json)
        .with_context(|| format!("failed to write {}", cli.output.display()))?;

    eprintln!("wrote {}", cli.output.display());
    Ok(())
}
