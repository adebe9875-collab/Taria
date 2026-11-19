#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN="$ROOT/target/release/taria"
if [[ ! -x "$BIN" ]]; then
  echo "Build Taria first: cargo build --release" >&2
  exit 1
fi

install -Dm755 "$BIN" "$HOME/.local/bin/taria"
install -Dm644 "$ROOT/assets/taria-icon.png" "$HOME/.local/share/icons/hicolor/1024x1024/apps/taria.png"
install -d "$HOME/.local/share/applications"
cat > "$HOME/.local/share/applications/taria.desktop" <<'DESKTOP'
[Desktop Entry]
Name=Taria
Comment=Open-source Linux package manager
Exec=taria
Icon=taria
Terminal=false
Type=Application
Categories=System;PackageManager;Utility;
Keywords=packages;software;install;update;remove;
DESKTOP

if ! printf '%s' "$PATH" | tr ':' '\n' | grep -Fxq "$HOME/.local/bin"; then
  echo "Add this line to ~/.profile if Taria is not found after restarting your session:" 
  echo 'export PATH="$HOME/.local/bin:$PATH"'
fi

echo "Taria installed for the current user. Open it from the Applications menu or run: taria"
