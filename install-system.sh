#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")" && pwd)"
BIN="$ROOT/target/release/taria"

if [[ "${EUID}" -ne 0 ]]; then
  echo "Run: sudo $0" >&2
  exit 1
fi
if [[ ! -x "$BIN" ]]; then
  echo "Missing $BIN. Build first with: cargo build --release" >&2
  exit 1
fi

install -Dm755 "$BIN" /usr/local/bin/taria
install -Dm644 "$ROOT/man/taria.1" /usr/local/share/man/man1/taria.1
gzip -f -k /usr/local/share/man/man1/taria.1
if [[ -d /usr/share/bash-completion/completions ]]; then
  install -Dm644 "$ROOT/completions/taria.bash" /usr/share/bash-completion/completions/taria
fi
mandb -q 2>/dev/null || true
printf 'Taria installed at /usr/local/bin/taria\n'
printf 'Try: taria --help\n'
