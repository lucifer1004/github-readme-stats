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
github-readme-stats your-username -o stats.json
```

## Configuration file

Create `github-readme-stats.toml` in the working directory for optional settings
(token stays in `GHT` env only).

```toml
[time]
timezone = "+08:00"       # UTC offset for time distribution

[repos]
pinned = ["owner/repo1", "owner/repo2"]  # repos to fetch detailed stats for

[language]
commits_limit = 5000          # max commits to sample in total (default 5000; 0 = unlimited)
commits_per_repo = 1000       # max commits per repository (default 1000; 0 = unlimited)
repos_limit = 100             # max contributed repositories to sample (default 100; 0 = unlimited)
top_n = 10                    # top N languages to include (default 10)
exclude = ["HTML", "CSS"]     # languages to exclude (case-insensitive)
types = ["programming"]       # Linguist types to include (default: ["programming"])
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

- All data is fetched via GraphQL: profile and contributions in one query, then one query per contributed repository for commit history and language distribution.
- `language_usage` approximates per-language usage: each commit contributes a log-compressed weight (`ln(1 + additions + deletions)`) attributed across its repository's languages proportional to their byte shares (GraphQL does not expose per-commit file data). Log compression keeps large squashed commits and generated code from dominating. `changes` values are weights on this log scale, not raw line counts.
- Language filtering uses a build-time snapshot of [GitHub Linguist](https://github.com/github-linguist/linguist) `languages.yml`. To update, re-download `data/languages.yml` and rebuild.
- By default, only `programming` languages are included. Set `language.types` to `["programming", "markup"]` etc. to widen the filter. Valid types: `programming`, `data`, `markup`, `prose`.
- Time distribution and language usage share the same commit sample (`language.commits_limit`), covering the last year of contributions on default branches.

## Used by

- [github-stats-typst](https://github.com/lucifer1004/github-stats-typst) — GitHub Action to render stats as SVG cards

## License

MIT
