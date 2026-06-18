## Context

`qtui-config` validates a config whose `storage.quiqr_data_dir` is already set;
nothing helps a first-time user find it. Both Quiqr editions record the data
folder authoritatively: the Electron app in `instance_settings.json`
(`storage.dataFolder`, e.g. `~/Quiqr`) under the per-OS app config dir; the Server
in its NixOS module config. The real data folder holds `sites/<name>/` entries
(the layout fixed in `qiosq-rblj`). We add discovery + a `qtui init` flow + a
default config location so a bare `qtui` works without hand-editing TOML.

## Goals / Non-Goals

**Goals:**
- Discover + annotate candidate data folders (Electron settings + fallbacks),
  de-duped, with source / validity / site count.
- `qtui init`: choose (detected or asked) → write `~/.config/qiosq/config.toml`,
  non-clobbering; headless = auto-select single valid or clear error.
- Bare `qtui` loads the default config or offers init; `--config` overrides.

**Non-Goals:**
- Reading the Server edition's settings file directly (no stable user-readable
  path assumed); the Server's data folder is covered by the fallback list and, in
  the VM, by the known `dataFolder`. Can be added later.
- A TUI picker screen — the choice is a simple prompt for now (the WP5.1 init
  screen can come later without a spec change).
- Migrating/relocating Quiqr data; we only point at it.

## Decisions

- **Discovery lives in `qtui-config`** (`discover` module) — it is config
  knowledge, pure, and unit-testable. New deps: `dirs` (per-OS config dir) and
  `serde_json` (parse `instance_settings.json`).
- **No crate dependency on `qtui-storage`** (avoid inverting the layering; today
  storage does not depend on config, so a dep would be acyclic but backwards).
  For the "valid library + site count" annotation, `qtui-config` does a
  **lightweight local check**: count `<candidate>/sites/<name>/` entries that
  contain a `config.json`. This mirrors the storage rule cheaply without coupling.
  The binary, which depends on both, still uses the real `enumerate_sites` at
  runtime. *Alternative considered:* depend on `qtui-storage` for the exact count
  — rejected to keep `qtui-config` foundational and dependency-light.
- **Candidate type:** `DataDirCandidate { path: PathBuf, source: Source, valid:
  bool, site_count: usize }` where `Source` is `ElectronSettings | Fallback`.
  `discover() -> Vec<DataDirCandidate>` de-dups by canonicalized/absolute path
  (canonicalize when it exists, else compare the resolved absolute path).
- **`~` expansion** via `dirs::home_dir()`; tolerant JSON (serde `Value` or a
  small struct with `Option`s) so a missing/legacy `storage.dataFolder` just
  yields no Electron candidate.
- **Default config path:** `dirs::config_dir()/qiosq/config.toml`
  (`~/.config/qiosq/config.toml` on Linux). `qtui-config` exposes
  `default_config_path()` and `write_config(path, data_dir)` (renders a minimal
  valid TOML from the chosen data dir + sane defaults; reuses the example's
  shape). Writing creates parent dirs; refuses to overwrite unless `force`.
- **Binary flow:** `qtui init` → `discover()` → print candidates → prompt (choose
  index / custom path / quit) → `write_config`. Headless (`--script`/no TTY):
  auto-select the single `valid` candidate, else error listing candidates. Bare
  `qtui`: if `--config` given use it; else if default exists load it; else run
  `init` (interactive) or error (headless).

## Risks / Trade-offs

- **Duplicated site-detection logic** (light check in config vs `enumerate_sites`
  in storage). → Kept intentionally minimal (just the `sites/*/config.json`
  count) and documented; the authoritative enumeration stays in storage. Tests
  pin both to the same fixture shape.
- **Interactive prompt is hard to unit-test.** → Keep `discover()` +
  `write_config()` pure and fully tested; the prompt is a thin binary-side
  wrapper exercised by a headless-path test (auto-select) rather than simulated
  keystrokes.
- **`dirs` cross-platform behaviour.** → Acceptable; it is the standard crate and
  vendors under Nix (verified `dirs 6.0`).

## Open Questions

- Whether bare `qtui` with no config should auto-*run* init or just *offer* it.
  Decision: if interactive, run it; if headless, error with guidance. Revisit if
  it feels too eager.
