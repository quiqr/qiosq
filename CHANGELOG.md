# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project aims
to follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Maintainers: add entries under **Unreleased** as you work; `scripts/release.sh`
moves them into a dated version section at release time (see the README's
*Releasing* section).

## [Unreleased]

### Added
- Milestone M1 — the two-pane PoC alpha base, proven end to end against a real
  Quiqr Server in a NixOS VM (`nix flake check`):
  - `qtui-config` — load + validate the TOML config with clear per-field errors.
  - `qtui-storage` — enumerate Quiqr sites (real `sites/<name>/main/` layout) and
    a read-only, filtered `content/` tree.
  - `qtui-model` — tolerant parse of `quiqr/model/` into a `NavigationModel`.
  - `qtui-ui` — WordPerfect-5.1 two-pane shell, function-key legend, and the
    `SiteList → Browse → ViewFile → Agent` mode state machine.
  - `qtui-preview` — `hugo server` lifecycle (free port, never `:13131`).
  - `qtui-agent` — the `Agent` trait, `@path` intent injection, completion
    sentinel, and a fake agent for tests.
  - `qtui` — the host event loop (interactive + headless `--script`).
- `qtui init` — detect or ask where the Quiqr storage is, then write
  `~/.config/qiosq/config.toml`; bare `qtui` auto-loads it.
- `scripts/release.sh` — release management (version bump, changelog, tag, push,
  GitHub release).

### Changed
- The Nix build derives its version from `Cargo.toml` (single source of truth).
