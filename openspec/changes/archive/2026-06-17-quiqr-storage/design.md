## Context

E1 produced a validated `Config` whose `storage.quiqr_data_dir` is an absolute
path and whose `storage.hidden_dirs` lists directories to hide. E2 implements the
`qtui-storage` crate (an empty stub today) against the architecture's contract
(`docs/01-architecture.md` §3): enumerate sites, expose a filtered `content/`
tree, never write. This is a pure crate — no ratatui/rmux.

## Goals / Non-Goals

**Goals:**
- Read-only site discovery and a filtered content tree, as plain data types.
- Thorough unit tests over tempdir fixtures shaped like real Quiqr sites.

**Non-Goals:**
- Parsing the Quiqr schema (`quiqr/model/`) — that is E3 (`qtui-model`).
- Any UI rendering of the tree — that is E4.
- Any file writing or watching/live-reload — out of scope for the PoC layer.

## Decisions

- **Site detection rule.** A subdirectory is a site iff it has a Hugo config
  (`config.*`/`hugo.*` file, or a `config/` dir) **and** a `quiqr/` dir. The dual
  marker avoids matching bare Hugo sites or stray folders. *Alternative
  considered:* require only `quiqr/` — rejected because a Hugo config is the
  thing the preview (`hugo server`) needs, so its presence is part of "is this a
  servable site".
- **Tree shape.** A simple recursive `ContentNode { Dir { name, rel_path,
  children }, File { name, rel_path } }`. Relative paths (to `content/`) keep the
  type decoupled from where the site lives on disk and are what the UI and the
  "send to agent" `@path` injection will use. *Alternative considered:* a flat
  list of paths — rejected; the WP-style nav renders a tree.
- **Hiding by directory name, at every depth.** Filtering matches the configured
  `hidden_dirs` by base name anywhere in the walk (not just top level), since
  `resources/`/`.git/` can appear nested. Files are never hidden by this rule
  (only directories), matching the architecture wording.
- **Ordering.** Directories first, then files, each alphabetically — a stable,
  predictable order for snapshot tests and for the UI.
- **Errors.** Missing/unreadable data dir is an error that names the path
  (consistent with `qtui-config`'s error style). A missing `content/` dir is
  **not** an error — it yields an empty tree, because a brand-new site may have
  no content yet.
- **Test fixtures.** Build sites in a `tempfile::TempDir` rather than committing
  binary fixtures, so tests are hermetic and the "never writes to the real data
  dir" guarantee is easy to assert.

## Risks / Trade-offs

- **Symlink loops / permission errors mid-walk.** → Mitigation: do not follow
  symlinks into directories during the walk; surface a read error for the
  offending path rather than panicking (tolerant, like schema parsing will be).
- **Large content trees.** → Acceptable for the PoC; the tree is built eagerly.
  If it ever matters, lazy/streaming loading is a later optimization, not a spec
  change.

## Open Questions

- None blocking. Whether to also expose file size/mtime on `File` nodes is
  deferred until the UI (E4) needs it; the spec intentionally leaves it out.
