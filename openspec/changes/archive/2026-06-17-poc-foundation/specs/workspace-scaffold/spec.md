## ADDED Requirements

### Requirement: Seven-crate workspace

The repository SHALL define a Cargo workspace containing exactly seven member
crates — `qtui-config`, `qtui-storage`, `qtui-model`, `qtui-agent`,
`qtui-preview`, `qtui-ui`, and `qtui` — each of which compiles, so that every
later epic has a clear, independently testable home.

#### Scenario: Workspace builds clean
- **WHEN** `cargo build --workspace` is run
- **THEN** all seven crates compile without error

### Requirement: Pure crates stay dependency-light

The pure crates `qtui-config`, `qtui-storage`, and `qtui-model` MUST NOT depend
on `ratatui` or `rmux`, preserving the architecture's separation between pure
logic and the UI/agent layers.

#### Scenario: Pure crate manifests exclude UI deps
- **WHEN** the manifests of `qtui-config`, `qtui-storage`, and `qtui-model` are
  inspected
- **THEN** none of them lists `ratatui` or an `rmux` crate as a dependency

### Requirement: Binary entrypoint

The `qtui` crate SHALL be the workspace binary that, in E1, prints its version,
establishing the entrypoint that later epics wire the services into.

#### Scenario: Binary prints version
- **WHEN** the `qtui` binary is run with a version flag
- **THEN** it prints its package version and exits successfully
