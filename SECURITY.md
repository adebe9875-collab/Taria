# Security policy

## Package operations

Taria delegates package installation and removal to native APT/DPKG, Snap or Pacman tools. It does not construct shell command strings and does not execute arbitrary GitHub repository source code. Administrator operations require explicit invocation with `sudo taria ...`.

## GitHub and self-updates

The built-in self-update source is fixed to `adebe9875-collab/Taria`. The updater requires a matching SHA-256 asset and refuses a missing or mismatched checksum. SHA-256 is an integrity check, not an independent publisher signature; signed releases are planned.

Taria does not automatically upload local logs. `taria report` sends only the search text the user explicitly supplies to GitHub's public issue-search endpoint.

## Reporting vulnerabilities

Please use the repository's private security advisory feature where available. Do not post exploitable details in a public issue.
