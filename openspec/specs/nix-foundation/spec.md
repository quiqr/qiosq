# nix-foundation Specification

## Purpose
TBD - created by archiving change poc-foundation. Update Purpose after archive.
## Requirements
### Requirement: Reproducible dev shell

The flake SHALL provide a `devShells.default` that makes the Rust toolchain
(with `clippy` and `rustfmt`), `hugo`, `git`, `node`, `beans`, and `openspec`
available, so that a fresh checkout can build, test, and run the project tooling
with only `nix develop`.

#### Scenario: Dev shell exposes the toolchain
- **WHEN** a developer runs `nix develop`
- **THEN** `cargo`, `hugo`, `beans`, and `openspec` are on `PATH`

### Requirement: Buildable package

The flake SHALL expose `packages.default` that builds the `qtui` binary via
`nix build`.

#### Scenario: nix build produces the binary
- **WHEN** `nix build` is run on a clean checkout
- **THEN** it succeeds and produces a `qtui` binary that prints its version

### Requirement: Checks run unit tests

The flake SHALL expose `checks` that run the workspace unit tests, so that
`nix flake check` is the single CI gate for the project.

#### Scenario: flake check runs the config tests
- **WHEN** `nix flake check` is run on a clean checkout
- **THEN** it executes `cargo test --workspace`, including the `qtui-config`
  validation tests, and passes

### Requirement: Scaffolded end-to-end VM test

The flake SHALL expose a `checks.e2e` NixOS VM test that boots a virtual machine.
In E1 this check only asserts the VM reaches `multi-user.target`; the full Quiqr
Server scenario is wired in epic E7 and is marked with a `TODO(author)` for the
Quiqr NixOS module input.

#### Scenario: e2e VM boots
- **WHEN** the `e2e` check runs
- **THEN** the VM boots to `multi-user.target` and the check passes without yet
  asserting the full content flow

