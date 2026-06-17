## 1. Workspace scaffold

- [x] 1.1 Create the root `Cargo.toml` workspace with seven members under `crates/`.
- [x] 1.2 Create `qtui-config`, `qtui-storage`, `qtui-model`, `qtui-agent`, `qtui-preview`, `qtui-ui` as library crates (empty but compiling).
- [x] 1.3 Create `qtui` as the binary crate that depends on the others and prints its version.
- [x] 1.4 Add shared workspace lints / fmt / clippy config; ensure pure crates exclude ratatui/rmux.

## 2. qtui-config

- [x] 2.1 Define typed config structs for `[storage]`, `[preview]`, `[agent]`, `[agent.sandbox]`, `[ui]`, `[rmux]` (serde + toml).
- [x] 2.2 Implement `load(path)` returning a clear error for missing file / malformed TOML.
- [x] 2.3 Implement validation: resolve data dir to absolute + non-empty; port range valid, ordered, excludes 13131; non-empty agent command and completion sentinel — one distinct error per field.
- [x] 2.4 Fix and ship `config/quiqr-tui.example.toml` (remove the stray heredoc `EOF` line); ensure it loads + validates.

## 3. Nix

- [x] 3.1 Dev shell exposes rust toolchain, hugo, git, node, beans, openspec.
- [x] 3.2 `nix build` produces the `qtui` binary; `Cargo.lock` committed.
- [x] 3.3 `checks.unit` runs `cargo test --workspace`.
- [x] 3.4 `checks.e2e` NixOS VM test scaffolded to boot to `multi-user.target`; `TODO(author)` for the Quiqr module.

## 4. Tooling init

- [x] 4.1 Initialize beans in-repo; seed milestone M1 and epics E1–E7 from `docs/epics-seed.md`.
- [x] 4.2 Initialize OpenSpec in-repo; agent workflow instructions present; this `poc-foundation` change validates.

## 5. Tests (gate)

- [x] 5.1 Unit: example config loads; each invalid field produces a distinct, human-readable error.
- [x] 5.2 `cargo test --workspace` green on the host.
- [x] 5.3 `nix flake check` wiring verified (unit check; e2e scaffold boots).

## 6. Done

- [x] 6.1 OpenSpec change `poc-foundation` validates.
- [x] 6.2 All E1 beans closed; commit references bean IDs; OpenSpec change archived.
