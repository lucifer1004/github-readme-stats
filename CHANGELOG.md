# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2026-09-03

### Added

- `language.repos_limit` and `language.commits_per_repo` config keys. Sampling is now bounded per repository in addition to the global `commits_limit`; each of the three limits accepts `0` for "unlimited" (opt-in). Default `commits_limit` raised from 1000 to 5000.

### Changed

- Commit sampling, time distribution, and language stats moved from REST (commit search + per-commit detail requests) to GraphQL (`commitContributionsByRepository` + per-repo `history`/`languages`). This removes all usage of the aggressively rate-limited commit search endpoint and cuts API calls from up to ~1000 per run to one per contributed repository.
- Language stats are now approximate: a commit's log-compressed weight (`ln(1 + additions + deletions)`) is attributed across its repository's languages proportional to byte shares (GraphQL has no per-commit file data). Log compression prevents large squashed commits and generated code from dominating. `language_usage[].changes` and `language_total_changes` are now weights on a log scale, not raw line counts. Sampling covers the last year (the `contributionsCollection` window) instead of the most recent commits of all time.

### Removed

- REST code path and the `src/api/rest/` module.

## [0.2.0] - 2026-02-07

### Added

- Progress reporting via stderr at each major fetch stage, commit search pagination, and language stats processing.
- `language.types` config to control which Linguist language types to include (default: `["programming"]`).
- `data/overrides.toml` for popularity-based disambiguation of shared file extensions (e.g. `.rs` -> Rust, `.h` -> C).

### Changed

- Replaced `hyperpolyglot` with a build-time snapshot of GitHub Linguist `languages.yml` for language detection. Covers all languages Linguist knows (including Typst, Zig, etc.) and exposes `type` metadata for filtering.
- Language stats now default to programming languages only, filtering out data/markup/prose types (e.g. "Ignore List", "Text", "YAML").
- Single code path: reqwest for all HTTP (GraphQL + REST).

### Removed

- Dropped `--api rest` mode and the Octocrab dependency.
- Dropped `hyperpolyglot` dependency (stale since 2020, missing modern languages).
- Removed `repos.orgs` config key (was REST-only).

## [0.1.0] - 2026-02-07

### Added

- Initial release of the product.
