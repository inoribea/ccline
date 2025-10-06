# CODEX.md

This guide helps Codex CLI agents understand the CCometixLine repository and contribute effectively.

## Project Overview

CCometixLine is a Rust-based statusline generator that works with both Claude Code and Codex. It produces a single-line, ANSI-coloured summary containing the current model, working directory, Git state, transcript usage, cost information, and burn-rate indicators. The binary is designed to be fast (<50 ms startup) and lightweight (<10 MB RSS).

## Development Commands

```bash
# Build in debug mode
cargo build

# Build release binary
cargo build --release

# Run the CLI
cargo run -- --help

# Print default configuration
echo '{}' | cargo run -- --print-config

# Run unit tests
cargo test

# Lints & formatting
cargo fmt --check
cargo clippy --all-targets
```

## Key Modules

- `src/main.rs` – CLI entry point; parses arguments and wires providers.
- `src/cli.rs` – clap definitions for command-line options.
- `src/config/`
  - `types.rs` – Core models (`InputData`, provider detection, usage structs).
  - `block_overrides.rs` – Manual billing block overrides using `resolve_config_dir`.
  - `defaults.rs` / `loader.rs` – Default configuration scaffolding.
- `src/core/`
  - `statusline.rs` – Assembles enabled segments.
  - `segments/` – Individual statusline segments (model, git, usage, cost, burn-rate, update, directory).
- `src/utils/`
  - `data_loader.rs` – Scans Claude & Codex transcript roots, deduplicates entries.
  - `transcript.rs` – Normalizes transcript lines (Claude `assistant` vs Codex `token_count`).
  - `paths.rs` – Resolves shared config directory across providers.
- `src/billing/` – Cost calculations, pricing, and burn-rate metrics.
- `src/updater.rs` – GitHub release polling; stores state under `resolve_config_dir()`.

## Provider Support Notes

- Provider detection uses transcript path heuristics (`/.claude/`, `/.codex/`) and model identifiers.
- Usage parsing simultaneously handles Claude JSONL (`assistant` messages) and Codex `event_msg` token counts.
- Environment variables:
  - `CLAUDE_CONFIG_DIR`, `CODEX_SESSIONS_DIR` – extra transcript directories.
  - `CCLINE_CONFIG_HOME` – overrides config storage path for both providers.
  - `CCLINE_DISABLE_COST`, `CCLINE_SHOW_TIMING` – feature flags.

## Data Flow

1. Codex/Claude streams a JSON payload to stdin.
2. `InputData::from_reader` normalizes provider, workspace, transcript path, and model metadata.
3. `StatusLineGenerator` builds segment list from the configuration.
4. Each segment pulls additional context (Git commands, transcript statistics, pricing data).
5. Segments render coloured strings; the generator joins them with separators and prints to stdout.

## Testing Recommendations

- Segment render tests (model formatting, usage percentage, git status).
- Transcript parsing edge cases: missing usage, Codex reasoning tokens, duplicated events.
- Billing calculator totals using mocked pricing maps.
- Block override parsing and error handling.
- CLI argument parsing (esp. block management flags).

## Release Checklist

1. Update `Cargo.toml` version.
2. Update `CHANGELOG.md` with release notes.
3. `cargo test`, `cargo clippy`, `cargo fmt --check`.
4. Build release binaries (`cargo build --release`).
5. Package archives and publish GitHub release assets.
6. Create annotated git tag (e.g., `git tag -a v1.0.0 -m "Release 1.0.0"`).

## Troubleshooting

- **No cost segment** – ensure transcripts exist and `CCLINE_DISABLE_COST` is not set.
- **No git status** – run inside a git repository; requires Git 1.5+ on PATH.
- **Icons garbled** – configure terminal to use Nerd Fonts.
- **Codex transcripts not detected** – set `CODEX_SESSIONS_DIR` to the correct session root.

## Useful Commands

```bash
# Tail Codex sessions interactively
find ~/.codex/sessions -name '*.jsonl' -print | tail

# Preview latest statusline with sample payload
echo '{"model":"gpt-5-codex","workspace":{"cwd":"/tmp/project"},"transcriptPath":"/home/user/.codex/sessions/demo.jsonl"}' | cargo run
```

Happy hacking! Codex agents should follow the same coding conventions enforced for Claude and keep the documentation in both `README.md` and `README.zh.md` aligned.
