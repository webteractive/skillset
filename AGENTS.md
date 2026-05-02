# Repository Guidelines

## Project Structure & Module Organization

This repository is a Rust CLI named `skillset`. The entry point is `src/main.rs`, which defines the Clap command surface and delegates work to focused modules such as `src/install.rs`, `src/skills.rs`, `src/config.rs`, `src/registry.rs`, and `src/validate.rs`. Bundled skill content lives in `skills/<skill-name>/`, with each skill keeping its `SKILL.md` and optional README together. Release and install helpers are in `release.sh` and `install.sh`; example configuration is in `config.example.json`.

## Build, Test, and Development Commands

- `cargo build` compiles the debug binary for local development.
- `cargo run -- <command>` runs the CLI from source, for example `cargo run -- list` or `cargo run -- validate`.
- `cargo test` runs Rust unit and integration tests when present.
- `cargo fmt --check` verifies standard Rust formatting; use `cargo fmt` to apply it.
- `cargo clippy --all-targets --all-features` catches common correctness and maintainability issues.
- `cargo build --release` builds the optimized release binary used by `release.sh`.

## Coding Style & Naming Conventions

Use Rust 2021 idioms and keep formatting under `rustfmt`. Prefer small command-specific functions and modules over growing `main.rs`. Use `snake_case` for functions, variables, modules, and file names; use `PascalCase` for types and enum variants. Return `anyhow::Result` for fallible CLI flows and add context to errors that cross IO, JSON, git, or filesystem boundaries. Keep user-facing CLI text short and actionable.

## Testing Guidelines

There are currently no dedicated test files, so new behavior should include focused tests near the changed module or in `tests/` when the behavior is command-level. Name tests after the behavior being protected, for example `validates_missing_description` or `sync_skips_unchanged_skill`. Before submitting changes, run `cargo test`, `cargo fmt --check`, and preferably `cargo clippy --all-targets --all-features`.

## Commit & Pull Request Guidelines

Recent commits use concise, imperative subject lines such as `Harden error handling and security across CLI` and release commits like `Release version 0.2.3`. Keep commits focused and do not include `Co-Authored-By` trailers. Pull requests should describe the user-visible change, list validation commands run, link related issues, and include terminal output or screenshots only when they clarify CLI behavior.

## Security & Configuration Tips

Do not commit personal config, credentials, or generated binaries. Treat `config.example.json` as documentation for expected settings. When changing install, sync, or remove logic, be conservative with filesystem writes and preserve `--dry-run` behavior.
