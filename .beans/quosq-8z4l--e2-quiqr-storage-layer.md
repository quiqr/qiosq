---
# quosq-8z4l
title: E2 — Quiqr storage layer
status: todo
type: epic
created_at: 2026-06-17T15:56:53Z
updated_at: 2026-06-17T15:56:53Z
parent: quosq-mer9
---

Goal: read-only knowledge of the Quiqr/Hugo storage.

## Tasks
- [ ] T2.1 Locate data dir from config; enumerate sites (has `config.*` + `quiqr/`).
- [ ] T2.2 Read-only `content/` tree with derived/generated dirs hidden.
- [ ] T2.3 Site fixtures for tests.

## Tests
- [ ] Enumeration + tree-filtering unit tests over tempdir fixtures.
