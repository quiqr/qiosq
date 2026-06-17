---
# qiosq-cp3q
title: E4 — TUI shell (WP5.1 chrome)
status: completed
type: epic
priority: normal
created_at: 2026-06-17T15:56:53Z
updated_at: 2026-06-17T22:18:33Z
parent: qiosq-mer9
---

Goal: the two-pane shell, legend, and mode state machine.

## Tasks
- [x] T4.1 Two-pane ratatui layout + persistent function-key legend.
- [x] T4.2 Mode state machine: SiteList → Browse → ViewFile → Agent.
- [x] T4.3 Context-sensitive legend per mode.
- [x] T4.4 Dual navigation in Browse: raw `content/` tree and schema Menu (toggle).

## Tests
- [x] TestBackend snapshot per mode; transition tests.

## Summary of Changes

- qtui-ui implemented (was an E1 stub) on ratatui 0.30.1 (+ crossterm 0.29, re-exported). Depends on the pure crates qtui-storage + qtui-model. Pure core, terminal I/O at the edge — fully TestBackend-testable.
- AppState + Mode state machine: SiteList -> Browse{NavView} -> ViewFile -> Agent. Pure update(&mut AppState, KeyEvent) -> Option<Action> reducer (Enter opens; Esc pops one mode; Tab toggles Browse nav; q quits; F6 Ask AI; F3/F7/F9 verbs).
- Write-like verbs (New/Save/Discard/Ask AI) are emitted as Action values, never file ops (single-writer rule encoded in the type system).
- Context-sensitive function-key legend per mode (data + render): Browse=Preview/Toggle Nav/New; ViewFile=Ask AI only (no edit verb); Agent=Save/Discard/Back.
- render(&mut Frame, &AppState): vertical split (work area + 1-row legend); work area = left browser/launcher + right agent placeholder. Dual nav in Browse renders content tree OR schema menu; default view honours ui.show_schema_nav_first.
- 12 tests (TestBackend snapshots per mode + state-machine transitions + dual-nav content). cargo test --workspace (40 tests), clippy -D warnings, fmt, nix flake check all green.

Deferred to later epics: real terminal event loop (host binary, E5-E7), live preview URL lifecycle (E5 sets the field), agent pane content via ratatui-rmux (E7), opening schema-menu entries to their resolved paths (E5/E6).

OpenSpec change tui-shell archived (specs two-pane-layout, mode-state-machine, function-key-legend, dual-navigation promoted to openspec/specs/).
