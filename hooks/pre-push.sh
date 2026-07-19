#!/usr/bin/env bash
set -euo pipefail

echo "pre-push: running tests..."
cargo test --quiet
