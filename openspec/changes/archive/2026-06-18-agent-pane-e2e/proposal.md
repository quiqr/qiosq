## Why

This is the proof. The PoC's whole value is demonstrating the two-pane experience
end to end against a real Quiqr Server: list a site, serve it with Hugo, open a
file read-only, "Ask AI" to inject the intent, the agent writes content, and the
change is visible on disk. E1–E6 built every piece behind clean seams; E7 renders
the agent's output in the right pane, wires the `qtui` binary into a runnable host
event loop, and adds the NixOS VM end-to-end test that boots Quiqr Server and runs
the full flow with the fake agent.

## What Changes

- Render the agent pane: replace the right-pane placeholder with the agent's live
  output (a `PaneSnapshot`/`ratatui-rmux` view) refreshed each tick.
- Wire the `qtui` binary into a real host event loop: load config, enumerate
  sites (`qtui-storage`), build the navigation model (`qtui-model`), run the
  ratatui terminal driving `qtui-ui`'s `update`/`render`, start/stop the Hugo
  preview (`qtui-preview`) on site open/close, and drive the agent
  (`qtui-agent`). This is the first runnable interactive TUI.
- Add a headless/scripted mode (a `--script` flag) so the e2e test can exercise
  the flow deterministically without a live terminal.
- Provide the rmux daemon to the dev shell and the Nix check sandbox (built from
  the `rmux` source until the official flake lands) so the real agent path runs.
- Fill `checks.e2e`: boot a VM running **Quiqr Server** (the NixOS module from the
  `nixpkgs-quiqr` fork), provision a serveable sample site, run `qtui --script`
  with the **fake agent**, and assert the new content file exists on disk.

## Capabilities

### New Capabilities
- `agent-pane-render`: render the coding agent's current output in the right pane,
  refreshed from snapshots, never attaching the user to the raw session.
- `host-event-loop`: the `qtui` binary's runnable loop that wires config, storage,
  model, preview, and agent together behind the `qtui-ui` state machine, plus a
  headless `--script` mode for tests.
- `e2e-proof`: a NixOS VM test that boots Quiqr Server, provisions a site, and
  drives `qtui` through the full flow with the fake agent, asserting the on-disk
  result.

### Modified Capabilities
- `two-pane-layout`: the right pane changes from a static placeholder to the
  rendered agent output. (The two-pane shape and the no-write rule are unchanged.)

## Impact

- `crates/qtui-ui`: right pane renders supplied agent output (host pushes the
  latest snapshot into `AppState`); still no terminal I/O in the library.
- `crates/qtui` (binary): gains the real event loop + `--script` headless mode;
  depends on all service crates (already does) and now constructs them.
- `flake.nix`: new inputs `rmux` (interim: build the `rmux` binary from
  `github:mipmip/rmux`) and `nixpkgs-quiqr` (`github:mipmip/nixpkgs/quiqr-023`);
  `rmux` on `PATH`; `checks.e2e` imports `quiqr-server.nix` and runs the scenario.
- Resolves the long-standing seams: the e2e VM (`qiosq-zyu7`) and the rmux daemon
  availability (`qiosq-rts9`, interim via source build until the official flake).
- The single-writer rule holds: only the agent writes content; the UI and the
  test harness assert on disk but never write site content themselves.
