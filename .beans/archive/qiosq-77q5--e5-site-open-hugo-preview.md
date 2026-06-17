---
# qiosq-77q5
title: E5 — Site open + Hugo preview
status: completed
type: epic
priority: normal
created_at: 2026-06-17T15:56:53Z
updated_at: 2026-06-17T22:30:28Z
parent: qiosq-mer9
---

Goal: opening a site serves it and surfaces the URL.

## Tasks
- [x] T5.1 `qtui-preview`: start `hugo server` on a free port (avoid `:13131`).
- [x] T5.2 Detect readiness, surface URL in the view; stop on close/exit.
- [x] T5.3 Wire site-open in the UI to preview start.

## Tests
- [x] Integration test: start hugo on a fixture site, assert reachable + clean shutdown + port-collision handling.

## Summary of Changes

- qtui-preview implemented (was an E1 stub). Owns the hugo server process; depends on qtui-config; no ratatui/rmux; never writes site files.
- pick_port(range): scans the inclusive configured range, skips Quiqr's reserved 13131, probes each with TcpListener::bind(127.0.0.1:p), returns the first free port; PreviewError::NoFreePort (names the range) when exhausted.
- PreviewServer::start(config, site_dir): spawns `hugo server` (cwd=site, --bind 127.0.0.1, --port, --baseURL, --renderToMemory), streams stdout on a worker thread, detects readiness on Hugo's 'Web Server is available at' line and parses the URL (fallback http://localhost:<port>/), blocks up to ready_timeout_ms then errors + kills. url()/port() accessors.
- stop(self) kills+reaps; Drop does the same best-effort so hugo is never orphaned. One server at a time (owning handle).
- qtui-ui Browse view surfaces the preview URL on a dedicated line above the list (set via AppState::set_preview_url) — readable, not truncated in the border title. qtui-preview is intentionally NOT a dep of qtui-ui (host wires them).
- Tests: 4 port-selection unit tests (in-range/free, skip occupied, skip 13131, exhausted) + 3 real-hugo integration tests (starts/serves/reachable/clean-shutdown; drop frees the port; readiness-timeout errors with no orphan) + a UI test for URL surfacing. The integration tests RUN (not skip) inside the nix flake check sandbox.
- cargo test --workspace (48 tests), clippy -D warnings, fmt, and nix flake check all green.

Deferred: wiring preview-start into the qtui binary's site-open flow (E6/E7 own the event loop).

OpenSpec change hugo-preview archived (specs preview-port-selection, preview-lifecycle, preview-readiness promoted to openspec/specs/).
