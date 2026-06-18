---
# qiosq-r7gf
title: initconf, detect or ask where the quiqr storage is
status: completed
type: task
priority: normal
created_at: 2026-06-18T07:16:35Z
updated_at: 2026-06-18T08:05:55Z
blocked_by:
    - qiosq-rblj
---

## Problem
Today `qtui` requires `--config <path>` and the config must already contain an absolute `storage.quiqr_data_dir`. First run is therefore 'hand-edit a TOML before anything works'. We want a friendly first-run: detect where the Quiqr storage is, or ask, then write the config.

## Grounding (investigated 2026-06-18, on this machine)
- Loader (qtui-config): `storage.quiqr_data_dir` must be non-empty; resolved to absolute (ValidationError::EmptyDataDir otherwise). No init flow exists.
- **TWO Quiqr editions to support, both record the storage path authoritatively (same JSON shape):**
  - **Electron desktop app** (installed here): settings dir is the platform config dir — Linux `~/.config/quiqr/`, macOS `~/Library/Application Support/quiqr/`, Windows `%APPDATA%/quiqr/`. `instance_settings.json` has `storage: { type: "fs", dataFolder: "~/Quiqr" }`. `quiqr-app-config.json` adds workspaceKey="main", sitePath, libraryView, plus *.json.v1-backup + user_prefs_*.json.
  - **Quiqr Server** (NixOS module): settings/dataFolder come from the module config (`services.quiqr-server.settings.storage.dataFolder`, default `~/Quiqr`; configDir default `/var/lib/quiqr-server`). Same `settings.storage` schema as the Electron instance_settings.
- CONFIRMED data-folder layout (`~/Quiqr`): `sites/`, `logs/`, `temp/`, `tools/`. Each site = `sites/<name>/` containing `config.json` + a `main/` working copy (the real Hugo/Quiqr root with `quiqr/model/`, `content/`) + `main-*.jsonl` workspace journals. **So the true site root is `sites/<name>/main/`** — NOT the immediate data-dir children.
- => qtui-storage::enumerate_sites (E2) scans immediate subdirs for `config.* + quiqr/`; that does NOT match this real layout (sites are at `sites/<name>/main/`). This is a real correctness gap — tracked as its own follow-up bean, separate from init.

## Proposed approach: thorough discovery, then let the user choose
Add a `qtui init` flow (and auto-trigger when there is no config / empty data dir). Instead of picking one path, DISCOVER ALL candidate Quiqr installs across both editions and present them as a list.
1. **Discover settings sources (both editions):**
   - Electron config dirs per-OS: `~/.config/quiqr` (Linux), `~/Library/Application Support/quiqr` (macOS), `%APPDATA%/quiqr` (Windows). Read `instance_settings.json` -> `storage.dataFolder`.
   - Quiqr Server: a configured/known configDir if discoverable; otherwise covered by the candidate paths below.
2. **Discover candidate data folders:** the dataFolder(s) from step 1 (expand `~`), PLUS an ordered fallback list (`~/Quiqr`, `~/Quiqr Data`). De-dup by resolved absolute path.
3. **Validate + annotate each candidate:** does it exist? does it look like a Quiqr library (has `sites/` with `sites/*/main/quiqr/`)? how many sites? which edition/source found it? Build a list of {path, source(Electron/Server/fallback), siteCount, valid}.
4. **Present options to the user:** show the discovered candidates (path + source + site count) and let them pick; offer 'enter a custom path' too. If exactly one valid candidate and non-interactive, auto-select it.
5. **Headless/`--script`:** never prompt; auto-select a single unambiguous candidate, else error clearly listing what was found.
6. **Write config:** create the qtui config (XDG path, see open questions) from the example + chosen data dir; report where; never overwrite without consent.

## Open questions (for refinement)
- Where should the written config live? `$XDG_CONFIG_HOME/qiosq/config.toml` (likely, for a real install) vs repo `config/quiqr-tui.toml`.
- `qtui init` as a subcommand AND auto-offer on first launch with no config? (Lean: both.)
- Server-edition settings discovery: is there a stable on-disk location to read its configDir/dataFolder, or do we rely on the candidate list + (in the VM) the known dataFolder? 
- Cross-OS config dirs: use a crate (e.g. `dirs`/`directories`) for the per-OS Electron config path rather than hand-rolling.
- Tolerant JSON parsing of instance_settings.json across Quiqr versions (key may move / be absent).

## Acceptance criteria (draft)
- [ ] Discovers candidate data folders from BOTH editions' settings (Electron instance_settings.json per-OS config dir; Server config) plus fallback paths, de-duped.
- [ ] Annotates each candidate with source + whether it's a valid Quiqr library + site count.
- [ ] Presents the list and lets the user choose (or enter a custom path); auto-selects a single unambiguous candidate.
- [ ] Writes a config the loader accepts (round-trips Config::load_and_validate); never overwrites without consent.
- [ ] Headless mode never blocks on a prompt; errors clearly listing what was found.
- [ ] Unit tests for discovery over tempdir fixtures (fake config dir + data folders with/without sites); no network. Depends on the storage-layout fix for accurate site counts.

## Notes
Post-M1 feature (not part of milestone M1, which is complete). Needs its own OpenSpec change + epic when scheduled. Left as a draft pending refinement of the open questions above.

## Summary of Changes
Friendly first-run: detect the Quiqr storage (across both editions) or ask, then write the config.
- qtui-config::discover (pure, injectable home/config roots): reads the Electron desktop app's instance_settings.json (storage.dataFolder) from the per-OS app config dir (dirs crate), expands ~, plus fallbacks ~/Quiqr + ~/Quiqr Data; de-dups by resolved path; annotates each candidate with source (ElectronSettings|Fallback) + valid + site_count (lightweight sites/*/config.json count — no qtui-storage dep, keeps config foundational). Tolerant of missing/malformed JSON.
- qtui-config: default_config_path() = <config_dir>/qiosq/config.toml; write_config(path, data_dir, force) renders a minimal valid TOML (round-trips Config::load_and_validate), creates parent dirs, refuses to clobber without force.
- qtui binary: `qtui init` discovers, lists candidates (path/source/site count), prompts to choose or enter a custom path; headless auto-selects the single valid candidate, else errors listing candidates. Bare `qtui` (no --config) loads ~/.config/qiosq/config.toml if present, else errors with guidance to run `qtui init`; --config still overrides.
- Decisions taken: config at XDG ~/.config/qiosq/config.toml; full scope (discover+choose+write+auto-load); bare-launch OFFERS init via guidance rather than auto-launching (safer).
- Added deps: dirs 6 + serde_json (qtui-config). Verified end to end on the real machine: `qtui init` found ~/Quiqr (1 site), wrote the config, and a subsequent bare `qtui --script` auto-loaded it. cargo test --workspace, clippy -D warnings, fmt, and nix flake check (incl. VM e2e) all green.

Depended on qiosq-rblj (real-layout enumeration), now landed. Follow-up idea (not built): a WP5.1 in-TUI init screen instead of the plain prompt; Server-edition settings-file discovery.

OpenSpec change init-config archived (capabilities storage-discovery + init-config added).
