# github-readme-stats

Rust CLI to fetch GitHub user statistics as JSON.

## Installation

Download from [Releases](https://github.com/lucifer1004/github-readme-stats/releases) or build from source:

```bash
cargo install --git https://github.com/lucifer1004/github-readme-stats
```

## Usage

```bash
export GHT="ghp_your_token"

# Basic usage
github-readme-stats your-username -o stats.json

# Optional settings are now read from github-readme-stats.toml
github-readme-stats your-username -o stats.json
```

## Configuration file

Create `github-readme-stats.toml` in the working directory for optional settings
(token stays in `GHT` env only).

```toml
[time]
timezone = "+08:00"

[repos]
pinned = ["owner/repo1", "owner/repo2"]
orgs = ["my-org"]

[language]
commits_limit = 1000
top_n = 10
exclude = ["HTML", "CSS"]
```

## Environment Variables

| Variable | Required | Description                                      |
| -------- | -------- | ------------------------------------------------ |
| `GHT`    | yes      | GitHub Personal Access Token (`read:user` scope) |

## Output

JSON with the following structure:

```json
{
  "name": "Display Name",
  "username": "github-username",
  "repos": 42,
  "stars": 100,
  "forks": 25,
  "followers": 50,
  "commits": 1234,
  "prs": 56,
  "issues": 78,
  "account_age_years": 5,
  "account_age_days": 1825,
  "contribution_calendar": { ... },
  "streaks": { ... },
  "pinned_repos": [ ... ],
  "time_distribution": { ... },
  "language_usage": [ ... ],
  "language_total_changes": 12345,
  "language_sampled_commits": 1000
}
```

## Notes

- `language_usage` is derived from commit file changes (additions + deletions), not repo sizes.
- Time distribution and language usage share the same commit sample (`language.commits_limit`).
- Language stats can be rate-limit heavy because each sampled commit fetches commit details.

## Used by

- [github-stats-typst](https://github.com/lucifer1004/github-stats-typst) — GitHub Action to render stats as SVG cards

## License

MIT
