# CCometixLine

[English](README.md) | [中文](README.zh.md)

CCometixLine 是一个为 Claude Code 与 Codex 提供支持的高性能状态栏工具。它实时读取会话 JSON 数据、Git 信息以及转录用量，生成紧凑的命令行状态条，展示模型、目录、Git 状态、令牌使用量、成本与燃烧率。

![Language:Rust](https://img.shields.io/static/v1?label=Language&message=Rust&color=orange&style=flat-square)
![License:MIT](https://img.shields.io/static/v1?label=License&message=MIT&color=blue&style=flat-square)

## 亮点

- **双提供方支持**：自动识别 Claude 与 Codex 转录文件，无需额外配置。
- **即时状态栏**：模型、目录、Git、用量、成本、燃烧率集成在一行输出中。
- **智能模型名称**：将 Claude 与 Codex 模型 ID 规范化为易读标签。
- **精准用量分析**：解析 Claude `assistant` 消息与 Codex `token_count` 事件。
- **成本 & 燃烧率**：复刻 ccusage 的计费块算法，支持手动覆盖。
- **极速轻量**：Rust 原生实现，启动耗时仅毫秒级，内存占用 <10 MB。

## 安装步骤

### 1. 选择安装目录

| 提供方      | 默认安装路径              |
|-------------|---------------------------|
| Claude Code | `~/.claude/ccline`        |
| Codex CLI   | `~/.codex/ccline`         |

建议先导出 `CCLINE_HOME` 变量，方便后续命令：

```bash
# Claude Code
export CCLINE_HOME="$HOME/.claude/ccline"
# Codex CLI
# export CCLINE_HOME="$HOME/.codex/ccline"
mkdir -p "$CCLINE_HOME"
```

### 2. 下载发行版二进制

#### Linux (glibc, x86_64)
```bash
wget https://github.com/Haleclipse/CCometixLine/releases/latest/download/ccline-linux-x64.tar.gz
 tar -xzf ccline-linux-x64.tar.gz
 install -Dm755 ccline "$CCLINE_HOME/ccline"
```

#### Linux (musl 静态, x86_64)
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

### 3. 配置编辑器 / CLI

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

Windows 请将命令路径替换为 `%USERPROFILE%\.claude\ccline\ccline.exe` 或 `%USERPROFILE%\.codex\ccline\ccline.exe`。

### 从源码构建

```bash
git clone https://github.com/Haleclipse/CCometixLine.git
cd CCometixLine
cargo build --release
install -Dm755 target/release/ccometixline "$CCLINE_HOME/ccline"
```

## 使用方式

CCometixLine 从标准输入读取一次 JSON 负载，输出带颜色的状态栏字符串。

```bash
# Claude / Codex 会自动注入 JSON
data | ccline

# 查看默认配置
echo '{}' | ccline --print-config

# 管理 5 小时计费块
ccline --set-block-start 10
ccline --show-block-status
ccline --clear-block-start
```

## 数据来源与环境变量

- 转录目录：
  - Claude：`~/.config/claude/projects`、`~/.claude/projects`
  - Codex：`~/.codex/sessions`
- `CLAUDE_CONFIG_DIR`：额外的 Claude 项目根目录（自动追加 `/projects`）。
- `CODEX_SESSIONS_DIR`：额外的 Codex 会话根目录（逗号分隔）。
- `CCLINE_CONFIG_HOME`：覆盖块设置与更新状态的存储目录。
- `CCLINE_DISABLE_COST=1`：隐藏成本与燃烧率段。
- `CCLINE_SHOW_TIMING=1`：附加性能 profiling 信息，便于调试。

## 状态栏段落

| 段落     | 描述 |
|----------|------|
| Model    | 根据提供方显示可读模型名称，如 `Sonnet 3.5`、`GPT-5 Codex` |
| Directory| 当前工作目录 |
| Git      | 分支、整洁度 (✓ / ● / ⚠) 与领先/落后计数 |
| Usage    | 基于 200 k 上下文限制的占用百分比 |
| Cost     | 会话成本、当日总额、当前计费块摘要 |
| Burn rate| 令牌/分钟趋势，结合 🔥 / ⚡ 指示 |
| Update   | 检测到新版本时的提醒 |

## 性能

- 启动时间 < 50 ms
- 内存占用 < 10 MB
- 发布版二进制 ≈ 2 MB

## 依赖要求

- Git 1.5+（推荐 2.22+ 以获得更佳分支检测）
- 支持 Nerd Font 的终端字体（如 FiraCode NF、JetBrains Mono NF）
- Claude Code 桌面端 **或** Codex CLI（用于状态栏集成）

## 开发脚本

```bash
cargo fmt
cargo clippy --all-targets
cargo test
```

## 规划

- TOML 配置文件
- 内置 TUI 配置器
- 主题自定义
- 插件扩展点

## 鸣谢

成本与计费逻辑参考自 [ccusage](https://github.com/ryoppippi/ccusage)。

## 许可证

遵循 [MIT License](LICENSE)。

## Star 历史

[![Star History Chart](https://api.star-history.com/svg?repos=Haleclipse/CCometixLine&type=Date)](https://star-history.com/#Haleclipse/CCometixLine&Date)
