## 1. Crate setup

- [x] 1.1 Add `ratatui` to the workspace and `qtui-ui` deps; depend on `qtui-storage` + `qtui-model` (pure types).
- [x] 1.2 Define `AppState`, `Mode` (SiteList, Browse{NavView}, ViewFile, Agent), `NavView` (ContentTree|SchemaMenu), and an `Action` enum (Quit, Open, Back, ToggleNav, AskAi, RequestNew/Save/Discard).

## 2. State machine (pure)

- [x] 2.1 `AppState::new(...)` starts in `SiteList`; holds sites, optional open-site content tree + nav model, selection indices, preview URL, default nav view.
- [x] 2.2 `update(&mut AppState, KeyEvent) -> Option<Action>`: SiteList→Browse on open; Browse→ViewFile on open; ViewFile→Agent on Ask AI; Back pops one mode; Tab toggles nav in Browse; q quits.
- [x] 2.3 Write-like verbs return Actions only; never touch the filesystem.

## 3. Legend (data + render)

- [x] 3.1 `legend(&AppState) -> Vec<LegendEntry{key,label}>` per mode (Browse: F5 Preview + toggle; ViewFile: F6 Ask AI, no edit verb; Agent: F7 Save/F9 Discard/Back).
- [x] 3.2 Legend changes when the mode changes.

## 4. Render (ratatui)

- [x] 4.1 `render(&mut Frame, &AppState)`: vertical split = work area + 1-row legend; work area = horizontal left (browser/launcher) + right (agent placeholder).
- [x] 4.2 Left pane content per mode: SiteList lists sites; Browse lists the content tree OR the schema menu (active NavView); ViewFile shows the read-only file path/placeholder.
- [x] 4.3 Right pane: labelled agent-pane placeholder. No terminal I/O in the library.

## 5. Tests (TestBackend)

- [x] 5.1 Snapshot/structure test per mode: layout has left+right panes and a bottom legend row; legend content matches the mode.
- [x] 5.2 ViewFile offers "Ask AI" and exposes no edit verb.
- [x] 5.3 Transition tests: SiteList→Browse→ViewFile→Agent and Back; Tab toggles nav; q -> Quit action; default nav view honoured.
- [x] 5.4 Dual nav: content-tree view lists content entries; schema-menu view lists model menu groups/entries.

## 6. Gate

- [x] 6.1 `cargo test --workspace` green; `cargo clippy --workspace --all-targets -- -D warnings` clean (dev shell).
- [x] 6.2 `cargo fmt --all --check` clean.
- [x] 6.3 `nix flake check` green.

## 7. Done

- [x] 7.1 OpenSpec change `tui-shell` validates.
- [x] 7.2 E4 beans closed; commit references `qiosq-cp3q`; change archived.
