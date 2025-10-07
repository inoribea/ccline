use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::io;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub theme: String,
    pub segments: SegmentsConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SegmentsConfig {
    pub directory: bool,
    pub git: bool,
    pub model: bool,
    pub usage: bool,
    #[serde(default = "default_true")]
    pub cost: bool,
    #[serde(default = "default_true")]
    pub burn_rate: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderKind {
    Claude,
    Codex,
}

#[derive(Debug, Clone)]
pub struct Model {
    pub display_name: String,
    pub identifier: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Workspace {
    pub current_dir: String,
}

#[derive(Debug, Clone)]
pub struct InputData {
    pub provider: ProviderKind,
    pub model: Model,
    pub workspace: Workspace,
    pub transcript_path: String,
}

impl InputData {
    pub fn from_reader<R: io::Read>(reader: R) -> io::Result<Self> {
        let value: Value = serde_json::from_reader(reader)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
        Self::from_value(value).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
    }

    pub fn from_value(value: Value) -> Result<Self, String> {
        let obj = value
            .as_object()
            .ok_or_else(|| "Input must be a JSON object".to_string())?;

        let transcript_path = find_string(
            obj,
            &[
                "transcript_path",
                "transcriptPath",
                "transcript_file",
                "transcriptFile",
            ],
        )
        .ok_or_else(|| "Missing transcript_path in input".to_string())?;

        let workspace_dir = extract_workspace_dir(obj)
            .ok_or_else(|| "Missing workspace.current_dir or workingDirectory".to_string())?;

        let (model_display, model_identifier) = extract_model_info(obj)
            .ok_or_else(|| "Missing model information in input".to_string())?;

        let provider = detect_provider(&transcript_path, model_identifier.as_deref(), obj);

        Ok(Self {
            provider,
            model: Model {
                display_name: model_display,
                identifier: model_identifier,
            },
            workspace: Workspace {
                current_dir: workspace_dir,
            },
            transcript_path,
        })
    }
}

fn find_string(map: &serde_json::Map<String, Value>, keys: &[&str]) -> Option<String> {
    for key in keys {
        if let Some(Value::String(value)) = map.get(*key) {
            if !value.is_empty() {
                return Some(value.clone());
            }
        }
    }
    None
}

fn extract_workspace_dir(map: &serde_json::Map<String, Value>) -> Option<String> {
    if let Some(workspace) = map.get("workspace").and_then(|v| v.as_object()) {
        if let Some(dir) = find_string(workspace, &["current_dir", "currentDir", "cwd"]) {
            return Some(dir);
        }
        if let Some(dir) = find_string(workspace, &["path", "directory", "root"]) {
            return Some(dir);
        }
        if let Some(dir) = find_string(workspace, &["working_directory", "workingDirectory"]) {
            return Some(dir);
        }
    }

    find_string(
        map,
        &[
            "working_directory",
            "workingDirectory",
            "cwd",
            "current_dir",
            "currentDir",
        ],
    )
}

fn extract_model_info(map: &serde_json::Map<String, Value>) -> Option<(String, Option<String>)> {
    if let Some(model_value) = map.get("model") {
        match model_value {
            Value::String(name) => {
                let display = prettify_model_name(name);
                return Some((display, Some(name.clone())));
            }
            Value::Object(model_obj) => {
                let display =
                    find_string(model_obj, &["display_name", "displayName", "name", "id"])?;
                let identifier =
                    find_string(model_obj, &["identifier", "id", "name", "model", "slug"])
                        .or_else(|| Some(display.clone()));
                return Some((display, identifier));
            }
            _ => {}
        }
    }

    if let Some(display) = find_string(
        map,
        &[
            "model_display_name",
            "modelDisplayName",
            "modelName",
            "model",
        ],
    ) {
        let identifier = find_string(map, &["model_id", "modelId", "model", "modelName"])
            .or_else(|| Some(display.clone()));
        return Some((display, identifier));
    }

    None
}

fn detect_provider(
    transcript_path: &str,
    model_identifier: Option<&str>,
    map: &serde_json::Map<String, Value>,
) -> ProviderKind {
    let lowered_path = transcript_path.to_lowercase();
    if lowered_path.contains("/.codex/") || lowered_path.contains("\\.codex\\") {
        return ProviderKind::Codex;
    }
    if lowered_path.contains("/.claude/") || lowered_path.contains("\\.claude\\") {
        return ProviderKind::Claude;
    }

    if let Some(identifier) = model_identifier {
        let lowered = identifier.to_lowercase();
        if lowered.contains("codex") || lowered.contains("gpt-") {
            return ProviderKind::Codex;
        }
        if lowered.contains("claude") {
            return ProviderKind::Claude;
        }
    }

    if let Some(model_value) = map.get("model") {
        if let Some(model_str) = model_value.as_str() {
            let lowered = model_str.to_lowercase();
            if lowered.contains("codex") || lowered.contains("gpt-") {
                return ProviderKind::Codex;
            }
            if lowered.contains("claude") {
                return ProviderKind::Claude;
            }
        }
    }

    ProviderKind::Claude
}

fn prettify_model_name(raw: &str) -> String {
    if raw.is_empty() {
        return raw.to_string();
    }

    if raw.contains("gpt-5") && raw.contains("codex") {
        return "GPT-5 Codex".to_string();
    }

    raw.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_input_data_claude_format() {
        let value = json!({
            "model": {"display_name": "claude-3-5-sonnet"},
            "workspace": {"current_dir": "/home/user/project"},
            "transcript_path": "/home/user/.claude/projects/demo/session.jsonl"
        });

        let input = InputData::from_value(value).expect("should parse claude format");
        assert_eq!(input.provider, ProviderKind::Claude);
        assert_eq!(input.model.display_name, "claude-3-5-sonnet");
        assert_eq!(input.model.identifier.as_deref(), Some("claude-3-5-sonnet"));
        assert_eq!(input.workspace.current_dir, "/home/user/project");
    }

    #[test]
    fn test_input_data_codex_format() {
        let value = json!({
            "model": "gpt-5-codex",
            "workspace": {"cwd": "/home/inoribea/code/demo"},
            "transcriptPath": "/home/inoribea/.codex/sessions/2025/10/demo.jsonl"
        });

        let input = InputData::from_value(value).expect("should parse codex format");
        assert_eq!(input.provider, ProviderKind::Codex);
        assert_eq!(input.model.display_name, "GPT-5 Codex");
        assert_eq!(input.model.identifier.as_deref(), Some("gpt-5-codex"));
        assert_eq!(input.workspace.current_dir, "/home/inoribea/code/demo");
    }
}

// OpenAI-style nested token details
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct PromptTokensDetails {
    #[serde(default)]
    pub cached_tokens: Option<u32>,
    #[serde(default)]
    pub audio_tokens: Option<u32>,
}

// Raw usage data from different LLM providers (flexible parsing)
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct RawUsage {
    // Common input token naming variants
    #[serde(default, alias = "prompt_tokens")]
    pub input_tokens: Option<u32>,

    // Common output token naming variants
    #[serde(default, alias = "completion_tokens")]
    pub output_tokens: Option<u32>,

    // Total tokens (some providers only provide this)
    #[serde(default)]
    pub total_tokens: Option<u32>,

    // Anthropic-style cache fields
    #[serde(default, alias = "cache_creation_prompt_tokens")]
    pub cache_creation_input_tokens: Option<u32>,

    #[serde(default, alias = "cache_read_prompt_tokens")]
    pub cache_read_input_tokens: Option<u32>,

    // OpenAI-style nested details
    #[serde(default)]
    pub prompt_tokens_details: Option<PromptTokensDetails>,

    // Completion token details (OpenAI)
    #[serde(default)]
    pub completion_tokens_details: Option<HashMap<String, u32>>,

    // Catch unknown fields for future compatibility and debugging
    #[serde(flatten, skip_serializing)]
    pub extra: HashMap<String, serde_json::Value>,
}

// Normalized internal representation after processing
#[derive(Debug, Clone, Serialize, Default, PartialEq)]
pub struct NormalizedUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub total_tokens: u32,
    pub cache_creation_input_tokens: u32,
    pub cache_read_input_tokens: u32,

    // Metadata for debugging and analysis
    pub calculation_source: String,
    pub raw_data_available: Vec<String>,
}

impl NormalizedUsage {
    /// Get tokens that count toward context window
    /// This includes all tokens that consume context window space
    /// Output tokens from this turn will become input tokens in the next turn
    pub fn context_tokens(&self) -> u32 {
        self.input_tokens
            + self.cache_creation_input_tokens
            + self.cache_read_input_tokens
            + self.output_tokens
    }

    /// Get total tokens for cost calculation
    /// Priority: use total_tokens if available, otherwise sum all components
    pub fn total_for_cost(&self) -> u32 {
        if self.total_tokens > 0 {
            self.total_tokens
        } else {
            self.input_tokens
                + self.output_tokens
                + self.cache_creation_input_tokens
                + self.cache_read_input_tokens
        }
    }

    /// Get the most appropriate token count for general display
    /// For OpenAI format: use total_tokens directly
    /// For Anthropic format: use context_tokens (input + cache)
    pub fn display_tokens(&self) -> u32 {
        // For Claude/Anthropic format: prefer input-related tokens for context window display
        let context = self.context_tokens();
        if context > 0 {
            return context;
        }

        // For OpenAI format: use total_tokens when no input breakdown available
        if self.total_tokens > 0 {
            return self.total_tokens;
        }

        // Fallback to any available tokens
        self.input_tokens.max(self.output_tokens)
    }
}

impl RawUsage {
    /// Convert raw usage data to normalized format with intelligent token inference
    pub fn normalize(self) -> NormalizedUsage {
        let mut result = NormalizedUsage::default();
        let mut sources = Vec::new();

        // Collect available raw data fields
        let mut available_fields = Vec::new();
        if self.input_tokens.is_some() {
            available_fields.push("input_tokens".to_string());
        }
        if self.output_tokens.is_some() {
            available_fields.push("output_tokens".to_string());
        }
        if self.total_tokens.is_some() {
            available_fields.push("total_tokens".to_string());
        }
        if self.cache_creation_input_tokens.is_some() {
            available_fields.push("cache_creation".to_string());
        }
        if self.cache_read_input_tokens.is_some() {
            available_fields.push("cache_read".to_string());
        }

        result.raw_data_available = available_fields;

        // Extract directly available values
        let input = self.input_tokens.unwrap_or(0);
        let output = self.output_tokens.unwrap_or(0);
        let total = self.total_tokens.unwrap_or(0);

        // Handle cache tokens with fallback to OpenAI nested format
        let cache_read = self
            .cache_read_input_tokens
            .or_else(|| {
                self.prompt_tokens_details
                    .as_ref()
                    .and_then(|d| d.cached_tokens)
            })
            .unwrap_or(0);

        let cache_creation = self.cache_creation_input_tokens.unwrap_or(0);

        // Token calculation logic - prioritize total_tokens for OpenAI format
        let final_total = if total > 0 {
            sources.push("total_tokens_direct".to_string());
            total
        } else if input > 0 || output > 0 || cache_read > 0 || cache_creation > 0 {
            let calculated = input + output + cache_read + cache_creation;
            sources.push("total_from_components".to_string());
            calculated
        } else {
            0
        };

        // Final assignment
        result.input_tokens = input;
        result.output_tokens = output;
        result.total_tokens = final_total;
        result.cache_creation_input_tokens = cache_creation;
        result.cache_read_input_tokens = cache_read;
        result.calculation_source = sources.join("+");

        result
    }
}

// Legacy alias for backward compatibility
pub type Usage = RawUsage;

#[derive(Debug, Clone, Deserialize)]
pub struct Message {
    #[serde(default)]
    pub id: Option<String>,
    pub usage: Option<Usage>,
    pub model: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TranscriptEntry {
    #[serde(alias = "kind")]
    pub r#type: Option<String>,
    pub message: Option<Message>,
    #[serde(default, alias = "requestId")]
    pub request_id: Option<String>,
    #[serde(default, alias = "ts")]
    pub timestamp: Option<String>,
    #[serde(default)]
    pub payload: Option<TranscriptPayload>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TranscriptPayload {
    #[serde(default)]
    pub r#type: Option<String>,
    #[serde(default)]
    pub info: Option<TokenCountInfo>,
    #[serde(default)]
    pub model: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TokenCountInfo {
    #[serde(default)]
    pub total_token_usage: Option<TokenUsageBreakdown>,
    #[serde(default)]
    pub last_token_usage: Option<TokenUsageBreakdown>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TokenUsageBreakdown {
    #[serde(default)]
    pub input_tokens: Option<u32>,
    #[serde(default)]
    pub cached_input_tokens: Option<u32>,
    #[serde(default)]
    pub output_tokens: Option<u32>,
    #[serde(default)]
    pub reasoning_output_tokens: Option<u32>,
    #[serde(default)]
    pub total_tokens: Option<u32>,
}
