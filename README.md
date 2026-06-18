# quiqr-tui (codename *qiosq*)

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

## Releasing (maintainers)

Releases are cut with `scripts/release.sh`, run from the Nix dev shell (it needs
`jj`, `git`, `gh`, and `cargo` — all provided by `nix develop`). The workspace
version lives only in `Cargo.toml`; the flake derives its version from there.

1. Between releases, add notes under the `## [Unreleased]` heading in
   `CHANGELOG.md` as you work.
2. Make sure `gh` is authenticated (`gh auth status`) and the working copy is
   clean.
3. Cut the release:

   ```bash
   scripts/release.sh patch      # or: minor | major
   scripts/release.sh minor --dry-run   # preview without changing anything
   ```

   The script computes the next SemVer version, bumps `Cargo.toml` (refreshing
   `Cargo.lock`), promotes the `Unreleased` changelog entries into a dated
   `## [X.Y.Z]` section, commits the bump, creates and pushes the annotated tag
   `vX.Y.Z`, and creates the GitHub release with those notes.

   It refuses to run on a dirty working copy unless you pass `--allow-dirty`
   (which commits only the files the script itself changed). Use `--dry-run` to
   see the plan first.

---

## Status

Alpha base. Milestone M1 — the two-pane PoC — is implemented and proven end to
end: `nix flake check` passes all unit/integration tests **and** a NixOS VM test
that boots Quiqr Server, runs `qtui` against it, and asserts the agent writes
content on disk. Run `qtui init` to point it at your Quiqr storage, then `qtui`.
Later milestones (SSH/kiosk hardening, the wrapper app, the publish pipeline)
are not yet built.
