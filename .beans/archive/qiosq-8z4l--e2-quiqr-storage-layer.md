---
# qiosq-8z4l
title: E2 — Quiqr storage layer
status: completed
type: epic
priority: normal
created_at: 2026-06-17T15:56:53Z
updated_at: 2026-06-17T21:09:53Z
parent: qiosq-mer9
---

Goal: read-only knowledge of the Quiqr/Hugo storage.

## Tasks
- [x] T2.1 Locate data dir from config; enumerate sites (has `config.*` + `quiqr/`).
- [x] T2.2 Read-only `content/` tree with derived/generated dirs hidden.
- [x] T2.3 Site fixtures for tests.

## Tests
- [x] Enumeration + tree-filtering unit tests over tempdir fixtures.

## Summary of Changes

- qtui-storage implemented (was an E1 stub). Pure crate; thiserror dep + tempfile dev-dep (added to workspace).
- Site enumeration: enumerate_sites(data_dir) lists immediate subdirs that are Quiqr sites, sorted by name; is_quiqr_site requires BOTH a Hugo config (config.*/hugo.* file or config/ dir) AND a quiqr/ dir. Empty list when none; StorageError::DataDir (names the path) when missing/unreadable.
- Content tree: content_tree(site, hidden_dirs) builds a read-only ContentNode tree of content/, dirs-first then alphabetical, hiding configured dirs (public/resources/.quiqr-cache/.git/themes by default) at every depth; empty when no content/; does not follow symlinks.
- 10 unit tests over tempfile fixtures incl. an explicit never-writes snapshot assertion. cargo test --workspace (22 tests), clippy -D warnings, fmt, and nix flake check all green.

OpenSpec change quiqr-storage archived (specs site-enumeration, content-tree promoted to openspec/specs/).
