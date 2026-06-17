# 03 — Glossary

Shared vocabulary so humans and the agent mean the same thing.

- **Quiqr** — Local-first flat-file CMS for Hugo (and other SSGs). Stores sites
  as plain files under the *Quiqr Data directory*. We share that directory; there
  is no API to integrate against.
- **Quiqr Server edition** — Quiqr run as a standalone web server (same codebase
  as the Electron desktop app, different run mode). Already running on the target
  server.
- **Quiqr Data directory** — Where Quiqr keeps all sites (e.g. `~/Quiqr Data`).
  Each subdirectory is one site.
- **Quiqr schema / model** — `quiqr/model/base.yaml` plus `quiqr/model/includes/*`
  inside a site. Defines collections, singles, menu, dynamics. Drives Quiqr's
  forms; for us it drives navigation and constrains agent output. **Read-only.**
- **Single** — A schema-defined single content entity (e.g. the About page).
- **Collection** — A schema-defined set of like items (e.g. blog posts).
- **content/** — The Hugo content tree; the files authors work with. The agent
  may write here; the left pane only reads it.
- **rmux** — Rust multiplexer with an SDK + ratatui widget. We use it to spawn,
  drive, and render the coding-agent session inside our own UI, keeping the user
  out of any raw terminal session.
- **Agent / coding agent** — Claude Code by default, behind the `Agent` trait.
  The only writer of site content. Sandboxed and pinned to the site repo.
- **Completion sentinel** — A string the agent prints when a task is finished, so
  we can detect completion via `wait_for_text`.
- **Fake agent** — A tiny scripted test binary that imitates the agent
  (echoes the sentinel, writes a known file) for deterministic, offline tests.
- **WP5.1 chrome** — The WordPerfect-5.1-inspired UI: full-screen work area +
  persistent context-sensitive function-key legend; no shell visible.
- **Kiosk** — The locked-down mode (later epic) where the user cannot escape the
  TUI to a shell.
- **beans** — Git-tracked, agent-first issue tracker. Holds **epics**,
  **milestones**, and **tasks**. The agent's durable memory/audit trail. Run
  `beans prime` to learn its interface.
- **OpenSpec** — Spec-driven-development CLI. Work is organized as **changes**
  (proposal + specs + tasks) that lead the code. One change per epic.
- **Epic** — A large goal (E1…E7 for the PoC). Maps to one OpenSpec change and a
  beans epic.
- **Milestone** — A group of epics (M1 = the PoC alpha base).
- **PoC / alpha base** — This deliverable: the two-pane experience proven e2e
  against Quiqr Server in a Nix VM test.
