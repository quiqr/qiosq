## Context

E6 is the core interaction: read-only view → "Ask AI" → intent injected into the
agent → agent writes → completion detected via the sentinel. E4 already emits the
`AskAi` action and forbids edit verbs in the viewer; E5 surfaces the live
preview. E6 implements the agent side in `qtui-agent` and renders the viewer's
file content in `qtui-ui`.

**Hard constraint discovered (`qiosq-rts9`):** `rmux-sdk 0.6.1` and
`ratatui-rmux 0.6.1` are on crates.io and build, but `rmux-sdk` is *daemon-backed*
(`Rmux::builder().connect_or_start()` needs a running rmux daemon from
helvesec/rmux). That daemon is not on crates.io, not in nixpkgs, and not on PATH,
so a test that drives anything through a *live* daemon cannot run in `nix flake
check`. The design therefore isolates rmux behind the `Agent` trait and tests the
bridge through a fake agent.

## Goals / Non-Goals

**Goals:**
- An `Agent` trait + intent formatting + sentinel detection, all testable with a
  fake agent (deterministic, offline) — both an in-process `FakeAgent` and the
  `tests/bin/fake-agent` scripted binary the testing strategy names.
- Render opened file contents read-only in `ViewFile` (host supplies bytes).
- A real `RmuxAgent` over `rmux-sdk` that *compiles* and encodes the real bridge.

**Non-Goals:**
- Running the real rmux daemon in CI (blocked by `qiosq-rts9`; packaging it is a
  later epic toward E7).
- Rendering the agent pane via `ratatui-rmux` (that is E7).
- Constraining agent output from field schemas (a later refinement; the trait
  takes a `schema_context` string so the seam exists).
- Any file writing by the UI or this crate — only the agent process writes.

## Decisions

- **`Agent` is a small synchronous-looking trait.** Methods: `start(workdir,
  schema_context)`, `send_intent(file_ref, extra)`, `poll_output() -> String` /
  `await_completion(sentinel)`, `snapshot() -> PaneSnapshot`. The real impl wraps
  the async rmux SDK behind a blocking facade (its own tokio runtime), so callers
  and tests stay synchronous. *Alternative considered:* make the whole trait
  async — rejected; it would force tokio into the UI/host for no benefit while
  the only async thing is the rmux client.
- **Intent formatting is a pure function.** `format_intent(path) ->
  "@{path} I want to do the following… "`. Pure and unit-tested; the agent impls
  just `send_text` it. Sentinel detection is likewise a pure predicate over
  accumulated output. This puts the spec's observable behaviour in pure code,
  independent of rmux.
- **Two test doubles.** (1) An in-process `FakeAgent` implementing `Agent`,
  accumulating sent text and a scripted output buffer — drives the trait-level
  bridge tests. (2) `tests/bin/fake-agent`: a tiny CLI that reads stdin and prints
  the sentinel (and writes a known file), for the e2e harness in E7 and to honour
  the testing strategy's explicit "scripted CLI" requirement.
- **`RmuxAgent` compiled, live path manual.** It is written against `rmux-sdk`
  (detached session via `EnsureSession … .detached(true)`, pane `(0,0)`,
  `send_text`/`wait_for_text`/`snapshot`). Behind a path that needs the daemon, it
  is covered by a `#[ignore]`d/manual test or an example, not by `flake check`.
  This keeps the real bridge in-tree and compiling while CI stays green.
- **Viewer content via the host.** `qtui-ui`'s `AppState` gains an
  `open_file_contents` the host sets when entering `ViewFile`; `render` shows it
  read-only. The library still never touches the filesystem (golden rule).

## Risks / Trade-offs

- **The real rmux path is unverified in CI.** → Mitigated by covering all
  observable bridge behaviour (intent text, sentinel, viewer) through the fake
  agent, and by keeping `RmuxAgent` compiling so type/API drift is caught at build
  time. Tracked by `qiosq-rts9` to add the daemon to Nix later.
- **tokio pulled into `qtui-agent`.** → Confined to that crate and its real impl;
  the trait + fake are runtime-free, so the pure crates and UI stay light.

## Open Questions

- Where the real `RmuxAgent` should source the agent command/args — config
  (`agent.command`/`args`) is the obvious answer (already in `qtui-config`); wired
  when the host event loop lands (E7). Not blocking E6's trait + tests.
