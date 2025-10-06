use std::path::PathBuf;

/// Resolve the preferred configuration directory for ccline.
///
/// Priority order:
/// 1. Environment override via `CCLINE_CONFIG_HOME`
/// 2. Existing Claude directory (`~/.claude/ccline`)
/// 3. Codex directory (`~/.codex/ccline`)
/// 4. Default to Codex directory when none exist (new installations)
pub fn resolve_config_dir() -> PathBuf {
    if let Ok(custom) = std::env::var("CCLINE_CONFIG_HOME") {
        let path = PathBuf::from(custom);
        if path.is_absolute() {
            return path;
        }
        if let Some(home) = dirs::home_dir() {
            return home.join(path);
        }
    }

    let default = dirs::home_dir().unwrap_or_default();
    let claude_dir = default.join(".claude").join("ccline");
    if claude_dir.exists() {
        return claude_dir;
    }

    let codex_dir = default.join(".codex").join("ccline");
    if codex_dir.exists() {
        return codex_dir;
    }

    codex_dir
}
