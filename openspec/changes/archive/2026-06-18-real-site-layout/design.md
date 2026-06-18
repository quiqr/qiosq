## Context

`qtui-storage::enumerate_sites` (E2) assumed `<data>/<site>/{config.*, quiqr/}`.
The real Quiqr data folder — verified across ~13 sites in the on-disk library —
is `<data>/sites/<name>/`, where `config.json` (`{key, name, source:{type,path},
…}`) points at the working copy via `source.path`. Standard sites use
`"main"`; an older site uses `"./"`. Non-site dirs (`test/`,
`test-workspace-isolation-site/`) have no `config.json`. Non-active workspaces
(e.g. `kitchen-sink/testing/`) exist but are not the site root.

## Goals / Non-Goals

**Goals:**
- Enumerate against the real layout (`<data>/sites/*` + `config.json`).
- Resolve and expose each site's working-copy root; content tree + model read
  from it.
- Reshape fixtures to the real layout; keep the whole workspace + VM e2e green.

**Non-Goals:**
- Workspace switching / non-`main` workspaces (read the active one from
  `source.path`; multi-workspace UI is out of scope here).
- Reading anything else from `config.json` (publish targets, etc.).
- The `initconf` discovery feature (`qiosq-r7gf`) — this fix unblocks its
  site-count step but is separate.

## Decisions

- **`config.json` is the site marker; `source.path` is authoritative.** Parse it
  with `serde_json` into a tolerant struct (`source.path: Option<String>`,
  unknown keys ignored). Resolve the working copy = `site_dir.join(path)` with
  `"."`/`"./"`/empty/absent → `site_dir`. This handles both `main` and flat
  sites uniformly and excludes non-sites (no `config.json`) for free.
  *Alternative considered:* keep sniffing for `quiqr/` dirs — rejected; the
  descriptor is what Quiqr itself uses and it disambiguates the working copy.
- **`Site` gains a `work_dir`** (the resolved working copy); `path` stays the
  site dir (for display/identity by `name`). `content_tree` takes the work dir.
  To minimise churn and keep `Site` honest, set `Site.path` to the **working
  copy** (what every consumer actually needs) and add the site-dir as a separate
  field if needed. *Decision:* make `Site { name, path }` where `path` = working
  copy (the root with `quiqr/`+`content/`), since all current consumers
  (`content_tree`, `load_model`, the host, preview cwd) want exactly that. The
  site directory is derivable (`path.parent()` for `main`) and not needed by
  consumers, so we don't add a field — keeps the type and all call sites stable.
- **Tolerance.** A `sites/<name>/` with `config.json` but a `source.path` that
  doesn't exist is still listed (the working copy may be created later), matching
  the existing "missing content/ → empty tree" leniency. A malformed
  `config.json` excludes that entry (can't resolve the working copy).
- **`is_quiqr_site`** is repurposed/kept for the working-copy check used by the
  preview/host, but enumeration no longer depends on the old Hugo-config sniff.

## Risks / Trade-offs

- **`Site.path` semantics change** (site dir → working copy). → All in-repo
  consumers want the working copy, so this is net-correct; tests + host updated
  in the same change. Documented on the type.
- **Fixture reshape touches several tests.** → Mechanical; the e2e VM provisions
  the reshaped fixture too, so the real path is exercised end to end.

## Open Questions

- Whether to read `config.json`'s `key`/`name` for the display name instead of
  the directory name. For now use the directory name (matches today); the
  descriptor name is usually identical. Not spec-critical.
