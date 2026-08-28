# PkgSeal

> **Find the right Linux package. Understand why. Install it with confidence.**

PkgSeal is a desktop application for discovering, comparing, evaluating, and eventually installing Linux software from multiple package sources.

Instead of asking users to choose blindly between an official repository package, AUR package, Flatpak, vendor package, AppImage, or another distribution format, PkgSeal collects the available options, explains their provenance and trade-offs, and recommends the most appropriate source according to an explicit policy.

PkgSeal starts with **Arch Linux**, but its core architecture is intentionally designed to support other Linux distributions later.

---

## Why PkgSeal?

Installing an application on Linux is often less obvious than it should be.

Searching for a single application can produce several legitimate installation paths:

```text
Brave Browser
├── Arch repository
├── AUR
├── Flatpak
├── vendor package
└── other third-party distribution
```

Those options are not equivalent.

They may differ in:

- publisher provenance;
- package maintainer;
- sandboxing;
- permissions;
- update mechanism;
- native integration;
- package signatures;
- build scripts;
- dependency model;
- upstream support;
- security trade-offs.

Most package managers are excellent at managing **their own source**.

PkgSeal focuses on the layer above them:

```text
Search
  ↓
Discover candidates
  ↓
Resolve application identity
  ↓
Collect evidence
  ↓
Apply policy
  ↓
Recommend
  ↓
Preview transaction
  ↓
Install
```

The goal is not to hide complexity behind a bigger **Install** button.

The goal is to make the decision understandable.

---

## Core principle

> **PkgSeal must never hide a trust decision behind an Install button.**

PkgSeal does not claim that a package is “100% safe”.

Instead, recommendations are based on explicit evidence such as:

```text
✓ package from an official repository
✓ publisher-supported installation method
✓ verified publisher
✓ package signature
✓ checksum present
✓ sandboxed application

⚠ community-maintained package
⚠ broad filesystem permissions
⚠ build logic changed since previous release
⚠ unverified publisher
```

No opaque `97/100 security score`.

The user should always be able to understand **why** PkgSeal recommends one candidate over another.

---

## Example

A future application page could look conceptually like this:

```text
Brave Browser

Recommended
brave-bin · AUR

Why PkgSeal recommends this

✓ Publisher-supported installation path
✓ Native Chromium sandbox behavior
✓ Checksum available
⚠ Community-maintained AUR recipe

Alternatives
├── Flatpak
└── other available candidates

[Review evidence]                     [Install]
```

For an AUR update:

```text
PKGBUILD changed

Version
1.2.0 → 1.3.0

Source URL
changed

Checksum
changed

Build logic
unchanged
```

PkgSeal should make package provenance understandable without requiring every user to manually audit every source from scratch.

---

## Initial package sources

The first supported ecosystem is Arch Linux.

PkgSeal initially targets:

- **Arch official repositories**
- **AUR**
- **Flatpak / Flathub**

The architecture is designed so future adapters can be added without rewriting the core decision engine:

```text
sources/
├── arch
├── aur
├── flatpak
├── debian       # future
├── fedora       # future
├── nix          # future
├── snap         # future
└── appimage     # future
```

---

## Product policies

Recommendations are policy-driven rather than hard-coded around a universal source ranking.

Initial policy ideas include:

### Balanced

Balances:

- provenance;
- upstream support;
- security;
- sandboxing;
- native integration;
- maintainability.

### Native First

Prefers native packages when their trust and maintenance characteristics are comparable.

### Sandbox First

Prefers sandboxed desktop applications when their permissions remain reasonable.

### Maximum Review

Requires stronger review before accepting community-maintained or broadly privileged packages.

A rule such as:

```text
Arch > Flatpak > AUR
```

is intentionally **not** treated as universally correct.

The right choice can depend on the specific application.

---

## Architecture

PkgSeal is organized by product responsibility rather than by language.

```text
pkgseal/
│
├── apps/
│   └── desktop/
│
├── engine/
│   ├── domain/
│   ├── resolver/
│   ├── policy/
│   └── transactions/
│
├── sources/
│   ├── arch/
│   ├── aur/
│   └── flatpak/
│
├── platform/
│   └── linux/
│
├── testkit/
├── fixtures/
├── docs/
└── scripts/
```

The repository should be readable as:

```text
apps       → what the user runs
engine     → what PkgSeal thinks and decides
sources    → where package information comes from
platform   → how PkgSeal interacts with Linux
testkit    → reusable test infrastructure
fixtures   → deterministic source data for tests
docs       → why architectural decisions exist
```

Detailed architecture:

- [`docs/architecture/overview.md`](docs/architecture/overview.md)
- [`docs/adr/001-core-architecture.md`](docs/adr/001-core-architecture.md)

---

## Desktop architecture

The desktop client uses:

- **Tauri 2**
- **React**
- **TypeScript**
- **Vite**
- **Tailwind CSS**
- **shadcn/ui**
- **Base UI**
- **TanStack Query**
- **Rust**

The frontend is intentionally treated as an unprivileged UI layer.

```text
React UI
   ↓ typed IPC
Tauri application boundary
   ↓
PkgSeal engine
   ↓
source adapters / Linux platform
   ↓
typed transaction
   ↓
privileged helper + Polkit
```

The frontend must never receive a generic API such as:

```text
run_as_root(command)
```

Privileged actions must remain narrow and typed.

---

## Rust core

PkgSeal's sensitive logic lives in Rust.

Initial core modules:

```text
engine/domain
engine/resolver
engine/policy
engine/transactions
```

Source-specific integration remains outside the domain:

```text
sources/arch
sources/aur
sources/flatpak
```

Important design rules:

- domain code performs no network or system IO;
- the policy engine is deterministic;
- source adapters do not define product policy;
- transaction plans are inspectable before execution;
- shell strings from user input are never executed;
- privilege boundaries are explicit.

---

## AUR security model

AUR packages are treated as community-provided content.

A `PKGBUILD` is data to inspect, not code to trust.

PkgSeal must **never** do this merely to analyze a package:

```bash
source PKGBUILD
```

or:

```bash
bash PKGBUILD
```

Static inspection may highlight patterns such as:

- `curl | sh`;
- `wget | sh`;
- `eval`;
- unexpected privilege escalation;
- setuid changes;
- root ownership changes;
- suspicious decoding or obfuscation;
- downloaded code executed during build;
- install scripts;
- unexpected network access.

A finding is not automatically proof of malicious behavior.

PkgSeal should report facts and context rather than produce alarmist conclusions.

---

## Flatpak model

For Flatpak applications, PkgSeal should surface details such as:

- publisher verification;
- application ID;
- remote;
- filesystem permissions;
- network access;
- Wayland/X11 access;
- device access;
- D-Bus permissions;
- runtime;
- sandbox characteristics.

A **verified publisher** means provenance has stronger evidence.

It does **not** mean the application is guaranteed to be vulnerability-free.

---

## Transaction model

PkgSeal will eventually install and remove software, but installation is intentionally not the first milestone.

Every mutation must first become a typed transaction plan:

```text
InstallTransaction
├── source
├── package
├── version
├── expected download
├── expected disk change
├── privileges required
└── operations
```

Transaction states:

```text
Planned
AwaitingConfirmation
Authorizing
Running
Succeeded
Failed
Cancelled
```

The user should know what will happen before the machine is modified.

---

## Privilege model

PkgSeal must never store the user's sudo password.

The target privilege architecture is:

```text
Desktop app
   ↓
Typed privileged request
   ↓
Polkit authorization
   ↓
Minimal privileged helper
   ↓
Specific package operation
```

Allowed design:

```text
install_arch_packages([...])
remove_arch_packages([...])
```

Forbidden design:

```text
run_command_as_root("...")
```

---

## Design direction

PkgSeal should feel like a polished desktop product, not a thin GUI over package-manager commands.

The visual language targets:

- compact desktop density;
- neutral surfaces;
- strong typography;
- restrained borders;
- subtle depth;
- one primary accent;
- clear success/warning/danger semantics;
- excellent keyboard navigation;
- dark and light themes;
- accessible focus and contrast;
- short, purposeful motion.

The interface should remain calm even when displaying security information.

No “hacker UI”.

No warning-red everywhere.

---

## Development strategy

PkgSeal is intentionally built **read-only first**.

### Phase 0 — Foundation

- Tauri shell
- React/Vite
- Rust workspace
- design system
- CI
- test infrastructure
- SQLite foundation
- security boundaries

### Phase 1 — Package explorer

Implement read-only search for:

- Arch
- AUR
- Flatpak

### Phase 2 — Resolver

Group package candidates belonging to the same real application.

### Phase 3 — Evidence

Collect:

- provenance;
- verification;
- permissions;
- AUR metadata;
- PKGBUILD findings.

### Phase 4 — Policy engine

Produce deterministic and explainable recommendations.

### Phase 5 — Transaction preview

Generate install/remove plans without executing them.

### Phase 6 — Arch transactions

Introduce controlled native package operations.

### Phase 7 — Flatpak transactions

Add Flatpak installation and removal.

### Phase 8 — AUR transactions

Only after the review and privilege model is mature.

### Phase 9 — Product polish

- keyboard-first UX;
- command palette;
- accessibility;
- HiDPI;
- performance;
- animations;
- release hardening.

---

## First milestone

The first meaningful version of PkgSeal will **not install anything**.

It should perform this pipeline extremely well:

```text
Search "Brave"
        ↓
Discover all candidates
        ↓
Resolve identity
        ↓
Inspect evidence
        ↓
Compare sources
        ↓
Recommend one
        ↓
Explain why
```

Reference applications for the first evaluation corpus:

- Brave
- Bitwarden
- Discord
- Spotify
- Visual Studio Code
- Steam
- Obsidian

If PkgSeal cannot reliably explain those decisions, it is not ready to modify the system.

---

## Testing philosophy

Package management is too sensitive for “works on my machine”.

PkgSeal uses:

- Rust unit tests;
- policy decision matrices;
- deterministic fixtures;
- frontend component tests;
- IPC boundary tests;
- integration tests;
- disposable Arch environments for real package transactions.

Network responses used by core tests should be captured as fixtures.

Real installation tests must never run against the developer's workstation.

---

## Quality gates

Every merge should eventually require:

```text
Frontend
├── lint
├── typecheck
├── tests
└── build

Rust
├── cargo fmt --check
├── cargo clippy -- -D warnings
├── cargo test
└── cargo build

Security
├── dependency audit
├── secret scanning
└── forbidden-pattern checks
```

---

## Project status

**Current status: early architecture / foundation**

PkgSeal is currently defining its architecture, trust model, UI foundations, and first read-only vertical slice.

The project is **not ready for production use** and should not yet be trusted to modify a real system.

---

## Road to `v0.1-alpha`

`v0.1-alpha` is complete when:

- PkgSeal launches cleanly on Arch Linux;
- the desktop design system is stable;
- Arch/AUR/Flatpak search works;
- equivalent package candidates are grouped correctly;
- provenance and security evidence are visible;
- the policy engine produces explainable recommendations;
- critical tests work without network access;
- CI is green;
- no arbitrary shell execution is exposed;
- **no system mutation is possible yet**.

---

## Security

Security issues should not be reported publicly before a responsible disclosure process exists.

A dedicated policy will live in:

[`SECURITY.md`](SECURITY.md)

PkgSeal itself follows the same principle it asks of package sources:

> provenance, review, explicit trust boundaries, and explainable decisions.

---

## Contributing

PkgSeal is currently in its foundation stage.

Contribution guidelines will be introduced once:

- repository conventions are stable;
- CI is running;
- the first source adapter contracts are finalized;
- coding and testing standards are enforced automatically.

Until then, architecture changes should be documented through ADRs.

---

## Documentation

Important project documents:

```text
docs/
├── adr/
│   └── 001-core-architecture.md
│
├── architecture/
│   └── overview.md
│
├── security/
└── product/
```

Start here:

- [Core Architecture ADR](docs/adr/001-core-architecture.md)
- [Architecture Overview](docs/architecture/overview.md)

---

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option. See `Cargo.toml:17-20` (`license = "MIT OR Apache-2.0"`).

SPDX: `MIT OR Apache-2.0`. Security policy: [SECURITY.md](SECURITY.md).

---

<p align="center">
  <strong>PkgSeal</strong><br />
  Find the right package. Understand why. Install it with confidence.
</p>
