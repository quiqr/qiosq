# 00 — Project Brief

**Codename:** qiosq (a q'ification of *kiosk*; was *Reveal*, a nod to WordPerfect 5.1's *Reveal Codes*)
**Repo:** qiosq (crates remain `quiqr-tui` / `qtui`)
**Status:** PoC / alpha base

---

## 1. The problem

Non-technical content authors need to manage a Quiqr/Hugo website but should
never see a shell, learn git, or touch raw files. Existing CMS frontends are
web apps; we want a **terminal-native, kiosked** experience delivered over SSH
to a server that already runs **Quiqr Server edition**, where an AI coding agent
does the actual writing and publishing.

The guiding metaphor is **WordPerfect 5.1**: a clean full-screen work area, a
persistent function-key legend, context-sensitive modes, and *no operating
system visible underneath*. People managed real work in TUIs for years; the
experience is approachable when the application commits to that discipline.

## 2. The core insight

The left pane is **read-only**. It browses and launches; it never mutates.
Every change to the site happens through the **coding agent** in the right
pane. This gives us:

- **One writer.** Only the sandboxed agent writes, so there is a single,
  auditable mutation path to defend.
- **A trivial lockdown surface on the left.** A browser/launcher with no write
  capability cannot do damage.
- **A natural verb set** for non-technical users: *browse*, *open*, *ask AI*.

## 3. Fixed components

| Component | Role | Notes |
|---|---|---|
| **Quiqr (storage + schema)** | Source of truth for site structure & content | Flat files on disk; no API. We share the directory directly. Author of Quiqr is on this team. |
| **Quiqr schema** (`quiqr/model/base.yaml` + `includes/`) | Drives user-facing navigation **and** constrains agent output | Collections/Singles/Menu → the WP-style nav. Field defs → prompt constraints so agent output validates in Quiqr forms. |
| **rmux** | Supervises & renders the agent session | `rmux-sdk` spawns/drives the agent pane; `ratatui-rmux` renders snapshots inside our chrome. User never sees a raw attached session. |
| **Coding agent (Claude Code)** | The worker that writes content & commits | Configurable via the agent adapter. Claude Code from day one; pane abstracted behind a trait. |
| **Kiosked TUI** | The whole user-facing product | ratatui, two panes, WP5.1 chrome. |

## 4. The user flow (target)

1. User connects via SSH (or wrapper app) → login spawns the TUI as their shell.
2. TUI lists the Quiqr sites the user is permitted to see (gated by Quiqr's
   user JSON).
3. User opens a site → TUI starts `hugo server` for it and shows the live
   preview URL in the view (the user opens that URL in their own browser).
4. User navigates either the raw `content/` tree **or** the schema-driven Quiqr
   Menu (Singles / Collections).
5. User opens a file (read-only) and presses the "Ask AI" key. The left pane
   injects `@path/to/file I want to do the following… ` into the agent pane and
   hands the user the cursor.
6. User finishes in plain language ("use this raw blog file to make a new post;
   here's the text; translate to English in our tone of voice").
7. Agent writes to `content/`; because Hugo is serving, the preview refreshes.
8. Depending on settings, the user asks the agent to commit; a git pipeline
   publishes. **Publishing is out of scope for us.**

## 5. Scope of the PoC (milestone 1)

**In scope** — prove the two-pane experience end to end on a server running
Quiqr Server:

- Configurable foundation (agent command, Quiqr data dir, ports) via a config file.
- Nix flake dev shell + a NixOS/QEMU VM test that boots Quiqr Server.
- Quiqr storage + schema reader (parse `base.yaml` + `includes/` → navigation model).
- Two-pane WP5.1 TUI shell with function-key legend and mode state machine.
- Site browser; on open, auto-start `hugo server` and surface the URL.
- Dual navigation: raw `content/` tree and schema-driven Singles/Collections view.
- Read-only file viewer.
- Send-to-agent bridge via rmux (`@path …` injection + cursor handoff).
- Agent pane rendering Claude Code via `ratatui-rmux`, with a completion sentinel.
- Thorough automated tests, including an e2e scenario against the VM.

**Explicitly out of scope (later epics)** — noted so they aren't lost:

- SSH server hardening / guaranteed kiosk (`ForceCommand`, restricted user,
  disabled escapes, respawn-on-exit). This is its own security epic and a
  different skill set (sshd/systemd), not Rust.
- The local wrapper app (SSH client experience).
- Multi-user authorization derived from Quiqr's user JSON (PoC may stub this).
- The git/publish pipeline (owned elsewhere).
- Embedded webview preview (PoC surfaces a URL only).

## 6. Known tensions & decisions

- **Preview is a URL, not an in-pane render.** A pure SSH terminal cannot show
  the themed Hugo site. Decision: surface the `hugo server` URL; the user opens
  it in their browser. In-app webview is a future wrapper-app concern.
- **Concurrency.** Quiqr Server and the kiosk can both touch one site. Decision
  for PoC: the kiosk session is the practical sole writer; git is the arbiter;
  writes are atomic (temp + rename). Do not edit `quiqr/model/` — treat schema
  as read-only.
- **Agent abstraction.** Claude Code is the only agent for the PoC, but it sits
  behind an `Agent` adapter trait from day one (cheap now, painful to retrofit).

## 7. Definition of "alpha base"

The PoC is the alpha base when: a fresh checkout, `nix develop` + `nix flake
check`, passes all unit/integration tests **and** the VM e2e test demonstrates
the full flow (open site → hugo serving → open file read-only → send to agent →
agent writes content → change visible) against a real Quiqr Server instance.
