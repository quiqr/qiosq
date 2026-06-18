---
# qiosq-s0ui
title: E7 — Agent pane render + E2E proof
status: in-progress
type: epic
priority: normal
created_at: 2026-06-17T15:56:53Z
updated_at: 2026-06-18T00:19:45Z
parent: qiosq-mer9
---

Goal: see the agent work, and prove the whole flow in a VM.

## Tasks
- [ ] T7.1 Render agent snapshots via `ratatui-rmux` in the right pane.
- [ ] T7.2 NixOS VM test booting Quiqr Server (author's Quiqr module).
- [ ] T7.3 Provision sample site + schema into the VM's Quiqr data dir.
- [ ] T7.4 Headless/scripted `qtui` run exercising the full flow with fake agent.
- [ ] T7.5 Assert on-disk result (+ optionally served output).

## Tests
- [ ] `checks.<system>.e2e` passes in `nix flake check`.
