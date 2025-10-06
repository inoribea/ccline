# CCometixLine

[English](README.md) | [中文](README.zh.md)

CCometixLine is a high-performance statusline generator for both Claude Code and Codex. It reads live session data, Git metadata, and transcript usage to show a compact command-line status bar that tracks your model, workspace, token usage, cost, and burn rate in real time.

![Language:Rust](https://img.shields.io/static/v1?label=Language&message=Rust&color=orange&style=flat-square)
![License:MIT](https://img.shields.io/static/v1?label=License&message=MIT&color=blue&style=flat-square)

## Highlights

- **Dual provider support** – Auto-detects Claude Code and Codex transcripts with zero configuration.
- **Live statusline** – Displays model, directory, Git status, usage, cost, and burn rate in a single line.
- **Provider-aware model names** – Normalises Claude and Codex identifiers into readable labels.
- **Accurate usage analytics** – Replays JSONL transcripts (Claude `assistant` messages and Codex `token_count` events).
- **Cost + burn rate** – Mirrors ccusage billing heuristics with manual override support.
- **Fast and lightweight** – Rust binary starts in milliseconds and uses <10 MB RSS.

## Installation

### 1. Choose an install directory

| Provider   | Default install path        |
|------------|-----------------------------|
| Claude Code | `~/.claude/ccline`          |
| Codex CLI  | `~/.codex/ccline`           |

Export `CCLINE_HOME` to simplify the commands below:

```bash
# Claude Code
export CCLINE_HOME="$HOME/.claude/ccline"
# Codex CLI
# export CCLINE_HOME="$HOME/.codex/ccline"
```

Create the directory before copying the binary:

```bash
mkdir -p "$CCLINE_HOME"
```

### 2. Download a release binary

#### Linux (glibc, x86_64)
```bash
wget https://github.com/Haleclipse/CCometixLine/releases/latest/download/ccline-linux-x64.tar.gz
 tar -xzf ccline-linux-x64.tar.gz
 install -Dm755 ccline "$CCLINE_HOME/ccline"
```

#### Linux (static musl, x86_64)
```bash
wget https://github.com/Haleclipse/CCometixLine/releases/latest/download/ccline-linux-x64-static.tar.gz
 tar -xzf ccline-linux-x64-static.tar.gz
 install -Dm755 ccline "$CCLINE_HOME/ccline"
```

#### macOS (Intel)
```bash
wget https://github.com/Haleclipse/CCometixLine/releases/latest/download/ccline-macos-x64.tar.gz
 tar -xzf ccline-macos-x64.tar.gz
 install -Dm755 ccline "$CCLINE_HOME/ccline"
```

#### macOS (Apple Silicon)
```bash
wget https://github.com/Haleclipse/CCometixLine/releases/latest/download/ccline-macos-arm64.tar.gz
 tar -xzf ccline-macos-arm64.tar.gz
 install -Dm755 ccline "$CCLINE_HOME/ccline"
```

#### Windows (PowerShell)
```powershell
# Claude Code:  $env:USERPROFILE\.claude\ccline
# Codex CLI:    $env:USERPROFILE\.codex\ccline
$env:CCLINE_HOME = "$env:USERPROFILE\.claude\ccline"
New-Item -ItemType Directory -Force -Path $env:CCLINE_HOME | Out-Null
Invoke-WebRequest -Uri "https://github.com/Haleclipse/CCometixLine/releases/latest/download/ccline-windows-x64.zip" -OutFile "ccline-windows-x64.zip"
Expand-Archive -Path "ccline-windows-x64.zip" -DestinationPath "." -Force
Move-Item "ccline.exe" "$env:CCLINE_HOME\ccline.exe" -Force
```

### 3. Wire up your editor/CLI

#### Claude Code (`settings.json`)
```json
{
  "statusLine": {
    "type": "command",
    "command": "~/.claude/ccline/ccline",
    "padding": 0
  }
}
```

#### Codex CLI (`~/.codex/config.toml`)
```toml
[status_line]
type = "command"
command = "~/.codex/ccline/ccline"
padding = 0
```

On Windows replace the command path with `%USERPROFILE%\.claude\ccline\ccline.exe` or `%USERPROFILE%\.codex\ccline\ccline.exe`.

### Build from source

```bash
git clone https://github.com/Haleclipse/CCometixLine.git
cd CCometixLine
cargo build --release
install -Dm755 target/release/ccometixline "$CCLINE_HOME/ccline"
```

## Usage

CCometixLine reads a single JSON payload from stdin and prints a fully-coloured statusline.

```bash
# Claude Code / Codex both feed the JSON payload automatically
ccline

# Inspect the default configuration
echo '{}' | ccline --print-config

# Manage 5‑hour billing blocks
ccline --set-block-start 10
ccline --show-block-status
ccline --clear-block-start
```

## Data sources & environment

- Transcript roots:
  - Claude: `~/.config/claude/projects` and `~/.claude/projects`
  - Codex: `~/.codex/sessions`
- `CLAUDE_CONFIG_DIR` – extra comma-separated Claude project roots (auto-append `/projects`).
- `CODEX_SESSIONS_DIR` – comma-separated Codex session roots.
- `CCLINE_CONFIG_HOME` – override the directory used for block overrides and update state.
- `CCLINE_DISABLE_COST=1` – hide cost and burn-rate segments.
- `CCLINE_SHOW_TIMING=1` – append profiling numbers useful for debugging.

## Statusline segments

| Segment    | Description |
|------------|-------------|
| Model      | Provider-aware label, e.g. `Sonnet 3.5`, `GPT-5 Codex` |
| Directory  | Current workspace / project folder |
| Git        | Branch, cleanliness (✓ / ● / ⚠), ahead/behind counters |
| Usage      | Context consumption within a 200 k token limit |
| Cost       | Session + daily spend, active billing block summary |
| Burn rate  | Tokens/minute trend with 🔥 / ⚡ indicators |
| Update     | Inline notifier when a new release is available |

## Performance

- Startup < 50 ms
- Memory footprint < 10 MB
- Release binary ≈ 2 MB

## Requirements

- Git 1.5+ (Git 2.22+ recommended for branch detection)
- Terminal with a Nerd Font (e.g., FiraCode NF, JetBrains Mono NF)
- Claude Code desktop app **or** Codex CLI (for statusline integration)

## Development

```bash
cargo fmt
cargo clippy --all-targets
cargo test
```

## Roadmap

- TOML configuration file
- In-app TUI configurator
- Theme customization
- Plugin hooks

## Acknowledgments

Cost and billing heuristics are inspired by [ccusage](https://github.com/ryoppippi/ccusage).

## License

MIT License. See [LICENSE](LICENSE).

## Star History

[![Star History Chart](https://api.star-history.com/svg?repos=Haleclipse/CCometixLine&type=Date)](https://star-history.com/#Haleclipse/CCometixLine&Date)
