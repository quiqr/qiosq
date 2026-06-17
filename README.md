# quiqr-tui (codename *Reveal*)

> A WordPerfect-5.1-style, kiosked terminal UI that lets non-technical authors
> manage a Quiqr/Hugo site over SSH and direct a coding agent (Claude Code) to
> write and publish content — without ever touching a shell.

This repository is a **bootstrap bundle**. It is intended to be handed to a
coding agent (Claude Code) which will build the Proof of Concept (PoC)
**autonomously**, using:

- **beans** — git-tracked, agent-first issue tracker (epics, milestones, tasks).
- **OpenSpec** — spec-driven proposals + tasks per epic (the *spec leads, code follows*).
- **Nix flakes** — reproducible dev shell + a QEMU/NixOS VM test that runs
  Quiqr Server so end-to-end tests are real, not mocked.

The PoC is the **alpha base** for later development. It is deliberately scoped:
SSH/kiosk hardening and the local wrapper app are *later* epics, not part of
milestone 1.

---

## What the product does (the story)

A non-technical user connects to a remote server (SSH or a wrapper app) and is
dropped — with no terminal knowledge required — into a two-pane TUI:

```
┌───────────────────────────────┬───────────────────────────────┐
│ LEFT: the application          │ RIGHT: the coding agent        │
│                                │                                │
│ • browse & open Quiqr sites    │ • Claude Code session          │
│ • on open: start `hugo server` │ • receives "@path/to/file …"   │
│   and show the preview URL     │   injected from the left pane  │
│ • navigate content/ OR the     │ • user finishes the sentence   │
│   Quiqr Menu (Singles /        │   in plain language            │
│   Collections) from the schema │ • writes content; preview      │
│ • open file = READ ONLY        │   auto-refreshes               │
│ • [F-key] sends file to agent  │ • can commit → git pipeline    │
├───────────────────────────────┴───────────────────────────────┤
│ F2 Browse  F3 New  F5 Preview  F6 Ask AI  F7 Save  F9 Discard  │
└────────────────────────────────────────────────────────────────┘
```

The left pane **never edits files**. All mutation happens through the agent on
the right. This single-writer design is what makes the kiosk safe.

See `docs/00-project-brief.md` for the full vision and `docs/01-architecture.md`
for component responsibilities.

---

## For the human operator (you)

1. Read `docs/00-project-brief.md` and `docs/01-architecture.md`. Adjust anything
   that doesn't match your intent **before** handing the repo to the agent.
2. Fill in the real Quiqr flake reference in `flake.nix` (marked `TODO(author)`).
   You are the author of the Quiqr Nix module/package, so wire in your actual
   input.
3. Copy `config/quiqr-tui.example.toml` → `config/quiqr-tui.toml` and set the
   Quiqr data dir / agent command for your machine.
4. Run `scripts/bootstrap.sh` (or let the agent run it) to initialize beans +
   OpenSpec and seed the epics.
5. Tell Claude Code: **"Run `beans prime`, read `CLAUDE.md`, then start the
   `poc-foundation` OpenSpec change."**

---

## For the coding agent

Your operating manual is **`CLAUDE.md`**. Read it first, every session. The
short version:

1. `beans prime` and heed its output. Beans is your durable memory and audit trail.
2. The work is organized as **epics** (in beans) and **OpenSpec changes** (one per
   epic). The spec is the source of truth; write/refine it before coding.
3. Nothing is "done" without tests. Every epic ends green on `nix flake check`,
   including the NixOS VM end-to-end test that boots Quiqr Server.
4. Commit small, reference bean IDs in commit messages, keep bean + spec state in
   sync with the code.

---

## Status

Pre-alpha. This bundle contains intent, specs, and harness — not yet an
implementation. The agent builds the implementation.
