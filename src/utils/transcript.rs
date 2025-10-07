use crate::billing::UsageEntry;
use crate::config::{NormalizedUsage, ProviderKind, TokenCountInfo, TranscriptEntry};
use chrono::{DateTime, Utc};
use std::collections::HashSet;

/// Extract session ID from file path (the UUID part)
pub fn extract_session_id(path: &std::path::Path) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string()
}

#[derive(Debug, Default)]
pub struct TranscriptState {
    provider: Option<ProviderKind>,
    current_model: Option<String>,
    pub last_normalized: Option<NormalizedUsage>,
}

impl TranscriptState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_provider(provider: Option<ProviderKind>) -> Self {
        Self {
            provider,
            ..Default::default()
        }
    }

    pub fn provider(&self) -> Option<ProviderKind> {
        self.provider
    }

    fn update_provider(&mut self, entry: &TranscriptEntry) {
        if self.provider.is_some() {
            return;
        }

        self.provider = detect_provider_from_entry(entry);
    }

    fn update_model_from_entry(&mut self, entry: &TranscriptEntry) {
        if let Some(message) = entry.message.as_ref() {
            if let Some(model) = message.model.as_ref() {
                if !model.is_empty() {
                    self.current_model = Some(model.clone());
                }
            }
        }

        if let Some(payload) = entry.payload.as_ref() {
            if let Some(model) = payload.model.as_ref() {
                if !model.is_empty() {
                    self.current_model = Some(model.clone());
                }
            }
        }
    }
}

/// Parse a JSONL line and extract usage entry if valid
pub fn parse_line_to_usage(
    line: &str,
    session_id: &str,
    seen: &mut HashSet<String>,
    state: &mut TranscriptState,
) -> Option<UsageEntry> {
    // Parse the JSON line
    let entry: TranscriptEntry = serde_json::from_str(line).ok()?;

    state.update_provider(&entry);
    state.update_model_from_entry(&entry);

    match state.provider() {
        Some(ProviderKind::Claude) => parse_claude_entry(&entry, session_id, seen, state),
        Some(ProviderKind::Codex) => parse_codex_entry(&entry, session_id, seen, state),
        None => None,
    }
}

fn parse_claude_entry(
    entry: &TranscriptEntry,
    session_id: &str,
    seen: &mut HashSet<String>,
    state: &mut TranscriptState,
) -> Option<UsageEntry> {
    if entry.r#type.as_deref() != Some("assistant") {
        return None;
    }

    let message = entry.message.as_ref()?;
    let raw_usage = message.usage.as_ref()?;

    if let (Some(msg_id), Some(req_id)) = (message.id.as_ref(), entry.request_id.as_ref()) {
        let hash = format!("claude:{}:{}:{}", session_id, msg_id, req_id);
        if seen.contains(&hash) {
            return None;
        }
        seen.insert(hash);
    }

    let normalized = raw_usage.clone().normalize();
    state.last_normalized = Some(normalized.clone());

    let model_ref = message.model.as_deref().or(state.current_model.as_deref());

    extract_usage_entry(
        &normalized,
        session_id,
        entry.timestamp.as_deref(),
        model_ref,
    )
}

fn parse_codex_entry(
    entry: &TranscriptEntry,
    session_id: &str,
    seen: &mut HashSet<String>,
    state: &mut TranscriptState,
) -> Option<UsageEntry> {
    let payload = entry.payload.as_ref()?;
    let payload_type = payload.r#type.as_deref()?;

    if payload_type != "token_count" {
        return None;
    }

    let info = payload.info.as_ref()?;
    info.last_token_usage.as_ref()?;

    let hash = codex_hash(session_id, info, entry.timestamp.as_deref());
    if seen.contains(&hash) {
        return None;
    }
    seen.insert(hash);

    let normalized = normalize_codex_usage(info);
    state.last_normalized = Some(normalized.clone());

    let model = state.current_model.clone().unwrap_or_default();

    extract_usage_entry(
        &normalized,
        session_id,
        entry.timestamp.as_deref(),
        Some(model.as_str()),
    )
}

/// Convert NormalizedUsage to UsageEntry
pub fn extract_usage_entry(
    normalized: &NormalizedUsage,
    session_id: &str,
    timestamp_str: Option<&str>,
    model: Option<&str>,
) -> Option<UsageEntry> {
    // Parse timestamp or use current time
    let timestamp = if let Some(ts_str) = timestamp_str {
        DateTime::parse_from_rfc3339(ts_str)
            .ok()
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(Utc::now)
    } else {
        Utc::now()
    };

    Some(UsageEntry {
        timestamp,
        input_tokens: normalized.input_tokens,
        output_tokens: normalized.output_tokens,
        cache_creation_tokens: normalized.cache_creation_input_tokens,
        cache_read_tokens: normalized.cache_read_input_tokens,
        model: model.unwrap_or("").to_string(),
        cost: None, // Will be calculated later with pricing data
        session_id: session_id.to_string(),
    })
}

fn detect_provider_from_entry(entry: &TranscriptEntry) -> Option<ProviderKind> {
    match entry.r#type.as_deref() {
        Some("assistant") if entry.message.is_some() => Some(ProviderKind::Claude),
        Some("event_msg") => entry
            .payload
            .as_ref()
            .and_then(|payload| payload.r#type.as_deref())
            .and_then(|payload_type| {
                if payload_type == "token_count" {
                    Some(ProviderKind::Codex)
                } else {
                    None
                }
            }),
        Some("turn_context") => entry
            .payload
            .as_ref()
            .and_then(|payload| payload.model.as_ref())
            .map(|_| ProviderKind::Codex),
        _ => None,
    }
}

fn codex_hash(session_id: &str, info: &TokenCountInfo, timestamp: Option<&str>) -> String {
    let totals = info
        .total_token_usage
        .as_ref()
        .and_then(|usage| usage.total_tokens)
        .unwrap_or(0);

    let last = info.last_token_usage.as_ref();
    let input = last.and_then(|u| u.input_tokens).unwrap_or(0);
    let cached = last.and_then(|u| u.cached_input_tokens).unwrap_or(0);
    let output = last.and_then(|u| u.output_tokens).unwrap_or(0);
    let reasoning = last.and_then(|u| u.reasoning_output_tokens).unwrap_or(0);

    let ts = timestamp.unwrap_or("");

    format!(
        "codex:{}:{}:{}:{}:{}:{}:{}",
        session_id, ts, totals, input, cached, output, reasoning
    )
}

fn normalize_codex_usage(info: &TokenCountInfo) -> NormalizedUsage {
    let last = info.last_token_usage.as_ref().cloned().unwrap_or_default();

    let mut raw_fields = Vec::new();

    if last.input_tokens.is_some() {
        raw_fields.push("input_tokens".to_string());
    }
    if last.cached_input_tokens.is_some() {
        raw_fields.push("cached_input_tokens".to_string());
    }
    if last.output_tokens.is_some() {
        raw_fields.push("output_tokens".to_string());
    }
    if last.reasoning_output_tokens.is_some() {
        raw_fields.push("reasoning_output_tokens".to_string());
    }
    if last.total_tokens.is_some() {
        raw_fields.push("total_tokens".to_string());
    }

    let input_tokens = last.input_tokens.unwrap_or(0);
    let cache_read = last.cached_input_tokens.unwrap_or(0);
    let reasoning_tokens = last.reasoning_output_tokens.unwrap_or(0);
    let output_tokens = last.output_tokens.unwrap_or(0) + reasoning_tokens;

    let total_tokens = last
        .total_tokens
        .unwrap_or_else(|| input_tokens + cache_read + output_tokens);

    NormalizedUsage {
        input_tokens,
        output_tokens,
        total_tokens,
        cache_creation_input_tokens: 0,
        cache_read_input_tokens: cache_read,
        calculation_source: "codex_last_token_usage".to_string(),
        raw_data_available: raw_fields,
    }
}

/// Parse entire transcript and return the latest normalized usage snapshot
pub fn parse_latest_usage<P: AsRef<std::path::Path>>(
    transcript_path: P,
    provider_hint: Option<ProviderKind>,
) -> Option<NormalizedUsage> {
    use std::io::{BufRead, BufReader};

    let file = std::fs::File::open(&transcript_path).ok()?;
    let reader = BufReader::new(file);
    let session_id = extract_session_id(transcript_path.as_ref());
    let mut state = TranscriptState::with_provider(provider_hint);
    let mut seen = HashSet::new();

    for line in reader.lines().map_while(Result::ok) {
        let _ = parse_line_to_usage(&line, &session_id, &mut seen, &mut state);
    }

    state.last_normalized
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::TokenUsageBreakdown;

    #[test]
    fn test_extract_session_id() {
        let path = std::path::Path::new(
            "/home/user/.claude/projects/test/c040b0ba-658d-4188-befa-0d2dad1f0ea5.jsonl",
        );
        assert_eq!(
            extract_session_id(path),
            "c040b0ba-658d-4188-befa-0d2dad1f0ea5"
        );
    }

    #[test]
    fn test_normalized_to_usage_entry() {
        let normalized = NormalizedUsage {
            input_tokens: 100,
            output_tokens: 50,
            total_tokens: 150,
            cache_creation_input_tokens: 10,
            cache_read_input_tokens: 5,
            calculation_source: "test".to_string(),
            raw_data_available: vec![],
        };

        let entry =
            extract_usage_entry(&normalized, "test-session", None, Some("claude-3-5-sonnet"))
                .unwrap();
        assert_eq!(entry.input_tokens, 100);
        assert_eq!(entry.output_tokens, 50);
        assert_eq!(entry.cache_creation_tokens, 10);
        assert_eq!(entry.cache_read_tokens, 5);
        assert_eq!(entry.session_id, "test-session");
        assert_eq!(entry.model, "claude-3-5-sonnet");
        assert!(entry.cost.is_none());
    }

    #[test]
    fn test_codex_normalization() {
        let info = TokenCountInfo {
            total_token_usage: Some(TokenUsageBreakdown {
                input_tokens: Some(1000),
                cached_input_tokens: Some(800),
                output_tokens: Some(50),
                reasoning_output_tokens: Some(20),
                total_tokens: Some(1870),
            }),
            last_token_usage: Some(TokenUsageBreakdown {
                input_tokens: Some(200),
                cached_input_tokens: Some(150),
                output_tokens: Some(12),
                reasoning_output_tokens: Some(8),
                total_tokens: Some(370),
            }),
        };

        let normalized = normalize_codex_usage(&info);
        assert_eq!(normalized.input_tokens, 200);
        assert_eq!(normalized.cache_read_input_tokens, 150);
        assert_eq!(normalized.output_tokens, 20);
        assert_eq!(normalized.total_tokens, 370);
        assert_eq!(normalized.calculation_source, "codex_last_token_usage");
    }
}
