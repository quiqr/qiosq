---
# qiosq-9c44
title: Anonymized real-site test fixture (notnix)
status: completed
type: task
priority: normal
tags:
    - discovered
created_at: 2026-06-17T23:13:09Z
updated_at: 2026-06-17T23:13:09Z
parent: qiosq-s0ui
---

## What
Added crates/qtui-storage/tests/fixtures/real-site: an anonymized copy of a real Quiqr/Hugo site (derived from the public notnix-com.github.io). Real quiqr/model/ schema (menu/singles/collections + partials), real-shaped content/ tree (8 files); theme + generated output omitted (storage/model layers don't need them). All author names/domains/prose replaced with generic placeholders; identifier sweep clean.

## Why
Validates qtui-storage (enumeration + content tree) and qtui-model (tolerant schema parse) against a genuine on-disk shape, not just hand-built minimal fixtures. Notably the singles defer fields via _mergePartial (empty fields + recorded ref) — exercises the tolerant path on real data.

## Tests added
- qtui-storage/tests/real_site.rs: is_quiqr_site + content_tree over the fixture.
- qtui-model/tests/golden.rs::parses_the_anonymized_real_site: collections pages/posts (+folders+fields), mainConfig single (empty fields + partial), 4 menu groups in order.
All green incl. nix flake check.

## Summary of Changes
Fixture + 3 tests committed. Feeds E7's VM provisioning step (a serveable variant with a theme will be provisioned into the VM separately).
