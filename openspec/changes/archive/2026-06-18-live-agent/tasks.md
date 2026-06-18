## 1. Agent selection (host)

- [x] 1.1 `host::build_agent(&Config, &Site) -> Box<dyn Agent>`: fake-agent command basename -> `FakeAgent`; else `RmuxAgent::new("qtui-<site>")` (sanitised session name).
- [x] 1.2 `Host` holds `agent: Option<Box<dyn Agent>>`; add `ensure_agent_started(&Site)` that lazily builds + `start(site.path, "")` once and reuses it.

## 2. Live intent + streaming (interactive)

- [x] 2.1 On `Action::AskAi`: `ensure_agent_started`, then `send_intent(file, "")` to the live agent (replace the one-shot `run_agent_intent` call in the interactive loop only).
- [x] 2.2 Each render tick: if an agent session exists, push `agent.snapshot()` lines via `AppState::set_agent_output`; on error, show the error text in the pane (no crash).
- [x] 2.3 Track completion: `agent.is_complete(sentinel)` -> reflect a done/working status (pane label or legend); does not block the loop.

## 3. Keep the scripted/e2e path

- [x] 3.1 `script.rs` still calls `run_agent_intent` (subprocess fake-agent) — unchanged. The e2e VM + `nix flake check` need no rmux daemon.

## 4. Tests

- [x] 4.1 Unit (host): `build_agent` selects FakeAgent for a fake-agent command and RmuxAgent otherwise (assert via a cheap observable, e.g. it doesn't connect on construction).
- [x] 4.2 Loop/host test via in-process FakeAgent: Ask AI starts the agent once + reuses it; snapshot lines reach `set_agent_output`.
- [x] 4.3 Real-rmux start+send_intent+snapshot is covered by the existing `#[ignore]`d `qtui-agent::rmux::tests::live_start_and_send` (the host's live path delegates to `RmuxAgent`); no separate host-level daemon test added.

## 5. Gate

- [x] 5.1 `cargo test --workspace` green; `cargo clippy --workspace --all-targets -- -D warnings` clean.
- [x] 5.2 `cargo fmt --all --check` clean.
- [x] 5.3 `nix flake check` green (incl. the VM e2e — still the fake path).

## 6. Done

- [x] 6.1 OpenSpec change `live-agent` validates.
- [x] 6.2 CHANGELOG Unreleased entry; `qiosq-4ppz` closed; commit references it; change archived.
