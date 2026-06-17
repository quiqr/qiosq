---
# quosq-77q5
title: E5 — Site open + Hugo preview
status: todo
type: epic
created_at: 2026-06-17T15:56:53Z
updated_at: 2026-06-17T15:56:53Z
parent: quosq-mer9
---

Goal: opening a site serves it and surfaces the URL.

## Tasks
- [ ] T5.1 `qtui-preview`: start `hugo server` on a free port (avoid `:13131`).
- [ ] T5.2 Detect readiness, surface URL in the view; stop on close/exit.
- [ ] T5.3 Wire site-open in the UI to preview start.

## Tests
- [ ] Integration test: start hugo on a fixture site, assert reachable + clean shutdown + port-collision handling.
