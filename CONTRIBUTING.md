# Contributing to pyl

## Prerequisites

- Rust 1.70+
- Windows 10/11 (registry features only work on Windows)

## Setup

```bat
git clone https://github.com/cumulus13/pyl
cd pyl
cargo build
```

## Before submitting a PR

```bat
cargo fmt --all
cargo clippy --all-features -- -D warnings
cargo build --release
.\target\release\pyl.exe -0
```

All four must pass clean (zero warnings, zero errors).

## Release checklist (maintainer)

1. Update `version` in `Cargo.toml`
2. Add entry to `CHANGELOG.md`
3. Commit: `git commit -m "chore: release v0.x.0"`
4. Tag: `git tag v0.x.0`
5. Push tag: `git push origin v0.x.0`

The `release.yml` workflow handles the rest — builds the binary, creates the GitHub Release, and publishes to crates.io automatically.

## Secrets required (repo settings → Secrets → Actions)

| Secret | Description |
|---|---|
| `CARGO_REGISTRY_TOKEN` | API token from [crates.io](https://crates.io/settings/tokens) |
