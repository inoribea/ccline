use super::Segment;
use crate::config::{InputData, ProviderKind};
use crate::utils::transcript::parse_latest_usage;

const CONTEXT_LIMIT: u32 = 200000;

pub struct UsageSegment {
    enabled: bool,
}

impl UsageSegment {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }
}

impl Segment for UsageSegment {
    fn render(&self, input: &InputData) -> String {
        if !self.enabled {
            return String::new();
        }

        let context_used_token = parse_transcript_usage(&input.transcript_path, input.provider);
        let context_used_rate = (context_used_token as f64 / CONTEXT_LIMIT as f64) * 100.0;

        // Format percentage: show integer when whole number, decimal when fractional
        let percentage_display = if context_used_rate.fract() == 0.0 {
            format!("{:.0}%", context_used_rate)
        } else {
            format!("{:.1}%", context_used_rate)
        };

        // Format tokens: show integer k when whole number, decimal k when fractional
        let tokens_display = if context_used_token >= 1000 {
            let k_value = context_used_token as f64 / 1000.0;
            if k_value.fract() == 0.0 {
                format!("{}k", k_value as u32)
            } else {
                format!("{:.1}k", k_value)
            }
        } else {
            context_used_token.to_string()
        };

        format!(
            "\u{f49b} {} · {} tokens",
            percentage_display, tokens_display
        )
    }

    fn enabled(&self) -> bool {
        self.enabled
    }
}

fn parse_transcript_usage<P: AsRef<std::path::Path>>(
    transcript_path: P,
    provider: ProviderKind,
) -> u32 {
    parse_latest_usage(transcript_path, Some(provider))
        .map(|usage| usage.display_tokens())
        .unwrap_or(0)
}
