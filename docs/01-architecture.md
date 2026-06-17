# 01 — Architecture

This document defines component boundaries and the target crate layout. The
agent should treat these as the intended shape but may refine via OpenSpec
proposals where it finds a better design — *changes to architecture go through a
spec, not a surprise commit.*

## 1. The stack

```
            Kiosk lockdown  (LATER EPIC — owns the TTY: ForceCommand, respawn)
                  │
            quiqr-tui binary  ── WP5.1 chrome: work area + F-key legend
                  │
   ┌──────────────┼───────────────────────────────────────────┐
   │              │                                             │
 Quiqr schema   rmux (drive/render)                       Quiqr storage
 (READ)           │                                          (R/W via agent)
   │              └── Claude Code (Agent adapter)                │
 defines nav         writes content, scoped to content/    shared flat files;
 + constrains        + emits completion sentinel           git is the arbiter
 agent prompts
```

## 2. Crate layout (Cargo workspace)

```
crates/
  qtui-config/      # config file load/validate (agent cmd, data dir, ports)
  qtui-storage/     # Quiqr/Hugo storage layer: locate data dir, enumerate sites
  qtui-model/       # parse quiqr/model/base.yaml + includes/ → navigation model
  qtui-agent/       # Agent adapter trait + Claude Code impl over rmux-sdk
  qtui-preview/     # start/stop `hugo server` per site, track URL & lifecycle
  qtui-ui/          # ratatui: two-pane layout, legend, mode state machine, widgets
  qtui/             # binary: wires everything together; the kiosk entrypoint
```

Rationale: each crate is independently testable. `qtui-model`, `qtui-storage`,
and `qtui-config` are pure(ish) and unit-test heavily. `qtui-agent` and
`qtui-preview` own external processes and get integration tests. `qtui-ui` is
testable with ratatui's `TestBackend`. The VM e2e test exercises `qtui` against
a real Quiqr Server.

## 3. Component contracts

### qtui-config
- Loads a TOML config (see `config/quiqr-tui.example.toml`).
- Resolves the Quiqr data directory, the agent command + args, hugo binary,
  preview port range, and the completion sentinel string.
- Validates on load; refuses to start with a clear error if misconfigured.

### qtui-storage
- Given the data dir, enumerate sites (each subdir that looks like a Quiqr/Hugo
  site: has `config.*` and a `quiqr/` folder).
- Provide a read-only file tree of `content/` with derived/generated dirs
  hidden (`public/`, `resources/`, `.quiqr-cache/`, `.git`, `themes/`).
- Never writes. (Writes are the agent's job, through the repo working dir.)

### qtui-model
- Parse `quiqr/model/base.yaml` and merge `quiqr/model/includes/*` (each include
  file is a top-level config root: collections, menu, singles, dynamics …).
- Produce a `NavigationModel`: the Menu with its Singles and Collections, each
  mapped to the files/paths it represents.
- Expose per-collection/single **field schemas** so `qtui-agent` can build
  prompt constraints (so agent output validates in Quiqr's forms).
- Treat the model as **read-only**. Be tolerant of partial/legacy schemas.

### qtui-preview
- On site open: start `hugo server` with the site's config, on a free port
  inside the configured range (avoid Quiqr's default `:13131`).
- Surface the URL to the UI; stream readiness ("Web Server is available at…").
- Stop the server on site close / app exit. One server at a time for the PoC.

### qtui-agent
- `trait Agent`: `start(workdir, schema_context)`, `send_intent(file_ref, ...)`,
  `is_idle()/await_completion(sentinel)`, `snapshot() -> PaneSnapshot`.
- Claude Code impl uses **rmux-sdk**: detached session, agent spawned in pane
  `(0,0)` with workdir pinned to the site repo and restricted permissions;
  `pane.send_text(...)` to inject intent; `pane.wait_for_text(sentinel)` to
  detect completion; `pane.snapshot()` each render tick.
- The agent session stays **detached**; we render snapshots via `ratatui-rmux`.
  The user is NEVER attached to the raw session.
- The "send file to agent" action injects `@{path} I want to do the following… `
  and leaves the cursor in the pane for the user to continue typing.

### qtui-ui
- Two panes: left (browser/launcher), right (agent snapshot widget).
- WP5.1 chrome: a persistent bottom **function-key legend** that reflects the
  current **mode**. Modes form a small state machine:
  - `SiteList` → choose a site
  - `Browse` → dual nav: `content/` tree OR schema Menu (toggle); shows preview URL
  - `ViewFile` → read-only viewer; offers "Ask AI" (F6)
  - `Agent` → cursor in the agent pane; legend shows Save/Discard/Back
- All write-like verbs (New, Save, Discard) are *requests routed to the agent*,
  not direct file ops.

### qtui (binary)
- Loads config, constructs the storage/model/preview/agent services, runs the UI
  loop, handles the mode state machine, and (in a later epic) the kiosk lockdown.

## 4. Agent sandboxing (PoC level)

- Working directory pinned to the opened site's repo.
- Restricted permission mode / tool allowlist so the agent cannot run arbitrary
  destructive commands.
- `quiqr/model/` is read-only to the agent (schema integrity).
- Commits (when asked) go to a working branch, never directly to `main`.
- Publishing/pipeline is out of scope.

## 5. Why rmux (and not raw tmux send-keys)

We want the *agent visible inside our own chrome* while the user stays in our
UI. rmux-sdk gives typed session control + snapshots and a ratatui widget, so we
embed and drive the agent without ever exposing a raw multiplexer the user could
escape into. This is the single reason rmux is a core component rather than an
implementation detail. (rmux is young — pin the version, expect to file issues.)

## 6. Testing seams (see docs/02-testing-strategy.md)

- `qtui-model`: golden-file tests over example `base.yaml` + includes.
- `qtui-storage`: tempdir fixtures shaped like Quiqr sites.
- `qtui-ui`: ratatui `TestBackend` snapshot tests of each mode + legend.
- `qtui-agent`: a **fake agent** (a tiny scripted CLI) driven through rmux to
  test the bridge without depending on a real LLM.
- e2e: NixOS VM test boots Quiqr Server + runs `qtui` headless against it.
