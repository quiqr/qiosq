# Design Notes: poc-foundation (E1)

## Workspace
Seven crates as in `docs/01-architecture.md`. Keep cross-crate deps minimal:
`qtui` depends on all; the rest avoid depending on each other except where the
contract requires (e.g. `qtui-agent` may use `qtui-model` types for schema
context). Pure crates (`qtui-config`, `qtui-storage`, `qtui-model`) must not pull
in ratatui/rmux.

## Config shape
TOML, loaded once at startup. See `config/quiqr-tui.example.toml`. Decisions:
- Resolve `quiqr_data_dir` to an absolute path; error if missing/unreadable.
- `agent.command` + `agent.args` keep Claude Code swappable behind the trait.
- `preview.port_range` excludes `13131` (Quiqr default) — validate this.
- `agent.completion_sentinel` is the string the agent prints when done.

## Nix decisions
- Use a flake with `flake-parts` or plain `outputs`; pick one and document it.
- Rust via `rust-overlay` or `fenix` (agent's choice; pin it).
- The agent CLI and rmux may not be in nixpkgs — prefer a pinned flake input or
  a small derivation over dropping them from the shell.
- `checks.<system>.e2e` uses the nixpkgs `nixosTest`/`runTest` framework. In E1
  it only boots and asserts the workspace builds; E7 fills the scenario.

## Open question for the author
- Exact attribute path of the Quiqr Server NixOS module + package (you authored
  it). Marked `TODO(author)` in `flake.nix`. The e2e VM imports it to run Quiqr
  Server.
