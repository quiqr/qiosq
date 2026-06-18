---
# qiosq-s0ui
title: E7 — Agent pane render + E2E proof
status: completed
type: epic
priority: normal
created_at: 2026-06-17T15:56:53Z
updated_at: 2026-06-18T00:39:07Z
parent: qiosq-mer9
---

Goal: see the agent work, and prove the whole flow in a VM.

## Tasks
- [x] T7.1 Render agent snapshots via `ratatui-rmux` in the right pane.
- [x] T7.2 NixOS VM test booting Quiqr Server (author's Quiqr module).
- [x] T7.3 Provision sample site + schema into the VM's Quiqr data dir.
- [x] T7.4 Headless/scripted `qtui` run exercising the full flow with fake agent.
- [x] T7.5 Assert on-disk result (+ optionally served output).

## Tests
- [x] `checks.<system>.e2e` passes in `nix flake check`.

## Summary of Changes

- Agent pane (qtui-ui): AppState::set_agent_output / agent_output; right pane renders the agent's latest snapshot lines, neutral label when empty. Fixed an E4 latent bug — content-tree rows now carry the real rel_path (ContentRow) so opening a file uses its true path and dirs aren't openable.
- Host event loop (qtui binary): host.rs wires config+storage+model+preview+agent; interactive.rs (crossterm raw-mode loop); script.rs (headless --script open-site,open-file,ask-ai,await). Agent in scripted mode = spawn agent.command, inject format_intent() on stdin, detect the completion sentinel. Stops the preview on exit (no orphan hugo).
- Flake inputs: rmux (github:mipmip/rmux, built from source -> bin/rmux + bin/rmux-daemon, on dev shell PATH) + nixpkgs-quiqr (github:mipmip/nixpkgs/quiqr-023, pinned). 
- checks.e2e: a NixOS VM imports quiqr-server.nix from the fork, enables the service (fs storage), provisions the anonymized real-site fixture into the dataFolder, writes a qtui config with the fake-agent as agent.command, runs `qtui --script`, and asserts the agent's new content file exists on disk.
- **PROVEN in the VM (nix flake check green):** quiqr-server boots -> multi-user.target; fixture provisioned; `qtui --script open-site,open-file,ask-ai,await` succeeds (1.44s); agent-wrote-this.md exists with 'written by fake-agent'. This is the alpha-base definition demonstrated against a real Quiqr Server.
- cargo test --workspace, clippy -D warnings, fmt, and full nix flake check (unit + e2e VM) all green.

Follow-ups left open: qiosq-rts9 (swap the source-built rmux for the official rmux flake when published — daemon wiring already works as interim). qiosq-zyu7 (VM recipe) is realized -> completed.

OpenSpec change agent-pane-e2e archived (specs agent-pane-render, host-event-loop, e2e-proof added; two-pane-layout modified).
