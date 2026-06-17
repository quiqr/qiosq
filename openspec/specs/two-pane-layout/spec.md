# two-pane-layout Specification

## Purpose
TBD - created by archiving change tui-shell. Update Purpose after archive.
## Requirements
### Requirement: Two-pane work area with a legend row

The UI SHALL render a full-screen layout consisting of a left pane
(browser/launcher), a right pane (the agent view), and a single persistent
function-key legend row pinned to the bottom. The layout SHALL be produced by
rendering into a ratatui `Frame`, with no direct terminal I/O in the library, so
it can be exercised with a `TestBackend`.

#### Scenario: Layout has left pane, right pane, and a bottom legend
- **WHEN** the app is rendered to a `TestBackend` of a given size
- **THEN** the buffer contains a left pane and a right pane side by side and a
  legend row on the bottom line

#### Scenario: Rendering performs no terminal I/O
- **WHEN** the render function is called with a `Frame`
- **THEN** it draws widgets into that frame only and does not read from or write
  to a real terminal

### Requirement: The right pane is the agent view

The UI SHALL reserve the right pane for the coding-agent session view. Until the
agent bridge lands (E6/E7) the right pane SHALL show a labelled placeholder, so
the two-pane shape is established and the user understands where the agent
appears.

#### Scenario: Right pane shows the agent placeholder
- **WHEN** the app is rendered before the agent bridge exists
- **THEN** the right pane is labelled as the agent pane (a placeholder), not left
  blank or used for left-pane content

### Requirement: The UI never mutates site files

The UI SHALL NOT perform any file writes. Verbs that imply a change (New, Save,
Discard, Ask AI) SHALL be surfaced only as requests/actions for the agent, never
executed as file operations by the UI.

#### Scenario: A write-like verb produces an action, not a file change
- **WHEN** the user triggers a write-like verb (e.g. Ask AI)
- **THEN** the UI emits a typed action describing the request and does not create,
  modify, or delete any file

