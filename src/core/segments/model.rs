use super::Segment;
use crate::config::InputData;

pub struct ModelSegment {
    enabled: bool,
}

impl ModelSegment {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }
}

impl Segment for ModelSegment {
    fn render(&self, input: &InputData) -> String {
        if !self.enabled {
            return String::new();
        }

        format!(
            "\u{e26d} {}",
            self.format_model_name(input.model.identifier.as_deref(), &input.model.display_name)
        )
    }

    fn enabled(&self) -> bool {
        self.enabled
    }
}

impl ModelSegment {
    fn format_model_name(&self, identifier: Option<&str>, display_name: &str) -> String {
        let source = identifier.unwrap_or(display_name);

        let formatted = match source {
            name if name.contains("claude-3-5-sonnet") => "Sonnet 3.5",
            name if name.contains("claude-3-7-sonnet") => "Sonnet 3.7",
            name if name.contains("claude-3-sonnet") => "Sonnet 3",
            name if name.contains("claude-3-haiku") => "Haiku 3",
            name if name.contains("claude-4-sonnet") => "Sonnet 4",
            name if name.contains("claude-4-opus") => "Opus 4",
            name if name.contains("sonnet-4") => "Sonnet 4",
            name if name.contains("gpt-5-codex") => "GPT-5 Codex",
            name if name.contains("codex") => "Codex",
            _ => display_name,
        };

        formatted.to_string()
    }
}
