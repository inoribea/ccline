# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

CCometixLine is a high-performance Claude Code statusline tool written in Rust that provides real-time usage tracking, Git integration, cost monitoring, and burn rate analysis. The tool integrates with Claude Code's statusline system to display comprehensive development information.

## Development Commands

### Building and Testing
```bash
# Build development version
cargo build

# Build optimized release version
cargo build --release

# Run tests
cargo test

# Check code formatting
cargo fmt --check

# Run linting
cargo clippy

# Run all checks (format, clippy, test)
cargo fmt --check && cargo clippy && cargo test
```

### Running the Application
```bash
# Run directly from source
cargo run

# Run with specific CLI arguments
cargo run -- --help
cargo run -- --print-config
cargo run -- --show-block-status

# Test statusline generation (requires JSON input)
echo '{"model":"claude-3-5-sonnet","workingDirectory":"/test","gitBranch":"main"}' | cargo run
```

### Installation and Distribution
```bash
# Install locally
cargo install --path .

# Create release binaries
cargo build --release
cp target/release/ccometixline ~/.claude/ccline/ccline
```

## Architecture Overview

### Core Components

1. **CLI Module** (`src/cli.rs`): Command-line argument parsing using clap
   - Handles version, config, update, and block management commands
   - Provides user-facing interface for all tool operations

2. **Configuration System** (`src/config/`):
   - `types.rs`: Core configuration structures and data models
   - `loader.rs`: Configuration loading logic with environment variable support
   - `defaults.rs`: Default configuration values
   - `block_overrides.rs`: Manual billing block synchronization management

3. **Core Engine** (`src/core/`):
   - `statusline.rs`: Main statusline generation orchestrator
   - `segments/`: Individual statusline segment implementations
     - `model.rs`: Claude model display with simplified names
     - `directory.rs`: Current workspace directory
     - `git.rs`: Git branch, status, and tracking information
     - `usage.rs`: Token usage tracking from transcripts
     - `cost.rs`: Real-time cost calculation and billing blocks
     - `burn_rate.rs`: Token consumption rate monitoring
     - `update.rs`: Self-update notifications

4. **Billing System** (`src/billing/`):
   - `calculator.rs`: Cost calculation engine
   - `pricing.rs`: Claude model pricing data
   - `block.rs`: 5-hour billing block detection algorithm
   - `types.rs`: Billing-related data structures

5. **Utilities** (`src/utils/`):
   - `data_loader.rs`: Claude Code data file parsing
   - `transcript.rs`: Conversation transcript analysis

6. **Self-Update** (`src/updater.rs`): GitHub release checking and binary updates

### Data Flow

1. Main entry point reads JSON input from stdin (Claude Code integration)
2. ConfigLoader assembles configuration from defaults + environment variables
3. StatusLineGenerator creates segment instances based on config
4. Each segment renders its content using the input data
5. Final statusline is assembled with ANSI color codes and separators

### Environment Variables

- `CCLINE_DISABLE_COST=1`: Disables cost and burn rate segments
- `CCLINE_SHOW_TIMING=1`: Shows performance timing for debugging

### Key Features Implementation

- **Git Integration**: Uses git commands to detect branch, status, and remote tracking
- **Cost Tracking**: Implements ccusage-compatible pricing and billing block algorithms
- **Performance**: Rust native implementation with <50ms startup time
- **Unicode Support**: Uses Nerd Font icons for visual elements
- **Cross-Platform**: Supports Linux, macOS, and Windows binaries

### Testing Strategy

Tests should focus on:
- Segment rendering with various input scenarios
- Configuration loading and validation
- Billing calculation accuracy
- CLI argument parsing
- Error handling for missing dependencies (git, transcript files)

### Dependencies

- Core: `serde`, `clap`, `toml`, `chrono`, `tokio`, `reqwest`
- Optional features: `ureq`, `semver`, `dirs` (for self-update)
- Build tools: Standard Rust toolchain (rustc, cargo)
- Runtime: Git 1.5+ for git segment functionality

### Release Process

1. Update version in `Cargo.toml`
2. Build release binaries for all platforms: `cargo build --release`
3. Test binary functionality across platforms
4. Package binaries as platform-specific archives
5. Create GitHub release with appropriate assets

## Integration Notes

This tool is designed specifically for Claude Code statusline integration. It expects:
- JSON input via stdin containing model, directory, and git information
- Nerd Font support in the terminal for proper icon display
- Access to Claude Code transcript files for usage/cost analysis
- Git repository context for git-related segments