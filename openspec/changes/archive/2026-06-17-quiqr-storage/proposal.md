## Why

The TUI needs to know which Quiqr sites exist and what files an author may browse
before any UI, schema parsing, or preview can happen (E3–E5 all build on it).
E1 gave us a validated config pointing at the Quiqr data directory; E2 turns that
path into read-only knowledge of the storage: the list of sites, and a filtered
`content/` tree per site. Per the architecture's single-writer rule, this layer
**never writes** — site mutation is the agent's job alone.

## What Changes

- Implement `qtui-storage` (currently an empty stub):
  - Enumerate sites under the configured data dir: a subdirectory is a site when
    it has a Hugo config (`config.*` or `hugo.*`, file or `config/` dir) **and** a
    `quiqr/` folder.
  - Build a read-only file tree of a site's `content/` directory, hiding
    derived/generated/VCS dirs from the configured `hidden_dirs`
    (`public`, `resources`, `.quiqr-cache`, `.git`, `themes`, …).
  - Expose plain data types (`Site`, `ContentNode`) with no I/O side effects
    beyond reading the filesystem; the crate performs **no writes**.
- Add Quiqr-site-shaped tempdir fixtures and unit tests for enumeration and tree
  filtering, including non-site dirs and hidden-dir cases.

## Capabilities

### New Capabilities
- `site-enumeration`: discover the Quiqr sites under the data directory and
  identify each by its on-disk shape (Hugo config + `quiqr/` folder).
- `content-tree`: produce a read-only, depth-ordered tree of a site's `content/`
  directory with derived/generated/VCS directories hidden.

### Modified Capabilities
<!-- None — qtui-storage was an empty stub from E1; no existing storage spec. -->

## Impact

- `crates/qtui-storage`: real implementation + `dev-dependencies` for tempdir
  fixtures (e.g. `tempfile`); stays a pure crate (no ratatui/rmux).
- Adds the `tempfile` dev-dependency to the workspace.
- No change to the UI, agent, preview, or config crates. No writes anywhere.
- The `qtui` binary does not yet wire this in (that happens with the UI in E4);
  E2 is library + tests only.
