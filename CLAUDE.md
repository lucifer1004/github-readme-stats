# Agent Development Guide

## Project summary

Rust CLI that fetches GitHub user statistics via GraphQL (profile/contributions) and REST
(commit sample, time distribution, language stats), then outputs JSON.
Single HTTP library: `reqwest`.

## Repository layout

- `src/main.rs`: CLI entry point and orchestration.
- `src/config.rs`: TOML config file parsing (`github-readme-stats.toml`).
- `src/api/mod.rs`: API module root, exports `GraphQLClient`.
- `src/api/graphql/`: GraphQL client (`client.rs`), response models (`models.rs`), retry shim (`retry.rs`).
- `src/api/rest/`: REST helpers -- `commit_search.rs`, `language_usage.rs`, `time_distribution.rs`.
- `src/api/http.rs`: shared `reqwest::Client` builder (auth headers, timeouts).
- `src/api/retry.rs`: generic retry with exponential backoff and rate-limit awareness.
- `src/api/queries/`: `.graphql` files (`user.graphql`, `repo.graphql`) embedded at build time.
- `src/models/`: JSON output models (`UserStats`, `PinnedRepo`, `LanguageUsage`, etc.).
- `data/languages.yml`: GitHub Linguist language definitions snapshot (used at build time).
- `build.rs`: build script that generates `queries.rs` and `languages.rs` from source data.
- `.github/workflows/ci.yml`: CI checks (fmt, clippy, build, test).
- `.github/workflows/release.yml`: tag-based release packaging.

## Build and test

- Format: `cargo fmt`
- Lint: `cargo clippy -- -D warnings`
- Build (release): `cargo build --release`
- Test: `cargo test`
- Local run with just: `just fetch <username>`

## Configuration

Only `GHT` (GitHub Personal Access Token) is set via environment variable.
All other settings live in `github-readme-stats.toml` in the working directory:

```toml
[time]
timezone = "+08:00"

[repos]
pinned = ["owner/repo1", "owner/repo2"]

[language]
commits_limit = 1000
top_n = 10
exclude = ["HTML", "CSS"]
types = ["programming"]
```

Example run:

```
export GHT="ghp_your_token"
cargo run -- your-username -o stats.json
```

## CLI flags

- `<username>`: GitHub username (required, positional).
- `-o, --output <path>`: output JSON path (default: `stats.json`).

## Build-time code generation

`build.rs` generates two files into `$OUT_DIR`:

- `queries.rs`: GraphQL queries from `src/api/queries/*.graphql`.
- `languages.rs`: extension-to-language lookup from `data/languages.yml` (GitHub Linguist snapshot).

If you change a `.graphql` file or update `languages.yml`, re-run `cargo build` to regenerate.
To update language definitions, re-download `data/languages.yml` from GitHub Linguist.

## Release flow

Tag `v*` to trigger `.github/workflows/release.yml`. It builds and packages
`x86_64-unknown-linux-gnu`, `x86_64-apple-darwin`, and `aarch64-apple-darwin`.

## Notes for agents

- Keep JSON output backward compatible.
- Prefer straightforward data transformations and avoid special cases.
- `reqwest` is the sole HTTP library -- no Octocrab.
- Progress output goes to stderr via `eprintln!`.
- Update `README.md` and `CHANGELOG.md` if you change config keys or CLI flags.
