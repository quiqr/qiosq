## Context

`host.run_agent_intent` spawns `agent.command`, writes the intent to stdin, and
waits for the sentinel — a one-shot, non-continuable model that the scripted/e2e
path relies on (the `fake-agent` binary writes a file + prints the sentinel).
`qtui-agent` already has the right abstraction: the `Agent` trait
(`start`/`send_intent`/`output`/`snapshot`/`is_complete`), an in-process
`FakeAgent`, and a compiled `RmuxAgent` (detached rmux session, pane (0,0),
workdir pinned). E8 wires a live `Agent` into the interactive loop while keeping
the subprocess fake path for CI.

## Goals / Non-Goals

**Goals:**
- Config-driven agent selection (fake vs rmux), behind `Box<dyn Agent>`.
- Lazy, site-pinned, reused live session in interactive mode.
- "Ask AI" → `send_intent` to the live session; pane streams `snapshot()` each
  tick; completion via the sentinel.
- CI/e2e unchanged (subprocess fake-agent).

**Non-Goals:**
- Multi-turn continued conversation / cursor handoff UX — that is E9 (this epic
  makes the live session work and stream; E9 builds the back-and-forth).
- Replacing `ratatui-rmux`'s widget — we render our own `PaneSnapshot` lines
  (already in place); the rmux widget can come later without a spec change.
- Changing the scripted path or `run_agent_intent`.

## Decisions

- **`Box<dyn Agent>` chosen by config.** `host::build_agent(&Config, site) ->
  Box<dyn Agent>`: if `Path::new(&config.agent.command).file_name()` is
  `fake-agent` (or matches the configured fake marker), construct `FakeAgent`;
  else `RmuxAgent::new(session_name)`. *Alternative considered:* a config enum
  for the agent kind — rejected; deriving from `agent.command` keeps config
  unchanged and matches how the e2e already sets `agent.command` to the
  fake-agent path.
- **Session name** for `RmuxAgent`: `qtui-<site-name>` (stable per site, so
  re-opening reuses it via `CreateOrReuse`). Sanitised to a valid `SessionName`.
- **Lazy start, held on `Host`.** `Host` gains `agent: Option<Box<dyn Agent>>`.
  On the first "Ask AI": build + `start(site.path, schema_ctx)` (schema_ctx empty
  for now), then `send_intent`. Subsequent Ask AI reuse it. The interactive loop
  owns the polling.
- **Streaming via the existing tick.** `interactive.rs` already draws every ~200
  ms. When an agent session exists, each tick calls `agent.snapshot()` and
  `state.set_agent_output(lines)`; the existing right-pane render shows them.
  `is_complete(sentinel)` flips a small status the legend/pane can show. Snapshot
  errors are surfaced into the pane text, never a crash.
- **FakeAgent in the live path.** `FakeAgent` implements `Agent` but produces no
  output on its own; for interactive use it is mostly a no-op visual. That's
  fine — the *real* value is `RmuxAgent`; `FakeAgent` selection keeps interactive
  runs daemon-free for local smoke/dev and is what unit tests drive.
- **Scripted path untouched.** `script.rs` keeps calling `run_agent_intent`
  (subprocess fake-agent) so the e2e VM and CI need no daemon.

## Risks / Trade-offs

- **`RmuxAgent` reconnects per call** (its current impl opens a client in
  `send_intent`/`snapshot`). For E8's poll-every-tick that's a little chatty but
  correct and simple; optimising to a persistent client is a later refinement,
  not a spec concern.
- **Live path can't run in CI** (needs the daemon). → Covered by unit tests with
  `FakeAgent` through the trait + a `#[ignore]`d real-rmux smoke test; the
  subprocess fake path remains the e2e gate.
- **Snapshot polling cost.** → Bounded by the existing ~200 ms tick; acceptable.

## Open Questions

- Whether interactive "Ask AI" with the *fake* agent should also drive the
  subprocess `fake-agent` (to actually write a file) for a local end-to-end
  demo. Decision: no — interactive fake is a visual no-op; the subprocess path is
  for scripted/e2e. Revisit if a local non-VM demo is wanted.
