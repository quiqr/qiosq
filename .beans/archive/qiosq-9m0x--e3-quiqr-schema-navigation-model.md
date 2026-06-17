---
# qiosq-9m0x
title: E3 — Quiqr schema → navigation model
status: completed
type: epic
priority: normal
created_at: 2026-06-17T15:56:53Z
updated_at: 2026-06-17T22:04:33Z
parent: qiosq-mer9
---

Goal: turn the schema into the WP-style Menu and into agent constraints.

## Tasks
- [x] T3.1 Parse `quiqr/model/base.yaml`.
- [x] T3.2 Merge `quiqr/model/includes/*` (collections, menu, singles, dynamics).
- [x] T3.3 Build `NavigationModel` (Menu → Singles/Collections → paths).
- [x] T3.4 Expose per-collection/single field schemas for prompt constraints.
- [x] T3.5 Tolerant handling of partial/legacy/malformed schemas.

## Tests
- [x] Golden-file tests incl. partial/malformed cases.

## Summary of Changes

- qtui-model implemented (was an E1 stub). Pure crate; serde + serde_yaml deps (serde_yaml added to workspace). Grounded in real Quiqr model shape (read actual sites under /home/pim).
- load_model(site_dir) reads quiqr/model/base.yaml + includes/; each include root (menu/singles/collections/dynamics) may be a <root>.yaml file OR a <root>/ dir (files merged in name order).
- NavigationModel: ordered MenuGroups -> MenuEntry::{Single,Collection} resolved by key (unknown keys dropped with a warning); Singles map to their file, Collections to their folder; titles fall back to key.
- Field schemas: recursive Field {key, title->key, type_->'string' default, fields}; nested fields (e.g. bundle-manager -> thumb) preserved; entities may have empty fields (e.g. _mergePartial singles).
- Tolerant: serde_yaml::Value projection ignores unknown keys; a malformed include is skipped and recorded in NavigationModel.warnings (never panics, never errors on content); _mergePartial recorded, never fetched (no network); absent/empty model -> empty NavigationModel.
- 5 golden-file tests over fixtures (full, partial, malformed, dir-include, missing). cargo test --workspace (28 tests), clippy -D warnings, fmt, and nix flake check all green.

Deferred: resolving _mergePartial and surfacing dynamics in the menu (no consumer in M1's UI scope).

OpenSpec change quiqr-model archived (specs schema-parsing, navigation-model, field-schemas promoted to openspec/specs/).
