# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **OpenAI/Codex model pricing**: Added accurate pricing for gpt-5-codex, o3, o4, and o4-mini models ($0.75/1M input, $6.00/1M output)
- **Comprehensive test suite**: Created integration tests for Codex transcript parsing, model detection, and pricing lookup
- **Codex integration guide**: New documentation at `docs/CODEX_INTEGRATION.md` with setup instructions and troubleshooting
- **Example test script**: Added `examples/test_codex_statusline.sh` for testing Codex integration

### Changed
- Extended LiteLLM pricing fetcher to include OpenAI models alongside Claude models
- Improved model provider detection to recognize GPT-5, O3, and O4 model identifiers

## [1.0.0] - 2025-10-06

### Added
- Codex provider support with automatic input detection and transcript normalization
- Shared configuration resolver that supports both Claude and Codex install directories and `CCLINE_CONFIG_HOME`
- Documentation for Codex installation, configuration, and new environment variables in English and Chinese

### Changed
- Refactored status segments to use provider-aware model metadata and shared usage parsing
- Updated test fixtures and examples to include provider context alongside model identifiers

### Fixed
- Ensured block overrides, update state, and cost/burn-rate calculations read from both Claude and Codex transcript archives without manual tweaks

## [0.1.1] - 2025-08-12

### Added
- Support for `total_tokens` field in token calculation for better accuracy with GLM-4.5 and similar providers
- Proper Git repository detection using `git rev-parse --git-dir`
- Cross-platform compatibility improvements for Windows path handling
- Pre-commit hooks for automatic code formatting
- **Static Linux binary**: Added musl-based static binary for universal Linux compatibility without glibc dependencies

### Changed
- **Token calculation priority**: `total_tokens` → Claude format → OpenAI format → fallback
- **Display formatting**: Removed redundant ".0" from integer percentages and token counts
  - `0.0%` → `0%`, `25.0%` → `25%`, `50.0k` → `50k`
- **CI/CD**: Updated GitHub Actions to use Ubuntu 22.04 for Linux builds and ubuntu-latest for Windows cross-compilation
- **Binary distribution**: Now provides two Linux options - dynamic (glibc) and static (musl) binaries
- **Version management**: Unified version number using `env!("CARGO_PKG_VERSION")`

### Fixed
- Git segment now properly hides for non-Git directories instead of showing misleading "detached" status
- Windows Git repository path handling issues by removing overly aggressive path sanitization
- GitHub Actions runner compatibility issues (updated to supported versions: ubuntu-22.04 for Linux, ubuntu-latest for Windows)
- **Git version compatibility**: Added fallback to `git symbolic-ref` for Git versions < 2.22 when `--show-current` is not available

### Removed
- Path sanitization function that could break Windows paths in Git operations

## [0.1.0] - 2025-08-11

### Added
- Initial release of CCometixLine
- High-performance Rust-based statusline tool for Claude Code
- Git integration with branch, status, and tracking info
- Model display with simplified Claude model names
- Usage tracking based on transcript analysis
- Directory display showing current workspace
- Minimal design using Nerd Font icons
- Cross-platform support (Linux, macOS, Windows)
- Command-line configuration options
- GitHub Actions CI/CD pipeline

### Technical Details
- Context limit: 200,000 tokens
- Startup time: < 50ms
- Memory usage: < 10MB
- Binary size: ~2MB optimized release build
