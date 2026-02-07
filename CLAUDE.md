# Agent Development Guide

## Project summary

Rust CLI that fetches GitHub user statistics via GraphQL and outputs JSON.

## Repository layout

- `src/main.rs`: CLI entry point and orchestration.
- `src/api/`: GitHub API clients and GraphQL queries.
- `src/models/`: JSON output models and helpers.
- `build.rs`: build script for GraphQL query assets.
- `.github/workflows/ci.yml`: CI checks (fmt, clippy, build, test).
- `.github/workflows/release.yml`: tag-based release packaging.

## Build and test

- Format: `cargo fmt`
- Lint: `cargo clippy -- -D warnings`
- Build (release): `cargo build --release`
- Test: `cargo test`

## Running locally

Set required environment variables, then run the binary:

- `GHT`: GitHub Personal Access Token with `read:user` scope.
- `TIMEZONE`: optional, e.g. `+08:00` (defaults to `+00:00`).
- `PINNED_REPOS`: optional, comma-separated `owner/repo`.
- `ORGS`: optional, comma-separated orgs (REST mode only).

Example:

```
export GHT="ghp_your_token"
export TIMEZONE="+08:00"
export PINNED_REPOS="owner/repo1,owner/repo2"
cargo run -- your-username -o stats.json
```

## CLI flags

- `--api graphql|rest`: GraphQL is default and includes richer data. REST is legacy and omits
  `contribution_calendar`, `streaks`, `pinned_repos`, and `time_distribution`.
- `-o, --output <path>`: output JSON path.

## GraphQL query assets

`.graphql` files live in `src/api/queries/` and are embedded at build time by `build.rs`.
If you change a query, re-run `cargo build` so `queries.rs` is regenerated.

## Release flow

Tag `v*` to trigger `.github/workflows/release.yml`. It builds and packages
`x86_64-unknown-linux-gnu`, `x86_64-apple-darwin`, and `aarch64-apple-darwin`.

## Notes for agents

- Keep JSON output backward compatible.
- Prefer straightforward data transformations and avoid special cases.
- Update docs if you add or change environment variables.
