## Why

The product promise is "open a site and immediately see it." A pure SSH terminal
cannot render a themed Hugo site, so the decision (project brief §6) is to start
`hugo server` for the opened site and surface its URL — the user opens that URL
in their own browser, and because Hugo is watching, the agent's writes
auto-refresh. E4 gave the shell a `preview_url` slot and a Preview legend entry;
E5 implements `qtui-preview` to actually run and manage the server.

## What Changes

- Implement `qtui-preview` (currently an empty stub):
  - Pick a free TCP port inside the configured `preview.port_range`, **never
    `13131`** (Quiqr's default), probing with a `TcpListener`.
  - Start `hugo server` for the opened site (workdir pinned to the site, bound to
    `127.0.0.1` on the chosen port), streaming its stdout.
  - Detect readiness from Hugo's "Web Server is available at …" line, parse the
    URL, and surface it; time out with a clear error if readiness is not reached
    within the configured `ready_timeout_ms`.
  - Stop the server on close/exit (explicit stop and on drop). One server at a
    time for the PoC.
- Surface the URL in the `Browse` view (the field already exists on `AppState`).
- Integration test that starts Hugo on a minimal site fixture, asserts the URL is
  surfaced and reachable, then asserts clean shutdown; unit tests for port
  selection including the `13131` exclusion and collisions.

## Capabilities

### New Capabilities
- `preview-port-selection`: choose a free port inside the configured range,
  excluding Quiqr's reserved `13131`, and fail clearly when none is free.
- `preview-lifecycle`: start `hugo server` for a site and stop it on close/exit,
  guaranteeing the child process is terminated; at most one server at a time.
- `preview-readiness`: detect the server's readiness from its output, surface the
  URL, and time out with a clear error if it never becomes ready.

### Modified Capabilities
<!-- None — qtui-preview was an empty stub from E1; no existing preview spec. -->

## Impact

- `crates/qtui-preview`: real implementation owning a child process; integration
  tests that invoke the real `hugo` binary (provided by the flake dev shell and
  the Nix check sandbox).
- `qtui-config` is consumed for the port range, hugo binary, and timeout
  (`qtui-preview` depends on `qtui-config`).
- The library never writes site files; it only spawns/stops Hugo and reads its
  stdout. The `qtui` binary wires preview-start into site-open in a later step
  (full event loop is E6/E7); E5 is the managed-process library + its tests.
