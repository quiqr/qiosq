---
# qiosq-rts9
title: rmux daemon not available in Nix/CI sandbox
status: todo
type: task
priority: normal
tags:
    - discovered
    - blocked
created_at: 2026-06-17T22:40:02Z
updated_at: 2026-06-17T22:40:02Z
parent: qiosq-me0f
---

## What
rmux-sdk 0.6.1 (and ratatui-rmux 0.6.1) ARE on crates.io and build fine (pulls tokio; async). BUT rmux-sdk is **daemon-backed**: `Rmux::builder().connect_or_start()` needs a running rmux daemon (from github.com/helvesec/rmux), which is NOT on crates.io and NOT in nixpkgs, and no rmux/rmuxd is on PATH.

## Impact
The real Claude-Code-over-rmux Agent impl can be written against rmux-sdk, but a bridge integration test that drives anything THROUGH a live rmux daemon cannot run in `nix flake check` (deterministic, offline, no daemon).

## Decision (E6)
Put all rmux/daemon code behind the `Agent` trait. Test the bridge LOGIC (intent formatting `@{path} I want to do the following… `, completion-sentinel detection, read-only viewer, mode wiring) against the fake-agent THROUGH the trait — no daemon needed, CI stays green and offline. Keep the real rmux path as a manual/integration seam (a feature-gated or `#[ignore]`d test, or an example) exercised by hand.

## To unblock later (E7 or a daemon epic)
Package the rmux daemon as a Nix flake input or derivation (build from helvesec/rmux), add it to the dev shell + check sandbox, then add a real end-to-end rmux test. Tracks toward E7's VM e2e. Tried: crates.io resolve+build (OK); `command -v rmux/rmuxd` (absent); `nix eval nixpkgs#rmux` (absent).
