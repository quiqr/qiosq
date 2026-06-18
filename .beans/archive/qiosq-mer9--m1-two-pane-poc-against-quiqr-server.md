---
# qiosq-mer9
title: M1 — Two-pane PoC against Quiqr Server
status: completed
type: milestone
priority: high
created_at: 2026-06-17T15:56:29Z
updated_at: 2026-06-18T00:39:20Z
---

The alpha base. Complete when `nix flake check` passes including the VM e2e test that demonstrates the full flow (open site → hugo serving → open file read-only → send to agent → agent writes content → change visible) against a real Quiqr Server instance.

Groups epics E1–E7. Source: docs/epics-seed.md.

## Summary — M1 COMPLETE
All seven epics done (E1 foundation, E2 storage, E3 model, E4 TUI shell, E5 preview, E6 agent bridge, E7 agent pane + e2e). `nix flake check` is green INCLUDING the NixOS VM e2e test that boots Quiqr Server (from mipmip/nixpkgs/quiqr-023), provisions a site, runs `qtui --script` with the fake agent, and asserts the agent's new content file on disk. The two-pane PoC / alpha base is proven end to end.
Open follow-up only: qiosq-rts9 (swap source-built rmux for the official rmux flake when published; interim already works).
