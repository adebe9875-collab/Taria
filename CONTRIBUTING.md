# Contributing

1. Fork or branch from `main`.
2. Keep package-manager calls argument-safe; do not use `sh -c` or interpolate user input into shell commands.
3. Keep APT, Snap and Pacman behavior isolated in `src/backend.rs`.
4. Never install arbitrary GitHub source code. GitHub application installs must use a reviewed catalog entry and verified release artifact.
5. Run these checks before opening a pull request:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo build --release
```

Changes that install, remove, update or replace executables must explain privilege and rollback behavior in the pull request.
