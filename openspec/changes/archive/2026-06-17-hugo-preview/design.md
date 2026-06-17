## Context

`qtui-preview` (an E1 stub) owns the `hugo server` process for the opened site.
It is the first crate to manage an external process, so it gets integration tests
(not just unit tests) that invoke the real `hugo` binary — available in the flake
dev shell and the Nix check sandbox. Verified against hugo 0.162: the readiness
line is `Web Server is available at http://localhost:<port>/ (bind address
127.0.0.1)`, and the server is reachable within a couple of seconds even for a
bare site.

## Goals / Non-Goals

**Goals:**
- Pick a free port in `preview.port_range`, never `13131`.
- Start `hugo server` for a site, detect readiness, surface the URL.
- Guarantee the child is stopped on explicit stop and on drop; one server max.
- Integration test on a real fixture site: reachable + clean shutdown; unit
  tests for port selection.

**Non-Goals:**
- Rendering the site in the TUI (we surface a URL only — brief §6).
- The full app event loop / wiring preview-start into the binary (E6/E7 own the
  loop; E5 exposes the library and the `AppState::set_preview_url` hook already
  added in E4).
- Multiple concurrent servers, hot config reload, HTTPS.

## Decisions

- **Pick the port ourselves, then pass `--port`.** We probe with
  `TcpListener::bind((127.0.0.1, p))` across the range (skipping `13131`), take
  the first that binds, drop the listener, and hand that port to Hugo. There is a
  tiny TOCTOU window between dropping the probe listener and Hugo binding, but for
  a single-user kiosk on a dedicated range it is acceptable and keeps the URL
  deterministic. *Alternative considered:* let Hugo pick (`--port 0`) and parse
  the chosen port from its output — rejected; less control over the range and the
  `13131` exclusion, and harder to test collisions.
- **Readiness via stdout streaming on a thread.** Spawn `hugo server` with piped
  stdout, read lines on a worker thread, and signal readiness when the line
  matches `"Web Server is available at"`, extracting the URL (falling back to
  `http://localhost:<port>/`). The start call blocks on a channel up to
  `ready_timeout_ms`. *Alternative considered:* poll the port with HTTP — rejected
  as the spec/contract explicitly streams the readiness line, and Hugo may accept
  TCP before it is truly serving.
- **Lifecycle via a guard type.** `PreviewServer` holds the `Child`; `stop()` kills
  and reaps it, and `Drop` does the same best-effort so a dropped handle never
  orphans Hugo. "One at a time" is the caller's contract, reinforced by the type
  being a single owned handle (starting a new one drops/stops the old).
- **Config-driven.** Port range, hugo binary, and timeout come from
  `qtui-config` (E1). `qtui-preview` depends on `qtui-config`; it stays free of
  ratatui/rmux.

## Risks / Trade-offs

- **Test flakiness from real Hugo timing.** → Use a generous readiness timeout in
  tests, bind to `127.0.0.1`, render to memory, and assert on the readiness line
  + a TCP/HTTP connect rather than page content. Kill on teardown.
- **Sandbox networking.** The Nix check sandbox allows loopback; binding
  `127.0.0.1` works. If a specific check runner forbids it, the hugo-needing test
  is gated to skip when `hugo` is unavailable (it is available in our flake).
- **Zombie processes on panic.** → `Drop` reaps the child; tests assert the port
  frees after stop.

## Open Questions

- Whether to surface Hugo's stderr/warnings into a diagnostics channel like
  `qtui-model`'s warnings. Deferred — not needed for M1; the readiness line and a
  start error are enough for the UI.
