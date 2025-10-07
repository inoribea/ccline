use ccometixline::config::{InputData, ProviderKind};
use ccometixline::utils::transcript::parse_line_to_usage;
use std::collections::HashSet;

#[test]
fn test_codex_input_data_parsing() {
    let json_input = r#"{
        "model": "gpt-5-codex",
        "workspace": {"cwd": "/home/user/project"},
        "transcriptPath": "/home/user/.codex/sessions/test-session.jsonl"
    }"#;

    let input_data =
        InputData::from_reader(json_input.as_bytes()).expect("Should parse Codex input format");

    assert_eq!(input_data.provider, ProviderKind::Codex);
    assert_eq!(input_data.model.display_name, "GPT-5 Codex");
    assert_eq!(input_data.model.identifier.as_deref(), Some("gpt-5-codex"));
    assert_eq!(input_data.workspace.current_dir, "/home/user/project");
    assert_eq!(
        input_data.transcript_path,
        "/home/user/.codex/sessions/test-session.jsonl"
    );
}

#[test]
fn test_codex_transcript_parsing() {
    // Simulated Codex transcript line with token_count event
    let line = r#"{
        "ts": "2025-10-07T12:00:00.000Z",
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
                    "total_tokens": 1700
                },
                "last_token_usage": {
                    "input_tokens": 100,
                    "cached_input_tokens": 20,
                    "output_tokens": 50,
                    "reasoning_output_tokens": 10,
                    "total_tokens": 170
                }
            }
        }
    }"#;

    let mut seen = HashSet::new();
    let mut state = ccometixline::utils::transcript::TranscriptState::new();

    let usage = parse_line_to_usage(line, "test-session", &mut seen, &mut state)
        .expect("Should parse Codex transcript line");

    // Verify usage data
    assert_eq!(usage.session_id, "test-session");
    assert_eq!(usage.model, "gpt-5-codex");
    assert_eq!(usage.input_tokens, 100);
    assert_eq!(usage.cache_read_tokens, 20);
    // output_tokens should include reasoning tokens (50 + 10 = 60)
    assert_eq!(usage.output_tokens, 60);
    assert_eq!(usage.cache_creation_tokens, 0);

    // Verify state was updated
    assert_eq!(state.provider(), Some(ProviderKind::Codex));

    let normalized = state.last_normalized.as_ref().unwrap();
    assert_eq!(normalized.input_tokens, 100);
    assert_eq!(normalized.output_tokens, 60);
    assert_eq!(normalized.cache_read_input_tokens, 20);
    assert_eq!(normalized.total_tokens, 170);
}

#[test]
fn test_codex_model_detection() {
    let test_cases = vec![
        ("gpt-5-codex", ProviderKind::Codex),
        ("gpt-4", ProviderKind::Codex),
        ("o3", ProviderKind::Codex),
        ("o4-mini", ProviderKind::Codex),
    ];

    for (model_name, expected_provider) in test_cases {
        let json_input = format!(
            r#"{{
                "model": "{}",
                "workspace": {{"cwd": "/tmp"}},
                "transcriptPath": "/home/user/.codex/sessions/test.jsonl"
            }}"#,
            model_name
        );

        let input_data = InputData::from_reader(json_input.as_bytes()).expect("Should parse input");

        assert_eq!(
            input_data.provider, expected_provider,
            "Model {} should be detected as {:?}",
            model_name, expected_provider
        );
    }
}

#[test]
fn test_codex_path_detection() {
    // Test various Codex path formats
    let test_cases = vec![
        (
            r#"/home/user/.codex/sessions/test.jsonl"#,
            r#"/home/user/.codex/sessions/test.jsonl"#,
        ),
        (
            r#"C:\\Users\\user\\.codex\\sessions\\test.jsonl"#,
            r#"C:\\Users\\user\\.codex\\sessions\\test.jsonl"#,
        ),
        (
            r#"/Users/user/.codex/sessions/2025/10/demo.jsonl"#,
            r#"/Users/user/.codex/sessions/2025/10/demo.jsonl"#,
        ),
    ];

    for (display_path, json_path) in test_cases {
        let json_input = format!(
            r#"{{
                "model": "gpt-5-codex",
                "workspace": {{"cwd": "/tmp"}},
                "transcriptPath": "{}"
            }}"#,
            json_path
        );

        let input_data = InputData::from_reader(json_input.as_bytes())
            .expect(&format!("Should parse input with path: {}", display_path));

        assert_eq!(
            input_data.provider,
            ProviderKind::Codex,
            "Path {} should be detected as Codex",
            display_path
        );
    }
}

#[test]
fn test_codex_deduplication() {
    let line = r#"{
        "ts": "2025-10-07T12:00:00.000Z",
        "dir": "to_tui",
        "kind": "event_msg",
        "payload": {
            "type": "token_count",
            "model": "gpt-5-codex",
            "info": {
                "total_token_usage": {
                    "total_tokens": 1000
                },
                "last_token_usage": {
                    "input_tokens": 100,
                    "output_tokens": 50,
                    "total_tokens": 150
                }
            }
        }
    }"#;

    let mut seen = HashSet::new();
    let mut state = ccometixline::utils::transcript::TranscriptState::new();

    // First parse should succeed
    let first = parse_line_to_usage(line, "test-session", &mut seen, &mut state);
    assert!(first.is_some(), "First parse should return usage");

    // Second parse of same line should be deduplicated
    let second = parse_line_to_usage(line, "test-session", &mut seen, &mut state);
    assert!(second.is_none(), "Duplicate should be filtered out");
}

#[test]
fn test_codex_pricing_lookup() {
    use ccometixline::billing::ModelPricing;

    let pricing = ModelPricing::fallback_pricing();

    // Test exact matches
    assert!(pricing.contains_key("gpt-5-codex"));
    assert!(pricing.contains_key("o3"));
    assert!(pricing.contains_key("o4-mini"));

    // Verify pricing values for gpt-5-codex
    let gpt5_pricing = pricing.get("gpt-5-codex").unwrap();
    assert_eq!(gpt5_pricing.input_cost_per_1k, 0.00075); // $0.75/1M
    assert_eq!(gpt5_pricing.output_cost_per_1k, 0.006); // $6.00/1M

    // Test fuzzy matching
    let fuzzy_result = ModelPricing::get_model_pricing(&pricing, "gpt-5-codex-preview");
    assert!(fuzzy_result.is_some(), "Should find gpt-5-codex-preview");
}

#[test]
fn test_mixed_provider_detection() {
    // Test that Claude paths are still detected correctly
    let claude_input = r#"{
        "model": {"display_name": "claude-3-5-sonnet"},
        "workspace": {"current_dir": "/tmp"},
        "transcript_path": "/home/user/.claude/projects/test/session.jsonl"
    }"#;

    let claude_data = InputData::from_reader(claude_input.as_bytes()).unwrap();
    assert_eq!(claude_data.provider, ProviderKind::Claude);

    // Test Codex detection
    let codex_input = r#"{
        "model": "gpt-5-codex",
        "workspace": {"cwd": "/tmp"},
        "transcriptPath": "/home/user/.codex/sessions/test.jsonl"
    }"#;

    let codex_data = InputData::from_reader(codex_input.as_bytes()).unwrap();
    assert_eq!(codex_data.provider, ProviderKind::Codex);
}
