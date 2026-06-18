---
# qiosq-clob
title: E9 — Real send-to-agent interaction (continued input)
status: todo
type: epic
priority: normal
created_at: 2026-06-18T10:07:49Z
updated_at: 2026-06-18T10:08:03Z
parent: qiosq-csrd
blocked_by:
    - qiosq-4ppz
---

Goal: 'really send the open file for further input of the coding agent' — the full round-trip in the interactive UI: inject @{path}, let the user type their request in the pane, run it, see the result, and continue the conversation (not a single fire-and-forget).

Depends on E8 (live session). May overlap; keep E8 = wiring the live session, E9 = the interaction/UX of sending + continuing.

## Tasks (draft)
- [ ] Cursor handoff into the live agent pane after intent injection.
- [ ] Support multiple turns (send again without restarting the session).
- [ ] Surface agent state (working / awaiting input / done) to the user.

## Tests
- [ ] Fake-agent multi-turn interaction test.
