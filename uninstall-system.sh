#!/usr/bin/env bash
set -euo pipefail
if [[ "${EUID}" -ne 0 ]]; then
  echo "Run: sudo $0" >&2
  exit 1
fi
rm -f /usr/local/bin/taria
rm -f /usr/local/share/man/man1/taria.1 /usr/local/share/man/man1/taria.1.gz
rm -f /usr/share/bash-completion/completions/taria
mandb -q 2>/dev/null || true
echo "Taria system integration removed. User logs in ~/.local/state/taria were kept."
