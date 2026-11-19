# Taria

**Taria is a terminal-native, open-source package manager for Linux.** It provides one command interface over native package managers and searches GitHub for upstream project information.

> This is the CLI product. It intentionally has no GTK desktop window.

- Language: Rust
- License: MIT
- Primary target: Ubuntu/Debian
- Additional backends: Snap where installed; Pacman on Arch-based systems
- GitHub: https://github.com/adebe9875-collab/Taria

## How it works

Taria is an orchestrator, not a replacement for APT, DPKG, Snap, or Pacman. It delegates package transactions to the distribution's native tools, preserving native dependency resolution and package signature checks.

| Source | Search | Install/remove | Update |
|---|---:|---:|---:|
| APT/DPKG | Yes | Yes | Yes |
| Snap | Yes, if installed | Yes, if installed | Yes |
| Pacman | Yes, on Arch-based systems | Yes, on Arch-based systems | Safe full sync/upgrade |
| GitHub | Repository discovery and Taria's own releases | No arbitrary source-code execution | SHA-256 verified Taria releases |

On Ubuntu, Taria uses APT and Snap where available. Pacman is used only where installed. GitHub repository search is for discovering projects; arbitrary repositories are not executed or installed as system packages. Community GitHub installs will require curated manifests with explicit publishers, supported architectures, exact release assets and checksums.

## Commands

```bash
# Search across installed package sources and GitHub
taria search firefox
taria s firefox
taria search firefox --source apt
taria search firefox --source snap
taria search firefox --source github

# Package information
taria info firefox --source apt

# Install and remove
sudo taria install firefox
sudo taria i firefox
sudo taria remove firefox
sudo taria r firefox
sudo taria remove firefox --purge

# Installed packages and updates
taria list
sudo taria update
sudo taria upgrade
sudo taria upgrade --full

# Diagnostics and GitHub recovery
taria doctor
taria report "short error description"
taria self-update --check
sudo taria self-update
```

Search, info, list and diagnostics do not need root. Package changes and updates require explicit `sudo`; Taria does not silently elevate privileges.

Use `--source apt|snap|pacman` where a package exists in more than one ecosystem. For example, Ubuntu's Firefox APT package is a distribution integration package that installs the Snap build.

## Build on Ubuntu

Requirements: Rust stable (1.85 or newer), Cargo, build-essential and pkg-config. This CLI release does not need GTK or PolicyKit development packages.

```bash
sudo apt update
sudo apt install -y build-essential pkg-config
cargo build --release
./target/release/taria --help
```

The binary is created at `target/release/taria`.

## Integrate with the system

After building, from the project root:

```bash
sudo ./install-system.sh
```

This installs `/usr/local/bin/taria`, a manual page and Bash completion. It does not replace native package managers, modify package repositories, or install a background daemon.

Uninstall the integration with:

```bash
sudo ./uninstall-system.sh
```

## GitHub recovery and self-update

The built-in repository is `adebe9875-collab/Taria`.

- Fatal errors are written locally to `~/.local/state/taria/errors.log`.
- On a fatal error, Taria searches the project's issue tracker using a generic error label and checks whether a newer stable release exists. It does not upload local logs or private paths.
- `taria doctor` checks package backends and the latest stable Taria release.
- `taria report "message"` sends only the search text explicitly supplied by the user to GitHub issue search.
- `taria self-update --check` checks for an update without changing files.
- `sudo taria self-update` downloads the release binary and its SHA-256 file, verifies the digest, stages the executable beside the current binary, then atomically replaces it.

Taria does not silently replace itself following an error. It detects and reports an available update; applying that update remains an explicit administrator action. SHA-256 protects against a corrupted or mismatched download, while separately signed release verification is planned.

The tagged GitHub Actions release workflow publishes these x86_64 assets:

- `taria-linux-x86_64`
- `taria-linux-x86_64.sha256`

AArch64 publishing is planned; the updater refuses to update an architecture for which a matching release asset does not exist.

## Project layout

```text
Taria/
├── src/                 # Rust CLI, package backends, GitHub and updater
├── catalog/             # reviewed GitHub package manifests (future installs)
├── completions/         # Bash completion
├── man/                 # manual page
└── .github/workflows/   # CI and tagged release builds
```

## Status

This is the CLI-focused 0.3.0 development release. The native package-manager adapters, GitHub discovery, diagnostics, local error logging and checksum-verified self-update are implemented. It still requires an end-to-end build and tests on Ubuntu 26.10 and Arch. Curated GitHub app installation, signed updates and broader distribution packaging remain roadmap items.
