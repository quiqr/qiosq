---
# qiosq-eyzw
title: E10 — Show the active pane (focus)
status: todo
type: epic
created_at: 2026-06-18T10:07:49Z
updated_at: 2026-06-18T10:07:49Z
parent: qiosq-csrd
---

Goal: 'show active pane' — the user can always tell whether focus is on the left (browser) or right (agent) pane, with a clear visual indicator (border/title highlight), and the legend reflects it.

## Tasks (draft)
- [ ] Track focus in AppState (which pane has the cursor).
- [ ] Render a focus indicator (highlighted border/title) on the active pane.
- [ ] Key to switch focus; legend reflects focus.

## Tests
- [ ] TestBackend: active-pane indicator renders on the focused side; toggles.
