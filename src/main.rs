// Implements [[RFC-0001:C-CLI]] and [[RFC-0002:C-GRAPHQL-CLIENT]]

use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;

mod api;
mod config;
mod models;

#[derive(Parser)]
#[command(name = "github-stats", version, about = "Fetch GitHub stats to JSON")]
struct Cli {
    /// GitHub username
    username: String,

    /// Output JSON path
    #[arg(short, long, default_value = "stats.json")]
    output: PathBuf,
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

    let config_path = std::path::Path::new("github-readme-stats.toml");
    let config = config::load_config(config_path)?;

    let pinned = config
        .as_ref()
        .and_then(|c| c.repos.as_ref())
        .and_then(|r| r.pinned.clone())
        .map(|items| items.join(","));
    let timezone = config
        .as_ref()
        .and_then(|c| c.time.as_ref())
        .and_then(|t| t.timezone.clone());
    let language_config =
        config::LanguageConfig::from_file(config.as_ref().and_then(|c| c.language.as_ref()));

    let client =
        api::GraphQLClient::new(token, pinned, timezone).with_language_config(language_config);
    let stats = client.fetch_stats(&cli.username).await?;

    let json = serde_json::to_string_pretty(&stats).context("failed to serialize stats to JSON")?;

    std::fs::write(&cli.output, &json)
        .with_context(|| format!("failed to write {}", cli.output.display()))?;

    eprintln!("wrote {}", cli.output.display());
    Ok(())
}
