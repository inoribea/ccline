use crate::billing::UsageEntry;
use crate::config::ProviderKind;
use crate::utils::transcript::{parse_line_to_usage, TranscriptState};
use glob::glob;
use std::collections::HashSet;
use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

pub struct DataLoader {
    transcript_dirs: Vec<PathBuf>,
}

impl DataLoader {
    pub fn new() -> Self {
        Self {
            transcript_dirs: Self::find_transcript_dirs(),
        }
    }

    /// Find all transcript directories for supported providers
    fn find_transcript_dirs() -> Vec<PathBuf> {
        let mut dirs: Vec<PathBuf> = Vec::new();

        // Get home directory
        if let Ok(home) = std::env::var("HOME") {
            let home_path = PathBuf::from(&home);

            // Claude (new) path (~/.config/claude/projects)
            let claude_new = home_path.join(".config/claude/projects");
            push_unique(&mut dirs, claude_new);

            // Claude (legacy) path (~/.claude/projects)
            let claude_old = home_path.join(".claude/projects");
            push_unique(&mut dirs, claude_old);

            // Codex transcripts (~/.codex/sessions)
            let codex_sessions = home_path.join(".codex/sessions");
            push_unique(&mut dirs, codex_sessions);
        }

        // Support custom directories via environment variable
        if let Ok(custom_dirs) = std::env::var("CLAUDE_CONFIG_DIR") {
            for dir in custom_dirs.split(',') {
                let trimmed = dir.trim();
                if trimmed.is_empty() {
                    continue;
                }
                let base = PathBuf::from(trimmed);
                let path = if base.ends_with("projects") {
                    base
                } else {
                    base.join("projects")
                };
                push_unique(&mut dirs, path);
            }
        }

        if let Ok(custom_dirs) = std::env::var("CODEX_SESSIONS_DIR") {
            for dir in custom_dirs.split(',') {
                let trimmed = dir.trim();
                if trimmed.is_empty() {
                    continue;
                }
                let path = PathBuf::from(trimmed);
                push_unique(&mut dirs, path);
            }
        }

        dirs
    }

    /// Load all usage data from all projects (fresh read every time)
    pub fn load_all_projects(&self) -> Vec<UsageEntry> {
        let mut all_entries = Vec::new();
        let mut seen_hashes = HashSet::new();

        // Scan all project directories
        for dir in &self.transcript_dirs {
            let pattern = format!("{}/**/*.jsonl", dir.display());
            if let Ok(paths) = glob(&pattern) {
                for path in paths.flatten() {
                    // Parse individual file
                    let entries = self.parse_jsonl_file(&path, &mut seen_hashes);
                    all_entries.extend(entries);
                }
            }
        }

        // Sort by timestamp
        all_entries.sort_by_key(|e| e.timestamp);

        all_entries
    }

    /// Parse a single JSONL file
    fn parse_jsonl_file(&self, path: &Path, seen: &mut HashSet<String>) -> Vec<UsageEntry> {
        let mut entries = Vec::new();

        // Extract session_id from filename (UUID)
        let session_id = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Read file content (handle large files)
        let content = match fs::metadata(path) {
            Ok(metadata) if metadata.len() > 100 * 1024 * 1024 => {
                // File > 100MB, only read last 10MB
                self.read_last_n_bytes(path, 10 * 1024 * 1024)
            }
            _ => fs::read_to_string(path).unwrap_or_default(),
        };

        let provider_hint = detect_provider_from_path(path);
        let mut state = TranscriptState::with_provider(provider_hint);

        // Parse each line
        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }

            // Parse transcript entry and extract usage
            if let Some(usage_entry) = parse_line_to_usage(line, &session_id, seen, &mut state) {
                entries.push(usage_entry);
            }
        }

        entries
    }

    /// Read the last N bytes of a file
    fn read_last_n_bytes(&self, path: &Path, n: usize) -> String {
        let mut file = match fs::File::open(path) {
            Ok(f) => f,
            Err(_) => return String::new(),
        };

        let file_len = match file.metadata() {
            Ok(m) => m.len(),
            Err(_) => return String::new(),
        };

        let start_pos = file_len.saturating_sub(n as u64);

        // Seek to start position
        if file.seek(SeekFrom::Start(start_pos)).is_err() {
            return String::new();
        }

        let mut buffer = Vec::new();
        let _ = file.read_to_end(&mut buffer);

        // Find first complete line (skip partial line at beginning)
        if let Some(pos) = buffer.iter().position(|&b| b == b'\n') {
            buffer.drain(..=pos);
        }

        String::from_utf8_lossy(&buffer).to_string()
    }
}

impl Default for DataLoader {
    fn default() -> Self {
        Self::new()
    }
}

fn push_unique(list: &mut Vec<PathBuf>, path: PathBuf) {
    if path.exists() && !list.contains(&path) {
        list.push(path);
    }
}

fn detect_provider_from_path(path: &Path) -> Option<ProviderKind> {
    let lowered = path.display().to_string().to_lowercase();
    if lowered.contains("/.codex/") || lowered.contains("\\.codex\\") {
        Some(ProviderKind::Codex)
    } else if lowered.contains("/.claude/") || lowered.contains("\\.claude\\") {
        Some(ProviderKind::Claude)
    } else {
        None
    }
}
