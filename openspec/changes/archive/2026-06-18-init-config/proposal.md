## Why

First run today means hand-editing a TOML to set `storage.quiqr_data_dir` before
`qtui` does anything — `--config <path>` is required and the data dir must already
be filled in. That is a poor first experience for the non-technical author the
product targets. Both Quiqr editions record their storage location authoritatively
on disk, so we can detect it (or ask) and write the config automatically.

## What Changes

- Add **storage discovery**: find candidate Quiqr data folders across both
  editions and annotate each one:
  - **Electron desktop**: read `instance_settings.json` → `storage.dataFolder`
    from the per-OS app config dir (`~/.config/quiqr` on Linux,
    `~/Library/Application Support/quiqr` on macOS, `%APPDATA%/quiqr` on Windows).
  - **Fallback paths**: `~/Quiqr`, `~/Quiqr Data`.
  - De-dup by resolved absolute path; annotate each with its source, whether it
    is a valid Quiqr library, and its site count (via the storage layer).
- Add a `qtui init` flow: present the discovered candidates (path + source + site
  count), let the user choose one or enter a custom path, and **write the config
  to `~/.config/qiosq/config.toml`** (XDG), never overwriting an existing one
  without consent. Headless/non-interactive runs auto-select a single valid
  candidate or error clearly, listing what was found.
- **Default config loading**: `qtui` with no `--config` loads
  `~/.config/qiosq/config.toml`; if it is absent, it runs/offers `init`.
  `--config <path>` still overrides.

## Capabilities

### New Capabilities
- `storage-discovery`: discover and annotate candidate Quiqr data folders from
  both editions' settings plus fallback paths.
- `init-config`: the `qtui init` flow that chooses a data folder (detected or
  asked) and writes the qtui config, plus default-config resolution for a bare
  `qtui` launch.

### Modified Capabilities
<!-- config-loading is reused (Config::load_and_validate) but its requirements
     don't change; the default-path resolution is new behaviour in init-config. -->

## Impact

- `crates/qtui-config`: add a `discover` module (+ the default config path and a
  config-writer); new deps `dirs` (per-OS config dir) and `serde_json` (parse
  `instance_settings.json`). Stays a pure crate (no ratatui/rmux). Reuses the
  storage layer's `enumerate_sites` for site counts (dev-time check kept light to
  avoid a hard dependency cycle — see design).
- `crates/qtui` (binary): new `qtui init` subcommand; bare `qtui` resolves the
  default config or offers init. `--config` precedence unchanged.
- Read-only discovery: reads settings/data dirs, writes only the qtui config file
  the user consents to. Never touches Quiqr's own files or site content.
