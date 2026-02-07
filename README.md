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

# With timezone for commit time distribution
export TIMEZONE="+08:00"
github-readme-stats your-username -o stats.json

# With pinned repos
export PINNED_REPOS="owner/repo1,owner/repo2"
github-readme-stats your-username -o stats.json
```

## Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `GHT` | yes | GitHub Personal Access Token (`read:user` scope) |
| `TIMEZONE` | no | Timezone offset (e.g., `+08:00`). Default: `+00:00` |
| `PINNED_REPOS` | no | Comma-separated repos to track (e.g., `owner/repo1,owner/repo2`) |

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
  "time_distribution": { ... }
}
```

## Used by

- [github-stats-typst](https://github.com/lucifer1004/github-stats-typst) — GitHub Action to render stats as SVG cards

## License

MIT
