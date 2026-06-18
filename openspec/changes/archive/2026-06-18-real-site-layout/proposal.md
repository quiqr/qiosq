## Why

E2's `site-enumeration` was written and tested against a hand-built shape
(`<data>/<site>/{config.*, quiqr/, content/}`) that does **not** match a real
Quiqr data folder. Confirmed against the actual library on disk (Electron desktop
edition; the Server edition uses the same layout):

```
~/Quiqr/                      # the data folder
  sites/                      # all sites live under here
    <name>/
      config.json             # Quiqr site descriptor; source.path = working copy
      main/                   # the working copy: quiqr/model/, content/, Hugo config
      main-*.jsonl            # workspace journals
  logs/  temp/  tools/
```

`config.json` records the working-copy path in `source.path` (e.g. `"main"`, or
`"./"` for an older flat site). Against this, the current enumeration finds **zero
sites**: it scans the data dir's immediate children (not `sites/*`) and looks for
`config.* + quiqr/` in the wrong directory. This change fixes enumeration to the
real layout so the PoC works against an actual Quiqr install.

## What Changes

- Enumerate sites under `<data>/sites/*` (not the data dir's immediate children).
- Recognise a site by its **`config.json`** descriptor and resolve its working
  copy from `source.path` (relative to the site dir): `"main"` →
  `sites/<name>/main/`, `"./"` → `sites/<name>/` itself.
- Expose both the site dir and the resolved **working-copy root** on `Site`; the
  content tree and schema model operate on the working-copy root (where `quiqr/`
  and `content/` actually live).
- Ignore `sites/*` entries without a readable `config.json` (e.g. `test/`,
  `test-workspace-isolation-site/`) and non-active workspaces.
- Reshape the test fixtures (including the anonymized `real-site`) to the real
  `sites/<name>/{config.json, main/…}` layout.

## Capabilities

### Modified Capabilities
- `site-enumeration`: change the discovery rule from "immediate subdir with
  `config.* + quiqr/`" to "the data folder's `sites/<name>/` entries, each
  identified by `config.json` with its working copy resolved from `source.path`."

## Impact

- `crates/qtui-storage`: `enumerate_sites` rewritten for the real layout; `Site`
  gains a working-copy path; adds a `serde_json` dependency (parse `config.json`).
  `content_tree` operates on the working-copy root.
- Test fixtures reshaped; enumeration + content-tree tests updated.
- `crates/qtui` host + the e2e VM provisioning updated to the real layout so the
  full flow still passes. Still read-only; no site files are written.
