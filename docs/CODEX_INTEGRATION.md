# Codex Integration Guide

This document explains how to integrate CCometixLine with Codex CLI.

## Quick Start

### 1. Install CCometixLine

```bash
# Create Codex ccline directory
mkdir -p ~/.codex/ccline

# Download and install (example for Linux x64)
wget https://github.com/Haleclipse/CCometixLine/releases/latest/download/ccline-linux-x64.tar.gz
tar -xzf ccline-linux-x64.tar.gz
install -Dm755 ccline ~/.codex/ccline/ccline
```

### 2. Configure Codex Statusline

Edit your Codex configuration file at `~/.codex/config.toml` and add:

```toml
# Enable custom statusline
statusline_command = ["/home/YOUR_USERNAME/.codex/ccline/ccline"]
```

Replace `YOUR_USERNAME` with your actual username, or use the full absolute path to the binary.

### 3. Test the Integration

Create a test JSON payload to verify the statusline works:

```bash
echo '{
  "model": "gpt-5-codex",
  "workspace": {"cwd": "'$(pwd)'"},
  "transcriptPath": "'$HOME'/.codex/sessions/latest.jsonl"
}' | ~/.codex/ccline/ccline
```

You should see a colorful statusline with model name, directory, and other information.

## Supported Models

CCometixLine supports all Codex/OpenAI models with accurate pricing:

| Model | Input Cost | Output Cost |
|-------|------------|-------------|
| `gpt-5-codex` | $0.75 / 1M tokens | $6.00 / 1M tokens |
| `gpt-5-codex-preview` | $0.75 / 1M tokens | $6.00 / 1M tokens |
| `o3` | $0.75 / 1M tokens | $6.00 / 1M tokens |
| `o4` | $0.75 / 1M tokens | $6.00 / 1M tokens |
| `o4-mini` | $0.75 / 1M tokens | $6.00 / 1M tokens |

## Data Format

Codex provides session data in the following JSON format:

```json
{
  "model": "gpt-5-codex",
  "workspace": {
    "cwd": "/home/user/project"
  },
  "transcriptPath": "/home/user/.codex/sessions/session-id.jsonl"
}
```

### Transcript Format

Codex transcripts use JSONL format with token count events:

```json
{
  "ts": "2025-10-07T10:00:00.000Z",
  "dir": "to_tui",
  "kind": "event_msg",
  "payload": {
    "type": "token_count",
    "model": "gpt-5-codex",
    "info": {
      "total_token_usage": {
        "input_tokens": 1000,
        "cached_input_tokens": 200,
        "output_tokens": 500,
        "reasoning_output_tokens": 100,
        "total_tokens": 1800
      },
      "last_token_usage": {
        "input_tokens": 100,
        "cached_input_tokens": 20,
        "output_tokens": 50,
        "reasoning_output_tokens": 10,
        "total_tokens": 180
      }
    }
  }
}
```

## Statusline Segments

The statusline displays the following information:

### 1. Model
- **Format**: `GPT-5 Codex` (prettified model name)
- **Icon**: Appropriate icon for OpenAI models

### 2. Directory
- **Format**: Current working directory (abbreviated)
- **Icon**: 📁

### 3. Git Status
- **Format**: Branch name + status indicators
- **Icons**:
  - ✓ = Clean
  - ● = Uncommitted changes
  - ⚠ = Untracked files

### 4. Token Usage
- **Format**: `[###···] 45%` (progress bar + percentage)
- **Calculation**: Based on total tokens vs context window

### 5. Cost
- **Format**: `$1.23 ($5.67/day)`
- **Includes**: Current session cost + daily total
- **Billing blocks**: 5-hour billing block tracking (similar to ccusage)

### 6. Burn Rate
- **Format**: `120 tok/min` with indicator
- **Icons**:
  - 🔥 = High usage (>100 tokens/min)
  - ⚡ = Normal usage

## Environment Variables

Configure CCometixLine behavior with these variables:

```bash
# Disable cost tracking
export CCLINE_DISABLE_COST=1

# Show performance timing (debug)
export CCLINE_SHOW_TIMING=1

# Override config directory
export CCLINE_CONFIG_HOME="$HOME/.config/ccline"

# Additional transcript directories
export CODEX_SESSIONS_DIR="$HOME/.codex/sessions"
```

## Troubleshooting

### Statusline not appearing

1. Check if the binary is executable:
   ```bash
   ls -la ~/.codex/ccline/ccline
   chmod +x ~/.codex/ccline/ccline
   ```

2. Test the binary directly:
   ```bash
   echo '{"model":"gpt-5-codex","workspace":{"cwd":"/tmp"},"transcriptPath":"/tmp/test.jsonl"}' | ~/.codex/ccline/ccline
   ```

3. Check Codex config file syntax:
   ```bash
   cat ~/.codex/config.toml
   ```

### No cost information

- Ensure transcripts exist: `ls ~/.codex/sessions/`
- Check that `CCLINE_DISABLE_COST` is not set
- Verify pricing data is available (should work offline with fallback pricing)

### Icons not displaying

Install a Nerd Font in your terminal:
- [FiraCode Nerd Font](https://www.nerdfonts.com/font-downloads)
- [JetBrains Mono Nerd Font](https://www.nerdfonts.com/font-downloads)

Configure your terminal to use the Nerd Font.

### High memory usage

CCometixLine is designed to be lightweight (<10 MB RSS). If you see high memory:
- Check if old transcript files are too large
- Consider archiving old sessions

## Advanced Configuration

### Manual Billing Block Override

Set a custom billing block start time:

```bash
# Set block start to 10:00 AM today
~/.codex/ccline/ccline --set-block-start "10:00"

# View current block status
~/.codex/ccline/ccline --show-block-status

# Clear override
~/.codex/ccline/ccline --clear-block-start
```

### Custom Pricing

While CCometixLine includes built-in pricing for all Codex models, you can extend it by modifying the source if needed.

## Differences from Claude Code Integration

| Feature | Claude Code | Codex CLI |
|---------|-------------|-----------|
| Transcript location | `~/.claude/projects/` | `~/.codex/sessions/` |
| Event type | `assistant` messages | `token_count` events |
| Model format | Object with `display_name` | String identifier |
| Workspace key | `current_dir` | `cwd` |
| Cache handling | `cache_creation` + `cache_read` | `cached_input_tokens` only |
| Reasoning tokens | N/A | `reasoning_output_tokens` |

## Contributing

Found an issue or want to improve Codex integration?

1. Check existing issues: https://github.com/Haleclipse/CCometixLine/issues
2. Submit a PR with your changes
3. Include test cases for new features

## Resources

- [Codex CLI Documentation](https://github.com/openai/codex)
- [CCometixLine Repository](https://github.com/Haleclipse/CCometixLine)
- [CODEX.md](../CODEX.md) - Developer guide for Codex integration
