## Why

The product *is* the two-pane WordPerfect-5.1 experience: a full-screen work
area, a persistent function-key legend, and context-sensitive modes with no
shell visible. E2 (storage) and E3 (schema) produce the data; E4 builds the
`qtui-ui` shell that renders it and drives the mode state machine the user
navigates. Everything downstream (preview surfacing in E5, the file viewer and
agent bridge in E6/E7) hangs off these modes and this legend.

## What Changes

- Implement `qtui-ui` (currently an empty stub) on `ratatui`:
  - A pure `AppState` holding the current `Mode`, the rendered data (site list,
    a site's `content/` tree from `qtui-storage`, its `NavigationModel` from
    `qtui-model`), selection/cursor state, and the preview URL (set by E5).
  - A `Mode` state machine: `SiteList → Browse → ViewFile → Agent`, where
    `Browse` toggles between two navigation views (raw `content/` tree and the
    schema Menu).
  - A pure key-event reducer producing typed transitions/actions, so the state
    machine is testable without a terminal.
  - A two-pane render (left browser/launcher, right agent-pane placeholder) with
    a persistent bottom **function-key legend** that reflects the current mode.
  - Write-like verbs (New, Save, Discard, Ask AI) are emitted as **agent
    requests** (actions), never file operations — the UI never mutates the site.
- `TestBackend` snapshot tests of every mode + legend, and state-machine
  transition tests.

## Capabilities

### New Capabilities
- `two-pane-layout`: the WP5.1 chrome — a two-pane work area plus a persistent
  bottom function-key legend, rendered with ratatui and testable via
  `TestBackend`.
- `mode-state-machine`: the `SiteList → Browse → ViewFile → Agent` modes and the
  pure key-event reducer that transitions between them and emits agent-request
  actions.
- `function-key-legend`: the context-sensitive F-key legend whose entries depend
  on the current mode.
- `dual-navigation`: in `Browse`, a toggle between the raw `content/` tree and
  the schema-driven Menu (Singles/Collections).

### Modified Capabilities
<!-- None — qtui-ui was an empty stub from E1; no existing UI spec. -->

## Impact

- `crates/qtui-ui`: real implementation; adds `ratatui` (which re-exports
  `crossterm`) and depends on the pure crates `qtui-storage` + `qtui-model`.
  Adds `ratatui` to the workspace. No terminal I/O in the library — rendering
  takes a `Frame`, input is fed as key events, so it is fully testable.
- The `qtui` binary does not yet run the event loop against a real terminal
  (that wiring lands with preview/agent in E5–E7); E4 is the testable shell.
- Read-only: the UI never writes site files; all mutation stays with the agent.
