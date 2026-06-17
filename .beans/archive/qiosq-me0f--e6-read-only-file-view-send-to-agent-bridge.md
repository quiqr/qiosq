---
# qiosq-me0f
title: E6 — Read-only file view + send-to-agent bridge
status: completed
type: epic
priority: normal
created_at: 2026-06-17T15:56:53Z
updated_at: 2026-06-17T22:50:01Z
parent: qiosq-mer9
---

Goal: the core interaction.

## Tasks
- [x] T6.1 Read-only file viewer mode; "Ask AI" (F6) affordance.
- [x] T6.2 `Agent` trait; Claude Code impl over rmux-sdk (detached, pinned workdir, restricted perms).
- [x] T6.3 `send_intent`: inject `@{path} I want to do the following… ` + cursor handoff.
- [x] T6.4 Completion sentinel detection (`wait_for_text`).
- [x] T6.5 `fake-agent` test binary + bridge integration tests.

## Tests
- [x] Bridge integration tests via fake agent; viewer is never writable.

## Summary of Changes

- qtui-agent implemented (was an E1 stub). `Agent` trait (start/send_intent/output/snapshot/is_complete) keeps Claude Code swappable; PaneSnapshot type.
- Pure, unit-tested bridge logic: format_intent(path) -> '@{path} I want to do the following… ' (trailing space hands the user the cursor); contains_sentinel(output, sentinel) (empty sentinel never matches).
- FakeAgent (in-process Agent) records sent text + replays scripted output. tests/bin/fake-agent scripted CLI reads stdin, optionally writes a known file (QTUI_FAKE_WRITE), prints the sentinel (QTUI_FAKE_SENTINEL) — for bridge tests + the E7 e2e harness.
- RmuxAgent over real rmux-sdk 0.6.1 (verified API): detached session (EnsureSession.detached(true)), pane (0,0), working_directory pinned, send_text intent, snapshot via visible_lines; blocking facade over the async SDK. COMPILES against the real crate.
- qtui-ui ViewFile now renders host-supplied file contents read-only (AppState::set_open_file(path, contents) / open_file_contents()); still no edit verb; Ask AI (F6) emits the @path intent action.
- Tests: 8 in qtui-agent (intent prefix, sentinel predicate, NotStarted guard, fake bridge completion+snapshot, scripted-binary writes-file+prints-sentinel, no-site-files-written, constructs-without-daemon) + 1 #[ignore]d live rmux test + a qtui-ui read-only-viewer test. 55 workspace tests; clippy -D warnings, fmt, nix flake check all green.

CONSTRAINT (discovered, bean qiosq-rts9, left OPEN/blocked): rmux-sdk is daemon-backed and the rmux daemon is not in nixpkgs/CI, so the real RmuxAgent live path is NOT covered by nix flake check — all observable bridge behaviour is covered through the fake agent instead. Packaging the daemon for Nix is future work (toward E7).

OpenSpec change agent-bridge archived (specs agent-trait, send-intent, completion-detection, readonly-file-view promoted to openspec/specs/).
