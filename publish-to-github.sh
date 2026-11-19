#!/usr/bin/env bash
set -euo pipefail

# Run this from the Taria project directory after installing Git + GitHub CLI.
if ! command -v git >/dev/null 2>&1; then
  echo "Git is required. Install it with: sudo apt install git" >&2
  exit 1
fi
if ! command -v gh >/dev/null 2>&1; then
  echo "GitHub CLI is required. Install it, then run: gh auth login" >&2
  exit 1
fi
if ! gh auth status >/dev/null 2>&1; then
  echo "Please sign in first with: gh auth login" >&2
  exit 1
fi

GH_LOGIN="$(gh api user --jq .login)"
GH_ID="$(gh api user --jq .id)"
git config user.name "$GH_LOGIN"
git config user.email "${GH_ID}+${GH_LOGIN}@users.noreply.github.com"

git init -b main
git add .
if ! git rev-parse --verify HEAD >/dev/null 2>&1; then
  git commit -m "Initial Taria package manager source"
else
  git commit -m "Update Taria package manager source" || true
fi
gh repo create Taria --public --description "Open-source Rust and GTK package manager for Linux" --source . --remote origin --push
