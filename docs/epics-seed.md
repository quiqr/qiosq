# Epics Seed Plan

This is the human-authored breakdown to load into **beans** during bootstrap.
The agent should translate these into beans (epics → tasks) using the real
`beans` CLI discovered via `beans prime` / `beans --help`. Keep bean IDs
referenced in commits and OpenSpec changes.

> One **OpenSpec change** per epic (E1…E7). Write/refine the spec before coding.

## Milestone M1 — "Two-pane PoC against Quiqr Server"

The alpha base. Complete when `nix flake check` passes including the VM e2e test
that demonstrates the full flow.

---

### E1 — Foundation & Nix
*Goal: a reproducible workspace that builds and tests under Nix.*
- T1.1 Cargo workspace with the seven crates (empty but compiling).
- T1.2 `flake.nix`: dev shell (rust toolchain, hugo, beans, openspec, the agent
  CLI, fake-agent build), `nix build`, `nix flake check` wiring.
- T1.3 `qtui-config`: TOML load + validation; example config; clear errors.
- T1.4 CI entry: `nix flake check` runs unit tests.
- T1.5 Initialize OpenSpec + beans in-repo; AGENTS.md present and correct.
- *Tests:* config load/validate unit tests; `nix flake check` green.

### E2 — Quiqr storage layer
*Goal: read-only knowledge of the Quiqr/Hugo storage.*
- T2.1 Locate data dir from config; enumerate sites (has `config.*` + `quiqr/`).
- T2.2 Read-only `content/` tree with derived/generated dirs hidden.
- T2.3 Site fixtures for tests.
- *Tests:* enumeration + tree-filtering unit tests over tempdir fixtures.

### E3 — Quiqr schema → navigation model
*Goal: turn the schema into the WP-style Menu and into agent constraints.*
- T3.1 Parse `quiqr/model/base.yaml`.
- T3.2 Merge `quiqr/model/includes/*` (collections, menu, singles, dynamics).
- T3.3 Build `NavigationModel` (Menu → Singles/Collections → paths).
- T3.4 Expose per-collection/single field schemas for prompt constraints.
- T3.5 Tolerant handling of partial/legacy/malformed schemas.
- *Tests:* golden-file tests incl. partial/malformed cases.

### E4 — TUI shell (WP5.1 chrome)
*Goal: the two-pane shell, legend, and mode state machine.*
- T4.1 Two-pane ratatui layout + persistent function-key legend.
- T4.2 Mode state machine: SiteList → Browse → ViewFile → Agent.
- T4.3 Context-sensitive legend per mode.
- T4.4 Dual navigation in Browse: raw `content/` tree and schema Menu (toggle).
- *Tests:* TestBackend snapshot per mode; transition tests.

### E5 — Site open + Hugo preview
*Goal: opening a site serves it and surfaces the URL.*
- T5.1 `qtui-preview`: start `hugo server` on a free port (avoid `:13131`).
- T5.2 Detect readiness, surface URL in the view; stop on close/exit.
- T5.3 Wire site-open in the UI to preview start.
- *Tests:* integration test starts hugo on a fixture site, asserts reachable +
  clean shutdown + port-collision handling.

### E6 — Read-only file view + send-to-agent bridge
*Goal: the core interaction.*
- T6.1 Read-only file viewer mode; "Ask AI" (F6) affordance.
- T6.2 `Agent` trait; Claude Code impl over rmux-sdk (detached, pinned workdir,
  restricted perms).
- T6.3 `send_intent`: inject `@{path} I want to do the following… ` + cursor
  handoff.
- T6.4 Completion sentinel detection (`wait_for_text`).
- T6.5 `fake-agent` test binary + bridge integration tests.
- *Tests:* bridge integration tests via fake agent; viewer is never writable.

### E7 — Agent pane render + E2E proof
*Goal: see the agent work, and prove the whole flow in a VM.*
- T7.1 Render agent snapshots via `ratatui-rmux` in the right pane.
- T7.2 NixOS VM test booting Quiqr Server (author's Quiqr module).
- T7.3 Provision sample site + schema into the VM's Quiqr data dir.
- T7.4 Headless/scripted `qtui` run exercising the full flow with fake agent.
- T7.5 Assert on-disk result (+ optionally served output).
- *Tests:* `checks.<system>.e2e` passes in `nix flake check`.

---

## Later milestones (DO NOT build in PoC — record as epics, leave stubs)

### M2 — Kiosk & access (separate security epic)
- E8 SSH hardening: restricted user, `ForceCommand`, disabled escapes/forwarding.
- E9 Lockdown in-binary: trap signals, panic-relaunch, seal rmux prefix/detach.
- E10 Multi-user authorization from Quiqr's user JSON.

### M3 — Delivery experience
- E11 Local wrapper app (SSH client UX).
- E12 Optional embedded webview preview.

### M4 — Publish integration
- E13 Commit-to-branch / PR flow surfaced as a safe verb (pipeline owns publish).
