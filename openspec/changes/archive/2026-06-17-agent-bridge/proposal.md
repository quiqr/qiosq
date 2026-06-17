## Why

This is the core interaction the whole product exists for: the user opens a file
**read-only**, presses "Ask AI", and the left pane injects `@{path} I want to do
the following… ` into the coding agent and hands over the cursor. The agent is
the single writer; the UI never edits. E4 already emits the `AskAi` action and
guarantees the viewer exposes no edit verb; E6 builds the agent side — the
`Agent` trait, the intent injection, completion detection — and the read-only
file viewer's content, with a **fake agent** so the bridge is tested without a
real LLM.

## What Changes

- Implement `qtui-agent` (currently an empty stub):
  - An `Agent` trait: `start(workdir, schema_context)`, `send_intent(file, …)`,
    `await_completion(sentinel)`, `snapshot()`.
  - `send_intent` formats and injects `@{path} I want to do the following… ` and
    leaves the cursor for the user (in headless tests the harness supplies the
    rest).
  - Completion detection: resolve when the configured **sentinel** appears in the
    agent's output.
  - A `FakeAgent` (in-process) plus a scripted `tests/bin/fake-agent` binary that
    echoes the sentinel and writes a known file — the deterministic, offline test
    double the testing strategy calls for.
  - A real `RmuxAgent` over `rmux-sdk` (detached session, agent in pane `(0,0)`,
    workdir pinned, restricted perms). It compiles and encodes the real bridge,
    but its live path needs an **rmux daemon** that is not available in the Nix
    check sandbox (tracked: `qiosq-rts9`), so it is exercised manually, not in CI.
- Show the opened file's text in the `ViewFile` mode as **read-only** (the host
  supplies the bytes; the library never reads or writes files).

## Capabilities

### New Capabilities
- `agent-trait`: the `Agent` abstraction that keeps Claude Code swappable and
  lets the bridge be driven by a fake agent in tests.
- `send-intent`: format and inject the `@{path} I want to do the following… `
  intent and hand the cursor to the user.
- `completion-detection`: detect task completion when the configured sentinel
  appears in the agent's output.
- `readonly-file-view`: render an opened file's contents read-only in the viewer,
  with no edit affordance.

### Modified Capabilities
<!-- None — qtui-agent was an empty stub from E1; ViewFile already exists from E4
     and gains rendered file content here without a spec-level behaviour change to
     the existing requirements. -->

## Impact

- `crates/qtui-agent`: real implementation; adds `rmux-sdk` (pinned) + an async
  runtime for the real impl; the trait + `FakeAgent` are sync-testable. May use
  `qtui-config` (sentinel, agent command) and `qtui-model` types (schema
  context). Adds a `tests/bin/fake-agent` binary.
- `qtui-ui`: `ViewFile` gains a way to display supplied file contents read-only.
- **Constraint (`qiosq-rts9`):** the rmux daemon is not in nixpkgs/CI, so the
  real `RmuxAgent`'s live path is not covered by `nix flake check`; the bridge
  logic is fully covered through the fake agent. No site files are ever written
  by this crate or the UI — only the agent writes, through its own process.
