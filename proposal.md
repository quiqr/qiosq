# Proposal: poc-foundation

> Worked example of an OpenSpec change. Use `openspec --help` to confirm this
> install's exact file layout and commands; adapt this content into the real
> structure during bootstrap. Follow this shape for E2–E7.

## Why

The quiqr-tui PoC needs a reproducible, testable foundation before any feature
work. Without a Nix flake dev shell, a compiling workspace, a validated config
loader, and initialized beans + OpenSpec, every later epic would rebuild the
same scaffolding ad hoc and tests would not be reproducible across machines.

This change establishes that foundation (epic **E1**) so that E2–E7 can each be
a clean, spec-led, test-gated increment.

## What changes

- Create a Cargo workspace with seven crates (`qtui-config`, `qtui-storage`,
  `qtui-model`, `qtui-agent`, `qtui-preview`, `qtui-ui`, `qtui`), empty but
  compiling.
- Add `flake.nix` providing a dev shell (rust toolchain, hugo, beans, openspec,
  the agent CLI), `nix build`, and `nix flake check`.
- Implement `qtui-config`: load and validate a TOML config; surface clear errors;
  ship `config/quiqr-tui.example.toml`.
- Initialize OpenSpec and beans in the repository; ensure `AGENTS.md` is present
  and correct for the agent workflow.
- Wire `nix flake check` to run unit tests.

## Non-goals

- No storage, schema, UI, preview, or agent functionality (those are E2–E7).
- No SSH/kiosk hardening, no wrapper app, no publish pipeline.
- The VM e2e harness is *scaffolded to exist* but only asserts the workspace
  boots; the full e2e scenario lands in E7.

## Acceptance criteria

- `nix develop` yields a shell with rust, hugo, beans, openspec, and the agent
  CLI available.
- `cargo build --workspace` succeeds; all seven crates compile.
- `qtui-config` loads the example config; each invalid field produces a distinct,
  human-readable error (unit-tested).
- `nix flake check` passes and runs the config unit tests.
- beans is initialized and the M1 epics (E1–E7) exist as beans; OpenSpec is
  initialized with this change present and validating.

## Risks

- **rmux / agent CLI availability in Nix.** Mitigation: pin versions; if a
  package is missing, add a flake input or build-from-source derivation rather
  than weakening the flake.
- **Two tools named "beans".** Mitigation: the agent discovers the real CLI via
  `beans prime` / `--help` before scripting it.
