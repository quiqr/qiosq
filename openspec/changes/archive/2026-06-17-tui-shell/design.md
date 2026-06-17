## Context

`qtui-ui` (an E1 stub) becomes the WP5.1 shell. It consumes the pure data from
`qtui-storage` (`Site`, `ContentNode`) and `qtui-model` (`NavigationModel`) and
renders a two-pane layout with a context-sensitive function-key legend, driven by
a mode state machine. The architecture (§3) and the testing strategy (§2) require
this crate to be exercised with ratatui's `TestBackend`, so the design keeps all
logic pure and pushes the terminal out of the library.

## Goals / Non-Goals

**Goals:**
- A pure `AppState` + `Mode` state machine and a pure key→transition reducer.
- A `render(frame, state)` that draws two panes + the legend.
- Dual navigation in `Browse` (content tree ↔ schema menu) with a configurable
  default.
- `TestBackend` snapshot + transition tests for every mode.

**Non-Goals:**
- Running a real terminal event loop / raw mode (the host binary does that in
  E5–E7; the library only renders frames and consumes key events).
- Starting Hugo / surfacing a live preview URL (E5 sets a field the legend/Browse
  can show; the lifecycle is E5's).
- The agent pane content (E6/E7) — the right pane is a labelled placeholder.
- Any file writing — write verbs are emitted as actions only.

## Decisions

- **Pure core, I/O at the edge.** The library exposes `AppState`,
  `Action`, `render(&mut Frame, &AppState)`, and `update(&mut AppState, KeyEvent)
  -> Option<Action>`. The future host loop owns the `Terminal`, reads crossterm
  events, calls `update`, then `render`. This is exactly what makes `TestBackend`
  tests trivial. *Alternative considered:* an `App::run(terminal)` method —
  rejected; it hides the state machine behind I/O and is hard to snapshot.
- **`Mode` carries its context.** `SiteList`, `Browse { nav: NavView }`,
  `ViewFile`, `Agent`. `NavView` is `ContentTree | SchemaMenu`. Mode-specific
  selection lives in `AppState` (e.g. selected site / tree row / menu entry).
- **Back is a fixed linear pop** over `SiteList → Browse → ViewFile → Agent`,
  matching the spec; no arbitrary history stack needed for the PoC.
- **Legend as data.** `legend(&AppState) -> Vec<LegendEntry { key, label }>`
  returns the entries; `render` lays them out. Keeping it as data lets tests
  assert legend contents directly, independent of pixel layout, and the renderer
  stays dumb. Keys follow the README: F2 Browse, F3 New, F5 Preview, F6 Ask AI,
  F7 Save, F9 Discard (plus F1/F10-style Back/Quit as needed).
- **Write verbs are `Action`s.** `update` returns `Action::AskAi { file }`,
  `Action::RequestSave`, `Action::RequestDiscard`, `Action::RequestNew`,
  `Action::Quit`, etc. The UI never touches the filesystem. This is the
  single-writer rule expressed in the type system.
- **ratatui 0.30.** Verified the API (TestBackend, `Frame::area`, `Layout`,
  `List`/`ListState`, `Paragraph`, styled `Line`/`Span`) against the pinned
  version. ratatui re-exports crossterm, so `crossterm::event::KeyEvent` is the
  input type without a separate dependency.

## Risks / Trade-offs

- **Snapshot brittleness.** Full-buffer string snapshots break on tiny layout
  tweaks. → Mitigation: assert on *content* (does the legend row contain "Ask
  AI"? does the left pane list "about.md"?) rather than committing exact buffer
  dumps, except for one small end-to-end layout snapshot.
- **ratatui churn.** 0.30 is recent. → Pinned in the workspace; the API is
  exercised by tests so an upgrade surfaces breakage immediately.

## Open Questions

- Exact key bindings beyond the README legend (Back/Quit/Toggle keys). Chosen
  pragmatically (Esc = Back, q = Quit, Tab = toggle nav) and documented in code;
  not spec-level, so changing them later needs no spec edit.
