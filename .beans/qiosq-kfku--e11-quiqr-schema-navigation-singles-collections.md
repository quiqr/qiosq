---
# qiosq-kfku
title: E11 — Quiqr schema navigation (singles + collections)
status: todo
type: epic
created_at: 2026-06-18T10:07:49Z
updated_at: 2026-06-18T10:07:49Z
parent: qiosq-csrd
---

Goal: 'Quiqr navigation should also work — start with singles but also implement collection listing.' Today only the content-tree view opens files; the schema menu just toggles.

Real schema (from qtui-model): singles map to a file (single.file); collections map to a folder (collection.folder) and need a LISTING of the items within (the content files under that folder).

## Tasks (draft)
- [ ] Opening a Single in the schema menu -> open its file (single.file) read-only.
- [ ] Opening a Collection -> list its items (files under collection.folder); selecting an item opens it.
- [ ] Resolve schema paths (single.file is often absolute-from-site-root like /content/_index.md) to the working-copy file the viewer/host reads.
- [ ] Handle entities with no file / _mergePartial gracefully.

## Tests
- [ ] qtui-ui transition + render tests over a fixture model: single opens its file; collection lists items; item opens.
