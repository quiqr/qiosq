## Context

This is the first change in the quiqr-tui PoC. There is no existing code — only
the architecture (`docs/01-architecture.md`), the testing bar
(`docs/02-testing-strategy.md`), and the project brief. E1 must produce a
foundation that the remaining six epics build on without re-deciding tooling.
Constraints from `CLAUDE.md`: Nix from the start, tests gate everything, and the
example config must let the PoC run on another machine by editing only that file.

## Goals / Non-Goals

**Goals:**
- A Cargo workspace of seven crates that builds clean and fixes the crate
  boundaries from the architecture.
- A reproducible Nix flake: dev shell, `nix build`, and `nix flake check` as the
  single CI gate, with the e2e VM test scaffolded.
- `qtui-config` that loads and validates the TOML config with one distinct,
  human-readable error per invalid field.

**Non-Goals:**
- Any storage, schema, UI, preview, or agent behaviour (E2–E7).
- The full e2e content-flow scenario (E7) — E1 only boots the VM.
- SSH/kiosk hardening, the wrapper app, multi-user auth, the publish pipeline.

## Decisions

- **Workspace dependency graph.** `qtui` depends on all other crates; the rest
  avoid depending on each other except where a contract requires it (e.g.
  `qtui-agent` may use `qtui-model` types later). The pure crates
  (`qtui-config`, `qtui-storage`, `qtui-model`) must not pull in ratatui/rmux.
  *Alternative considered:* a single crate — rejected because per-crate testing
  and the pure/impure split are core to the testing strategy.
- **Config shape & validation.** TOML, loaded once at startup, parsed with
  `serde` + the `toml` crate into typed structs, then validated in a separate
  pass that returns a `ConfigError` enum with one variant per field so each
  invalid field yields a distinct message. Decisions: resolve
  `quiqr_data_dir` to an absolute path; `preview.port_range` must exclude
  `13131`; `agent.command` and `agent.completion_sentinel` must be non-empty.
  *Alternative considered:* validate inline during deserialization — rejected
  because a dedicated pass gives clearer, field-named errors and is easier to
  unit-test exhaustively.
- **Nix layout.** `flake-parts` with `rust-overlay` (stable toolchain, pinned by
  the flake lock), matching the scaffold already in `flake.nix`. `nix build`
  uses `rustPlatform.buildRustPackage` against `Cargo.lock`; `checks.unit` runs
  `cargo test --workspace`; `checks.e2e` uses `testers.runNixOSTest`.
  *Alternative considered:* plain `outputs` without flake-parts — rejected to
  keep multi-system handling (`x86_64-linux`, `aarch64-*`) terse.
- **beans + openspec are committed in-repo.** Their state (`.beans/`, beans
  markdown, `openspec/`) is the durable audit trail and is tracked, not ignored.

## Risks / Trade-offs

- **rmux / agent CLI availability in Nix.** → Mitigation: for E1 the dev shell
  ships only tools that exist today (rust, hugo, beans, openspec, node); the
  `rmux-sdk` and a real agent derivation are deferred to the epics that need
  them (E6/E7), tracked as beans rather than weakening the flake now.
- **Two tools named "beans".** → Mitigation: confirmed the correct beans via
  `beans prime` / `--help` before scripting it; this install is the markdown
  issue tracker, not Rust Type Kit.
- **`nix flake check` cost (VM test).** → Mitigation: in E1 the VM only asserts
  boot, keeping the check fast; the heavy scenario lands in E7.

## Migration Plan

Not applicable — greenfield. Rollback is `jj` history; each epic is a discrete
commit referencing its beans, and the OpenSpec change is archived once merged.

## Open Questions

- Exact attribute path of the Quiqr Server NixOS module + package (authored by
  the project owner). Marked `TODO(author)` in `flake.nix`; the e2e VM imports it
  in E7. This does not block E1.
