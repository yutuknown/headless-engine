#!/bin/bash
# Build the engine first
# cargo build --release

# Run the engine and pipe commands to it
cat <<EOF | cargo run
{"method": "Navigate", "params": {"url": "https://www.google.com/search?q=headless+browser+evasion"}}
{"method": "ExtractDom", "params": {}}
{"method": "EvaluateJs", "params": {"code": "navigator.userAgent"}}
{"method": "Shutdown", "params": {}}
EOF
