# Tasks: poc-foundation (E1)

> Check off as you go; keep in sync with the corresponding beans. Tests are
> tasks, not afterthoughts.

## Scaffolding
- [ ] Create Cargo workspace + seven crates (compiling, empty).
- [ ] Add workspace lints / fmt / clippy config.

## Nix
- [ ] `flake.nix`: dev shell (rust toolchain, hugo, beans, openspec, agent CLI).
- [ ] `nix build` produces the `qtui` binary (even if it only prints version).
- [ ] `nix flake check` wired to run `cargo test --workspace`.
- [ ] Scaffold (not yet asserting full flow) the `checks.<system>.e2e` nixosTest;
      `TODO(author)` marker where the Quiqr module input is wired.

## Config
- [ ] `qtui-config`: TOML schema (data dir, agent cmd+args, hugo bin, port range,
      sentinel).
- [ ] Validation with distinct human-readable errors per field.
- [ ] `config/quiqr-tui.example.toml` shipped and documented.

## Tooling init
- [ ] Initialize OpenSpec in-repo; ensure AGENTS.md is correct.
- [ ] Initialize beans; create M1 epics E1–E7 from docs/epics-seed.md.

## Tests (gate)
- [ ] Unit: config loads example; each invalid field errors clearly.
- [ ] `nix flake check` green on a clean checkout.

## Done
- [ ] OpenSpec change validated.
- [ ] All E1 beans closed; commits reference bean IDs.
