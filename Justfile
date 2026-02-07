set dotenv-load

username := "lucifer1004"

# =============================================================================
# Build & Development
# =============================================================================

[unix]
pre-commit:
    @if command -v prek > /dev/null 2>&1; then prek run --all-files; else pre-commit run --all-files; fi

[windows]
pre-commit:
    if (Get-Command prek -ErrorAction SilentlyContinue) { prek run --all-files } else { pre-commit run --all-files }

# Build the release binary
build:
    cargo build --release

# Fetch GitHub stats to stats.json
fetch: build
    ./target/release/github-readme-stats {{username}} --output stats.json

# Lint and check
check:
    cargo fmt --check
    cargo clippy -- -D warnings
    cargo check
