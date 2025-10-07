#!/usr/bin/env bash
# Test CCometixLine with Codex sample data

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Build the project first
echo "Building CCometixLine..."
cd "$PROJECT_ROOT"
cargo build --release

BINARY="$PROJECT_ROOT/target/release/ccometixline"

if [ ! -f "$BINARY" ]; then
    echo "Error: Binary not found at $BINARY"
    exit 1
fi

echo "Testing Codex statusline generation..."
echo

# Test 1: Basic Codex input
echo "=== Test 1: Basic Codex Input ==="
echo '{
  "model": "gpt-5-codex",
  "workspace": {"cwd": "/home/user/project"},
  "transcriptPath": "/tmp/codex-test-session.jsonl"
}' | "$BINARY"
echo

# Test 2: O3 model
echo "=== Test 2: O3 Model ==="
echo '{
  "model": "o3",
  "workspace": {"cwd": "/home/user/workspace"},
  "transcriptPath": "/home/user/.codex/sessions/demo.jsonl"
}' | "$BINARY"
echo

# Test 3: O4-mini model
echo "=== Test 3: O4-mini Model ==="
echo '{
  "model": "o4-mini",
  "workspace": {"cwd": "/tmp/test"},
  "transcriptPath": "/home/user/.codex/sessions/test.jsonl"
}' | "$BINARY"
echo

# Test 4: Create a sample Codex transcript and test with it
echo "=== Test 4: With Sample Transcript ==="
TEMP_TRANSCRIPT=$(mktemp /tmp/codex-transcript-XXXXXX.jsonl)

cat > "$TEMP_TRANSCRIPT" <<'EOF'
{"ts":"2025-10-07T10:00:00.000Z","dir":"meta","kind":"session_start","cwd":"/home/user/project","model":"gpt-5-codex","model_provider_id":"openai","model_provider_name":"OpenAI"}
{"ts":"2025-10-07T10:00:05.000Z","dir":"to_tui","kind":"event_msg","payload":{"type":"token_count","model":"gpt-5-codex","info":{"total_token_usage":{"input_tokens":1500,"cached_input_tokens":200,"output_tokens":300,"reasoning_output_tokens":50,"total_tokens":2050},"last_token_usage":{"input_tokens":1500,"cached_input_tokens":200,"output_tokens":300,"reasoning_output_tokens":50,"total_tokens":2050}}}}
{"ts":"2025-10-07T10:01:00.000Z","dir":"to_tui","kind":"event_msg","payload":{"type":"token_count","model":"gpt-5-codex","info":{"total_token_usage":{"input_tokens":3000,"cached_input_tokens":500,"output_tokens":800,"reasoning_output_tokens":150,"total_tokens":4450},"last_token_usage":{"input_tokens":1500,"cached_input_tokens":300,"output_tokens":500,"reasoning_output_tokens":100,"total_tokens":2400}}}}
EOF

echo "Created test transcript: $TEMP_TRANSCRIPT"
cat "$TEMP_TRANSCRIPT"
echo

echo "Testing with sample transcript:"
echo "{
  \"model\": \"gpt-5-codex\",
  \"workspace\": {\"cwd\": \"$PROJECT_ROOT\"},
  \"transcriptPath\": \"$TEMP_TRANSCRIPT\"
}" | "$BINARY"
echo

# Cleanup
rm -f "$TEMP_TRANSCRIPT"
echo "Test complete!"
