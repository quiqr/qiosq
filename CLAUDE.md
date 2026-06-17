# CLAUDE.md — Operating Manual for the Coding Agent

**IMPORTANT: before you do anything else, run `beans prime` and heed its
output.** Then read this file fully. Re-read it at the start of every session.

You are building the **quiqr-tui** PoC (codename *qiosq*) autonomously. This
file is your contract. The product vision is in `docs/00-project-brief.md`; the
architecture is in `docs/01-architecture.md`; the testing bar is in
`docs/02-testing-strategy.md`. Read all three before writing code.

---

## 0. The two tools you must use

### beans (issue tracker = your durable memory)
- Run `beans prime` first; it tells you the exact CLI for this install.
  **Do not assume command syntax — discover it.** If `beans prime` is
  unavailable, run `beans --help` and `beans <subcommand> --help`.
- Work is organized as **epics** (large goals) containing **tasks**. Milestones
  group epics. The seed plan is in `docs/epics-seed.md`; translate it into beans
  during bootstrap if `scripts/bootstrap.sh` hasn't already.
- Every code commit references the relevant bean ID(s) in the message.
- A bean moves to in-progress when you start it and is closed only when its
  acceptance criteria — **including tests** — are met.
- Discovered work → create a new bean immediately (tag it `discovered`), don't
  silently expand scope.

### OpenSpec (spec-driven development = source of truth)
- Run `openspec --help` to learn the exact workflow commands for this install
  (e.g. init, change creation, validation, archive). **Discover, don't assume.**
- **One OpenSpec change per epic.** The spec leads; the code follows. Before
  implementing an epic, write/refine its proposal, specs, and tasks, and get the
  spec validating. `openspec/changes/poc-foundation/` is a worked example —
  follow its shape for the other epics.
- When an epic is complete and merged, archive its change per OpenSpec's flow so
  specs stay synchronized with the code.

The mental loop for every epic:
**propose (OpenSpec) → break into beans → implement against spec → test green →
commit referencing beans → archive the change.**

---

## 1. Golden rules (do not violate)

1. **The left pane never writes files.** All site mutation happens through the
   agent in the right pane. If you find yourself adding a file-write to the UI or
   storage layer, stop — that's an architecture change and needs an OpenSpec
   proposal first.
2. **`quiqr/model/` is read-only.** Never edit a site's Quiqr schema.
3. **Tests gate everything.** No bean closes red. `nix flake check` must pass,
   including the VM e2e test, before an epic is done.
4. **Nix from the start.** All dev/test/build go through the flake. Don't add a
   dependency the flake doesn't provide; add it to the flake instead.
5. **rmux keeps the agent detached.** Never `attach` the user to the raw agent
   session; only render snapshots via `ratatui-rmux`.
6. **Small commits, spec + bean + code in sync.** Each commit references beans
   and, where relevant, updates the OpenSpec change.
7. **When uncertain about a tool's real interface, run its `--help`.** This
   applies to beans, openspec, rmux, hugo, and the Quiqr CLI/module.

## 2. Project shape

Target Cargo workspace (see `docs/01-architecture.md` for contracts):

```
crates/qtui-config  qtui-storage  qtui-model  qtui-agent  qtui-preview  qtui-ui  qtui
```

Build order roughly follows the epics in `docs/epics-seed.md`:
foundation → storage → model → ui shell → preview → file view + send-to-agent →
agent pane → testing/e2e.

## 3. Configuration is first-class

Everything environment-specific is in the config file
(`config/quiqr-tui.toml`, example provided). At minimum: Quiqr data dir, agent
command + args, hugo binary, preview port range, completion sentinel. The PoC
must run on another machine by editing only that file. Claude Code is the
default agent but must sit behind the `Agent` trait so it can be swapped.

## 4. The agent bridge (rmux)

- Use `rmux-sdk` (pin the version; it's young). Detached session, agent in pane
  `(0,0)`, workdir pinned to the opened site repo, restricted permissions.
- `send_intent` injects `@{path} I want to do the following… ` and hands the
  cursor to the user (in headless tests, the harness supplies the rest).
- Detect completion via the configured **sentinel** the agent prints when done
  (`pane.wait_for_text`). Render via `ratatui-rmux` snapshots each tick.
- For tests, use the **fake agent** (`tests/bin/fake-agent`), not a real LLM.

## 5. Quiqr schema → navigation

Parse `quiqr/model/base.yaml` and merge `quiqr/model/includes/*` (each include
is a top-level root: collections, menu, singles, dynamics). Produce a
`NavigationModel` the UI renders as the WP-style Menu. Also expose field
schemas so the agent prompt can be constrained to produce Quiqr-valid front
matter. Be tolerant of partial/legacy schemas — never panic on a real site.

## 6. Preview

On site open, start `hugo server` (free port in the configured range, never
`:13131`), surface the URL, stop on close. The preview is a URL the user opens
in a browser; we do not render the site in the TUI.

## 7. Definition of done (per epic)

- OpenSpec change validated and (when merged) archived.
- All associated beans closed.
- Unit + integration + UI tests written and green.
- The VM e2e scenario still passes (`nix flake check`).
- Commits reference bean IDs; spec and code agree.

## 8. What NOT to build in the PoC

SSH/kiosk hardening, the wrapper client app, multi-user auth from Quiqr's user
JSON, the publish pipeline, and an embedded webview. These are later epics;
leave clean seams (e.g. a `kiosk` module stub, an auth stub) but don't
implement them. If you think one is necessary for the PoC, propose it via
OpenSpec and create a bean — don't just build it.

## 9. When you get stuck

- Re-read the relevant `docs/` file and the epic's OpenSpec change.
- Check the real tool interface with `--help`.
- Record the blocker as a bean (tag `blocked`) with what you tried.
- Prefer the smallest change that makes a test pass, then refactor.
