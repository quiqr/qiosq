---
# qiosq-rblj
title: 'qtui-storage: enumerate real Quiqr layout (sites/<name>/main/)'
status: completed
type: bug
priority: high
tags:
    - discovered
created_at: 2026-06-18T07:40:01Z
updated_at: 2026-06-18T07:50:42Z
---

## Problem
qtui-storage::enumerate_sites (E2) scans the IMMEDIATE subdirs of the data folder for a dir with `config.* + quiqr/`. But a real Quiqr data folder (Electron desktop + Server, confirmed on this machine at ~/Quiqr) is laid out differently:

```
~/Quiqr/
  sites/
    <name>/
      config.json            # site metadata
      main/                   # <- the actual Hugo/Quiqr working copy
        quiqr/model/...
        content/...
      main-*.jsonl            # workspace journals
  logs/  temp/  tools/
```

So: (a) sites live under a `sites/` subdir, not the data-dir root; (b) the real site root (with quiqr/model + content) is `sites/<name>/main/` — the per-workspace working copy (workspaceKey defaults to 'main', per quiqr-app-config.json). Our current enumeration finds ZERO sites against a real library.

## Why it matters
E2 passed its tests against hand-built/anonymized fixtures shaped as `<root>/{config.*,quiqr/,content/}` — which is NOT the real on-disk shape. The PoC e2e VM also provisioned the fixture flat, so it didn't catch this. Against an actual Quiqr install, nothing would list.

## Proposed fix (needs a spec/decision)
- enumerate_sites should understand the Quiqr layout: look under `<data>/sites/*/`, and treat `<data>/sites/<name>/main/` (the active workspace) as the site root. Possibly read `<data>/sites/<name>/config.json` for the workspace key instead of hardcoding 'main'.
- Keep tolerance: also accept the simpler flat shape (used by tests/older sites) OR migrate fixtures to the real shape.
- Confirm against more than one real site + the Server edition before locking the rule.

## Relationships
Blocks accurate site-count detection in [[qiosq-r7gf]] (initconf). Likely its own small OpenSpec change (modify the site-enumeration capability) or folded into the M2/next milestone.

Discovered 2026-06-18 while grounding qiosq-r7gf against the real ~/Quiqr and ~/.config/quiqr.

## Summary of Changes
Fixed qtui-storage enumeration to match the real Quiqr data-folder layout.
- enumerate_sites now reads `<data>/sites/*`; a site is an entry with a readable `config.json` descriptor; its working copy is resolved from `source.path` (tolerant SiteConfig via serde_json): `main` -> sites/<name>/main, `.`/`./`/empty/absent -> the site dir (flat site). Site.path is now the WORKING COPY (where quiqr/ + content/ live) — what content_tree, load_model, the preview cwd, and the host all want. Malformed config.json or no descriptor -> excluded. No sites/ dir -> empty list (not error). Added serde + serde_json deps.
- Reshaped the anonymized real-site fixture to sites/examplesite/{config.json (source.path=main), main/...}; updated qtui-storage unit + real_site tests (incl. a flat `./` site + a descriptor-less exclusion + a malformed-json exclusion), the qtui-model golden test path, and the e2e VM provisioning.
- Verified: full host `qtui --script` flow against the reshaped fixture writes content (working copy resolved correctly); cargo test --workspace, clippy -D warnings, fmt, and **nix flake check (incl. the VM e2e re-run on the real layout)** all green.

Unblocks qiosq-r7gf (accurate site-count detection).

Note: the previous E2 tests passed only because fixtures used a flatter hand-built shape that did not match a real Quiqr install; this closes that gap.
