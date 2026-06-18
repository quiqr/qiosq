## Why

The interactive `qtui` proves the shape but isn't yet *usable*: pressing "Ask AI"
runs the configured command as a **one-shot subprocess** (write the intent to
stdin, wait for the sentinel, capture stdout) — there is no live, continuable
agent session, and the real `RmuxAgent` (a detached Claude-over-rmux session) is
compiled but unwired. E8 makes a configured real agent run *inside* the running
app: a live session pinned to the site, the open file's intent sent to it, and
the agent pane streaming its output each tick. This is the core leap from PoC to
usable app.

## What Changes

- The host selects the agent implementation from config: when `agent.command`
  names the in-tree **fake agent**, use the in-process `FakeAgent`; otherwise use
  the real **`RmuxAgent`** (detached rmux session, workdir pinned to the site).
- The interactive loop holds a **long-lived agent** started lazily on the first
  "Ask AI" and reused across the session.
- "Ask AI" sends the `@{path} I want to do the following… ` intent to the **live
  session** (instead of a one-shot subprocess).
- Each render tick the loop pushes `agent.snapshot()` into the UI so the agent
  pane **streams live output**; completion is recognised via the configured
  sentinel.
- The **scripted/headless path is unchanged**: the e2e VM and `--script` runs
  keep using the deterministic subprocess fake-agent, so `nix flake check` stays
  green and offline (no rmux daemon, no real LLM in CI).

## Capabilities

### New Capabilities
- `live-agent`: a configured real coding agent runs as a live, reusable session
  in the interactive TUI — agent selection, lazy start pinned to the site, intent
  sent to the live session, and per-tick snapshot streaming.

### Modified Capabilities
<!-- agent-pane-render + host-event-loop already describe snapshot rendering and
     the host loop; live-agent adds the live-session wiring on top without
     changing their existing requirements. -->

## Impact

- `crates/qtui` (`host.rs` + `interactive.rs`): the host owns an
  `Option<Box<dyn Agent>>`, an agent-selection function, and a lazy-start; the
  loop sends the intent to the live agent and polls its snapshot each tick. The
  one-shot `run_agent_intent` stays for the scripted/e2e fake path.
- `crates/qtui-agent`: used as-is (the `Agent` trait, `RmuxAgent`, `FakeAgent`);
  no API change expected.
- Runtime: the real path needs the rmux daemon on `PATH` (provided by the dev
  shell; CI uses the fake agent, so it is unaffected — `qiosq-rts9`).
- Read-only invariant holds: only the agent process writes content; the UI
  streams snapshots and never attaches the user to the raw session.
