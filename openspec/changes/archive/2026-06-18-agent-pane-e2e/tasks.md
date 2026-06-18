## 1. Flake inputs (rmux + quiqr)

- [x] 1.1 Add `inputs.rmux.url = "github:mipmip/rmux"` (flake = false) + a `buildRustPackage` derivation building the `rmux` binary; add to dev shell packages. (Interim until the official rmux flake; tracks qiosq-rts9.)
- [x] 1.2 Add `inputs.nixpkgs-quiqr.url = "github:mipmip/nixpkgs/quiqr-023"`; expose `pkgs-quiqr` + the quiqr-server module path for the VM.

## 2. Agent pane render (qtui-ui)

- [x] 2.1 `AppState::set_agent_output(Vec<String>)`; right pane renders those lines, or a neutral label when empty.
- [x] 2.2 TestBackend test: pushed agent lines appear in the right pane; empty -> label.

## 3. Host event loop (qtui binary)

- [x] 3.1 Interactive loop: load config, enumerate sites, build model; crossterm raw-mode terminal driving qtui_ui update/render; open site -> PreviewServer::start + set_open_site; open file -> read bytes + set_open_file; Ask AI -> agent.send_intent + push snapshots; exit stops the preview.
- [x] 3.2 Build the agent from config (`agent.command`): FakeAgent when it points at the fake-agent binary, else RmuxAgent.
- [x] 3.3 Headless `--script <steps>` mode: same transitions without a TTY; exit code reflects success.

## 4. VM e2e (flake checks.e2e)

- [x] 4.1 VM node imports `${nixpkgs-quiqr}/nixos/modules/services/web-apps/quiqr-server.nix`; enable service, fs storage with a known dataFolder; hugo available.
- [x] 4.2 Provision a serveable sample site (schema with >=1 Single + >=1 Collection + content) into the dataFolder; write a qtui config pointing at it with the fake-agent as `agent.command`.
- [x] 4.3 testScript: wait_for the quiqr-server unit; run `qtui --script` exercising list -> preview reachable -> menu has Single+Collection -> open file read-only -> Ask AI -> fake agent writes file + sentinel.
- [x] 4.4 Assert the new content file exists on disk; check passes.

## 5. Gate

- [x] 5.1 `cargo test --workspace` green; `cargo clippy --workspace --all-targets -- -D warnings` clean.
- [x] 5.2 `cargo fmt --all --check` clean.
- [x] 5.3 `nix flake check` green INCLUDING checks.e2e (the full VM scenario).

## 6. Done

- [x] 6.1 OpenSpec change `agent-pane-e2e` validates.
- [x] 6.2 E7 bean closed; qiosq-zyu7 closed; qiosq-rts9 updated (interim daemon wired); commit references `qiosq-s0ui`; change archived. M1 milestone complete.
