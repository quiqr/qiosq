---
# qiosq-me0f
title: E6 — Read-only file view + send-to-agent bridge
status: todo
type: epic
created_at: 2026-06-17T15:56:53Z
updated_at: 2026-06-17T15:56:53Z
parent: qiosq-mer9
---

Goal: the core interaction.

## Tasks
- [ ] T6.1 Read-only file viewer mode; "Ask AI" (F6) affordance.
- [ ] T6.2 `Agent` trait; Claude Code impl over rmux-sdk (detached, pinned workdir, restricted perms).
- [ ] T6.3 `send_intent`: inject `@{path} I want to do the following… ` + cursor handoff.
- [ ] T6.4 Completion sentinel detection (`wait_for_text`).
- [ ] T6.5 `fake-agent` test binary + bridge integration tests.

## Tests
- [ ] Bridge integration tests via fake agent; viewer is never writable.
