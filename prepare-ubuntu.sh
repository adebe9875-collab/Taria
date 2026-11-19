#!/usr/bin/env bash
set -euo pipefail

if ! command -v cargo >/dev/null 2>&1 || ! command -v rustc >/dev/null 2>&1; then
  echo "Rust and Cargo are required. Install the stable Rust toolchain from https://rustup.rs/ first." >&2
  exit 1
fi

printf 'Rust: '; rustc --version
printf 'Cargo: '; cargo --version
printf '\nTaria CLI needs no GTK or PolicyKit development packages.\n'
if command -v apt-cache >/dev/null 2>&1; then echo 'Detected package backend: APT/DPKG'; fi
if command -v snap >/dev/null 2>&1; then echo 'Detected package backend: Snap'; fi
if command -v pacman >/dev/null 2>&1; then echo 'Detected package backend: Pacman'; fi
