# Security Policy

## Reporting a Vulnerability

PkgSeal handles package provenance and privileged installation flows. Security issues must not be reported publicly before coordinated disclosure.

- **Preferred channel:** Open a private security advisory on GitHub (Security → Advisories) or contact the maintainers via the email listed in the repository's security contacts.
- **Do not** file a public issue for sensitive findings, especially those involving privilege escalation, command injection, or AUR content handling.

Include:

- affected version / commit
- reproduction steps or proof-of-concept (redacted if needed)
- impact assessment
- suggested mitigation if you have one

We aim to acknowledge reports within **72 hours** and provide a remediation timeline once the issue is confirmed.

## Scope

In scope for this repository:

- `engine/policy` and `engine/domain` trust decisions
- Tauri IPC boundary (`apps/desktop/src-tauri`)
- source adapters (`sources/*`) — parsing, network, PKGBUILD static analysis
- privileged helper / Polkit integration (`platform/linux`)
- CI secret scanning and forbidden-pattern checks (` .github/workflows/ci.yml`)

Out of scope (but still appreciated if reported):

- Vulnerabilities inside upstream packages themselves (report to their publishers)
- Flatpak/AUR/Arch repository content — PkgSeal reports evidence, it does not claim upstream is vulnerability-free

## Security Model (summary)

- AUR `PKGBUILD` is treated as **untrusted data** — never sourced or executed for inspection.
- The frontend (React/WebView) is unprivileged and may only invoke typed Tauri commands — no generic `run_as_root(command)`.
- Policy is deterministic (`Evidence -> Policy -> Recommendation`) — no opaque `97/100` scores.
- `cargo audit`, `bun audit`, and `trufflehog` run in CI (`security` job).

## Supported Versions

Pre-`v0.1-alpha` — foundation stage, no stable release yet. Security fixes are applied to `dev` and the active release branch.

See also: `README.md` (Security section), `docs/adr/001-core-architecture.md` (threat model), `AGENTS.md` (shell/AUR safety rules).
