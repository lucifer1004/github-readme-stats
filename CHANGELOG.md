# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
