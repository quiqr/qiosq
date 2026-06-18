## Context

E7 is the final M1 epic — the proof. E1–E6 built the pieces behind seams: config,
storage, model, the UI state machine (with a placeholder right pane and a
`set_open_file`/snapshot-push surface), the Hugo preview lifecycle, and the agent
bridge (`Agent` trait + `FakeAgent` + a compiling `RmuxAgent`). E7 connects them:
render the agent pane, run the `qtui` binary as a real app, and add the VM e2e
test.

**Inputs now available:**
- rmux source: `github:mipmip/rmux` (main). It has no flake yet, but the repo's
  `rmux` binary is both the CLI and the hidden daemon `rmux-sdk` spawns. **Verified:
  it builds cleanly under Nix via `rustPlatform.buildRustPackage` with
  `cargoLock.lockFile`, producing `bin/rmux` + `bin/rmux-daemon`** (~6 min, no
  extra native deps). So we build it from source as an interim flake input and put
  it on `PATH`; swap to the official flake when published (`qiosq-rts9`).
- Quiqr: `github:mipmip/nixpkgs/quiqr-023` (branch). `quiqr-server.nix` and the
  `quiqr.server` package are confirmed present; importable into a `nixosTest`
  (`qiosq-zyu7`). Pin a rev on request.

## Goals / Non-Goals

**Goals:**
- Render the agent's snapshot output in the right pane.
- A runnable `qtui` host loop + a headless `--script` mode for the e2e test.
- `checks.e2e`: boot Quiqr Server, provision a site, run `qtui --script` with the
  fake agent, assert the new content file on disk.

**Non-Goals:**
- A real LLM in CI — the e2e uses the **fake agent** (testing strategy).
- SSH/kiosk hardening, the wrapper app, multi-user auth, publish pipeline (M2+).
- Constraining agent output from field schemas (the seam exists; not needed for
  the proof).

## Decisions

- **rmux daemon via a source-built flake input (interim).** Add
  `inputs.rmux.url = "github:mipmip/rmux"` with `flake = false`, and a
  `buildRustPackage` derivation in our flake that builds it; add to
  `devShells.default.packages` and to the e2e VM's `systemPackages`. When the
  official rmux flake lands, replace this derivation with the flake's package
  output. *Alternative considered:* wait for the official flake — rejected; the
  source build works today and keeps E7 moving, with a clean swap later.
- **Agent pane render: push snapshots into `AppState`.** Add
  `AppState::set_agent_output(lines)`; the right pane renders those lines (or a
  label when empty). The library still does no I/O; the host calls
  `agent.snapshot()` each tick and pushes the lines in. This keeps the render path
  identical for `FakeAgent` and `RmuxAgent`.
- **Host loop in the `qtui` binary.** Interactive mode: a crossterm raw-mode
  terminal loop — read key events, `qtui_ui::update`, on transitions call the
  services (open site → `enumerate`/`load_model`/`PreviewServer::start`; open file
  → read bytes + `set_open_file`; Ask AI → `agent.send_intent` + poll snapshots),
  then `qtui_ui::render`. Headless `--script` mode: the same transitions driven by
  a step list instead of key events, no raw terminal — this is what the e2e runs.
  *Alternative considered:* a separate test-only driver crate — rejected; a
  `--script` flag on the real binary is what the testing strategy asks for and
  exercises the real wiring.
- **Agent choice is config-driven.** The host builds `FakeAgent` vs `RmuxAgent`
  from `agent.command` (the e2e sets it to the `fake-agent` binary). The on-disk
  write in the e2e comes from the `fake-agent` CLI writing a content file.
- **e2e VM stays minimal.** Import `quiqr-server.nix`, enable the service with an
  `fs` data dir, provision the serveable fixture, run `qtui --script`, assert the
  file. No nginx/auth/restic.

## Risks / Trade-offs

- **rmux source build cost (~6 min) in `nix flake check`.** → Acceptable; it
  caches. The official flake will likely be faster/cached upstream later.
- **VM test weight + Quiqr Server boot time.** → Generous `wait_for_unit`; keep
  the provisioned site tiny (the serveable fixture).
- **`nixpkgs-quiqr` is a moving branch.** → Pin a rev in `flake.lock`; update only
  when asked. Document the branch in the input comment.
- **rmux daemon in a headless VM/CI.** → The e2e uses the **fake agent**, so the
  VM scenario does not require the rmux daemon at all; the daemon is only for the
  interactive `RmuxAgent` path, covered by the dev shell + a manual/ignored test.

## Open Questions

- Exact `--script` step format (a small enum sequence vs a tiny file). Chosen
  pragmatically in code (a comma/line-separated step list); not spec-level.
- Whether to render via `ratatui-rmux`'s widget or our own `PaneSnapshot` lines.
  Starting with our own lines (decoupled, testable); `ratatui-rmux` can replace
  the renderer later without a spec change.
