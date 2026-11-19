# Taria

**Taria** is an open-source Linux package manager project written in Rust. The first target is Ubuntu, with a GTK 4 desktop interface and an APT/DPKG backend. A small JSON catalog is included and can be mirrored from the project's GitHub repository.

> Status: early development (`0.1.0`). This is a starter implementation, not yet a replacement for APT or DPKG.

## Current features

- GTK 4 desktop window with a dark theme and orange accent.
- Search Ubuntu packages using `apt-cache`.
- View installed Debian packages using `dpkg-query`.
- CLI commands for search, package details, install, remove, update, upgrade, list, doctor, and version.
- Short command aliases: `s`, `i`, `r`, and `u`.
- Local JSON catalog plus optional GitHub-hosted catalog lookup.
- Package-name validation and argument-based process execution (no shell interpolation for package operations).

## Commands

```bash
taria                         # open the graphical interface
taria search firefox
taria s firefox                # search shortcut
taria info firefox
sudo taria install firefox
sudo taria i firefox           # install shortcut
sudo taria remove firefox
sudo taria r firefox           # remove shortcut
sudo taria update
sudo taria u                   # update shortcut
sudo taria upgrade
taria list
taria doctor
taria version
taria --help
```

Correction: the install shortcut is `sudo taria i firefox`; the update shortcut is `sudo taria u`.

## Build on Ubuntu

Install the GTK build dependencies:

```bash
sudo apt update
sudo apt install -y build-essential pkg-config libgtk-4-dev
```

Build the application:

```bash
cargo build --release
./target/release/taria
```

Install locally for the current user:

```bash
./install-local.sh
```

Make sure `~/.local/bin` is in your `PATH`.

## GitHub catalog

The app checks this catalog URL:

`https://raw.githubusercontent.com/adebe9875-collab/Taria/main/catalog/apps.json`

Until that public repository exists, online catalog lookup will be skipped and the bundled catalog remains available. A package catalog must contain trusted, reviewed metadata. Do not add arbitrary install scripts or unsigned binaries.

## Security model and current limits

- APT install/remove/update/upgrade operations require root. Run the relevant commands with `sudo`.
- The program passes package names as separate process arguments and rejects unexpected characters.
- The current GitHub catalog is metadata only. It does **not** download and execute arbitrary GitHub release assets.
- The current desktop interface searches APT and lists installed packages. Install/remove buttons and full progress reporting are planned, not yet implemented.
- Never run unreviewed install scripts from a repository with administrator privileges.

## License

MIT. See [LICENSE](LICENSE).
