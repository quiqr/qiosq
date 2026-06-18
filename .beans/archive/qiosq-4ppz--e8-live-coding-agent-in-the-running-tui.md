---
# qiosq-4ppz
title: E8 — Live coding agent in the running TUI
status: completed
type: epic
priority: high
created_at: 2026-06-18T10:07:49Z
updated_at: 2026-06-18T17:04:51Z
parent: qiosq-csrd
---

Goal: a configured real agent (Claude Code) runs INSIDE the interactive app, the open file's intent is sent to it, and the agent pane shows live output — the core 'usable' leap.

Today: the interactive loop runs the agent as a one-shot subprocess (host.run_agent_intent spawns agent.command, writes the intent, waits for the sentinel); RmuxAgent (the real detached-session impl) is compiled but not wired into the loop.

## Tasks (draft)
- [ ] Select RmuxAgent vs FakeAgent from config (e.g. agent.command == fake-agent -> fake; else real) and own its lifecycle across the loop.
- [ ] Ask AI (F6) sends the @{path} intent to the LIVE session and hands the user the cursor (real interactive input), not a one-shot spawn.
- [ ] Poll/stream the agent pane snapshot each tick (ratatui-rmux or our PaneSnapshot) so output appears live; detect completion via the sentinel.
- [ ] Keep the fake-agent path for the e2e/headless tests (no live LLM in CI).
- [ ] Needs the rmux daemon on PATH (qiosq-rts9 interim build works).

## Tests
- [ ] Bridge/loop tests via the fake agent; a manual/ignored real-rmux smoke test.

## Summary of Changes
Wired a live coding-agent session into the interactive TUI (the PoC->usable leap).
- host.rs: Host owns Option<Box<dyn Agent>>. build_agent(&Config,&Site) selects the impl from config — agent.command basename 'fake-agent' -> in-process FakeAgent; otherwise RmuxAgent::new(sanitised 'qtui-<site>' session). ensure_agent_started lazily start()s once (pinned to the site working copy) and reuses it; send_live_intent sends the @{path} intent to the LIVE session; poll_agent refreshes the pane from agent.snapshot() each tick (errors surfaced in-pane, never a crash) and appends a '— task complete —' line when is_complete(sentinel).
- interactive.rs: the loop now poll_agent()s every tick (so output streams between keypresses), and Ask AI calls send_live_intent instead of the one-shot subprocess.
- script.rs / e2e UNCHANGED: still the deterministic subprocess fake-agent (run_agent_intent), so nix flake check stays green + offline (no rmux daemon in CI).
- Tests: host unit tests (agent selection picks fake; lazy-start-once + reuse + poll populates the pane via FakeAgent); the real rmux path is covered by qtui-agent's existing #[ignore]d live test. cargo test --workspace, clippy -D warnings, fmt, and nix flake check (incl VM e2e) all green. CHANGELOG Unreleased entry added.

Follow-on (E9, qiosq-clob): multi-turn continued conversation + cursor handoff UX. Needs the rmux daemon for the real path (qiosq-rts9 interim build works).

OpenSpec change live-agent archived (capability live-agent added).
