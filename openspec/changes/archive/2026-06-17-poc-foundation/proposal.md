## Why

The quiqr-tui PoC (codename *Reveal*) needs a reproducible, testable foundation
before any feature work. Without a Nix flake dev shell, a compiling Cargo
workspace, a validated config loader, and initialized beans + OpenSpec, every
later epic (E2–E7) would rebuild the same scaffolding ad hoc and tests would not
be reproducible across machines. This change establishes epic **E1** so that
E2–E7 can each be a clean, spec-led, test-gated increment.

## What Changes

- Create a Cargo workspace with seven crates (`qtui-config`, `qtui-storage`,
  `qtui-model`, `qtui-agent`, `qtui-preview`, `qtui-ui`, `qtui`) — empty but
  compiling, with shared workspace lints.
- Add `flake.nix` providing a dev shell (rust toolchain, hugo, beans, openspec,
  the agent CLI, node), `nix build` of the `qtui` binary, and `nix flake check`.
- Implement `qtui-config`: load and validate a TOML config, surfacing a
  distinct, human-readable error per invalid field; ship
  `config/quiqr-tui.example.toml`.
- Initialize OpenSpec and beans in the repository; agent workflow instructions
  present and correct.
- Wire `nix flake check` to run the workspace unit tests; scaffold (not yet
  assert) the `checks.<system>.e2e` NixOS VM test.

## Capabilities

### New Capabilities
- `config-loading`: load, resolve, and validate the quiqr-tui TOML configuration
  (Quiqr data dir, agent command + args, hugo binary, preview port range,
  completion sentinel, sandbox paths) with clear per-field errors.
- `nix-foundation`: a Nix flake providing a reproducible dev shell, `nix build`,
  and a `nix flake check` that runs unit tests and scaffolds the e2e VM test.
- `workspace-scaffold`: a seven-crate Cargo workspace that builds cleanly and
  establishes the crate boundaries the architecture defines.

### Modified Capabilities
<!-- None — this is the first change; no existing specs. -->

## Impact

- New `Cargo.toml` workspace + `crates/*` (seven crates) and `Cargo.lock`.
- New/updated `flake.nix`; `nix flake check` becomes the CI gate.
- `config/quiqr-tui.example.toml` shipped and documented.
- Tooling state committed in-repo: `.beans/` + beans markdown, `openspec/`.
- No storage, schema, UI, preview, or agent functionality (those are E2–E7); no
  SSH/kiosk hardening, wrapper app, or publish pipeline. The VM e2e harness is
  scaffolded to exist but only asserts the workspace boots; the full scenario
  lands in E7.
