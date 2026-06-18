## 1. Crate setup

- [x] 1.1 Add `dirs` + `serde_json` deps to `qtui-config` (per-OS config dir; parse instance_settings.json). Keep it a pure crate (no qtui-storage dep).

## 2. Discovery (qtui-config::discover)

- [x] 2.1 `Source` enum (ElectronSettings | Fallback) + `DataDirCandidate { path, source, valid, site_count }`.
- [x] 2.2 Electron settings read: per-OS app config dir via `dirs::config_dir()` → `quiqr/instance_settings.json` → `storage.dataFolder`; expand leading `~`. Tolerant (missing/malformed/absent key → no candidate).
- [x] 2.3 Fallback candidates `~/Quiqr`, `~/Quiqr Data`. De-dup all by resolved absolute path.
- [x] 2.4 Annotate each: lightweight `valid` + `site_count` = count of `<path>/sites/<name>/` dirs containing `config.json` (mirrors the storage rule cheaply; no qtui-storage dep).
- [x] 2.5 `discover() -> Vec<DataDirCandidate>`.

## 3. Config path + writer (qtui-config)

- [x] 3.1 `default_config_path()` = `dirs::config_dir()/qiosq/config.toml`.
- [x] 3.2 `write_config(path, data_dir, force)`: render a minimal valid TOML (chosen data dir + defaults), create parent dirs, refuse to overwrite unless force; result round-trips through `Config::load_and_validate`.

## 4. Binary: init flow + default-config load (qtui)

- [x] 4.1 `qtui init`: discover, print candidates (path/source/site count), prompt (choose index / enter custom path / quit), then write_config to the default path (consent before overwrite). Print where it wrote.
- [x] 4.2 Headless init (no TTY / `--script`): auto-select the single valid candidate, else error listing candidates.
- [x] 4.3 Bare `qtui` (no --config): use `--config` if given; else load default path if present; else error with guidance pointing at `qtui init`. `--config` precedence preserved. (Chose explicit guidance over auto-launching init — safer/less surprising; revisit if desired.)

## 5. Tests

- [x] 5.1 Discovery unit tests over tempdir fixtures: fake config dir with instance_settings.json (valid / malformed / missing key) + data folders with/without `sites/*/config.json`; assert candidates, sources, ~ expansion, de-dup, validity + site_count.
- [x] 5.2 `write_config` test: writes a config that `Config::load_and_validate` accepts; refuses to clobber without force.
- [x] 5.3 `default_config_path` ends with `qiosq/config.toml`.

## 6. Gate

- [x] 6.1 `cargo test --workspace` green; `cargo clippy --workspace --all-targets -- -D warnings` clean.
- [x] 6.2 `cargo fmt --all --check` clean.
- [x] 6.3 `nix flake check` green (incl. the VM e2e).

## 7. Done

- [x] 7.1 OpenSpec change `init-config` validates.
- [x] 7.2 `qiosq-r7gf` closed; commit references it; change archived.
