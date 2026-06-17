---
# qiosq-sflc
title: E1 — Foundation & Nix
status: completed
type: epic
priority: high
created_at: 2026-06-17T15:56:53Z
updated_at: 2026-06-17T16:12:25Z
parent: qiosq-mer9
---

Goal: a reproducible workspace that builds and tests under Nix.

## Tasks
- [x] T1.1 Cargo workspace with the seven crates (empty but compiling).
- [x] T1.2 `flake.nix`: dev shell (rust toolchain, hugo, beans, openspec, agent CLI, fake-agent build), `nix build`, `nix flake check` wiring.
- [x] T1.3 `qtui-config`: TOML load + validation; example config; clear errors.
- [x] T1.4 CI entry: `nix flake check` runs unit tests.
- [x] T1.5 Initialize OpenSpec + beans in-repo; AGENTS.md present and correct.

## Tests (gate)
- [x] Config load/validate unit tests.
- [x] `nix flake check` green.

OpenSpec change: poc-foundation.

## Summary of Changes

- Cargo workspace with seven crates created; all compile. Pure crates (qtui-config, qtui-storage, qtui-model) exclude ratatui/rmux per architecture.
- qtui-config implemented: TOML load + two-phase validation, one distinct human-readable error per invalid field (empty data dir; port range incl. 13131; inverted/zero range; empty agent command; empty completion sentinel). Data dir resolved to absolute. 13 tests (11 unit + 2 integration) green, including loading the shipped example config.
- Fixed config/quiqr-tui.example.toml (removed the stray heredoc EOF line from the bootstrap bundle).
- flake.nix finalized: dev shell ships rust (clippy+rustfmt), hugo, git, node, beans (nixpkgs#beans), openspec (nixpkgs#openspec). nix build produces the qtui binary; checks.unit runs cargo test --workspace; checks.e2e NixOS VM boots to multi-user.target (full scenario TODO in E7).
- beans + OpenSpec initialized in-repo; M1 + E1-E7 seeded; poc-foundation change written and validates strictly.
- nix flake check passes (unit + e2e + package + devShell).

Deferred (out of E1 scope by design): the fake-agent build and rmux-sdk/agent CLI derivation land in E6/E7 where first used; the e2e content-flow scenario lands in E7.

OpenSpec change poc-foundation archived in the same commit.
