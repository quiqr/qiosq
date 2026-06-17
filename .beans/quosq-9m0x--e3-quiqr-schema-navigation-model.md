---
# quosq-9m0x
title: E3 — Quiqr schema → navigation model
status: todo
type: epic
created_at: 2026-06-17T15:56:53Z
updated_at: 2026-06-17T15:56:53Z
parent: quosq-mer9
---

Goal: turn the schema into the WP-style Menu and into agent constraints.

## Tasks
- [ ] T3.1 Parse `quiqr/model/base.yaml`.
- [ ] T3.2 Merge `quiqr/model/includes/*` (collections, menu, singles, dynamics).
- [ ] T3.3 Build `NavigationModel` (Menu → Singles/Collections → paths).
- [ ] T3.4 Expose per-collection/single field schemas for prompt constraints.
- [ ] T3.5 Tolerant handling of partial/legacy/malformed schemas.

## Tests
- [ ] Golden-file tests incl. partial/malformed cases.
