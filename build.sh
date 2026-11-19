#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
cargo build --release
printf '\nBuilt: %s/target/release/taria\n' "$PWD"
