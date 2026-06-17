## 1. Crate setup

- [x] 1.1 `qtui-agent` deps: `rmux-sdk` (pinned 0.6) + tokio (for the real impl) + `thiserror`; trait + fake stay runtime-free.
- [x] 1.2 Define `Agent` trait (`start`, `send_intent`, `poll_output`/`await_completion`, `snapshot`) and a `PaneSnapshot` type.

## 2. Pure bridge logic

- [x] 2.1 `format_intent(path) -> "@{path} I want to do the following… "` (pure, unit-tested).
- [x] 2.2 `contains_sentinel(output, sentinel) -> bool` completion predicate (pure, unit-tested).

## 3. Fake agent (test double)

- [x] 3.1 In-process `FakeAgent: Agent` — records sent text, exposes a scripted output buffer + snapshot.
- [x] 3.2 `tests/bin/fake-agent` scripted CLI: read stdin, print the sentinel, write a known file (for E7 e2e + the testing-strategy requirement).

## 4. Real rmux agent (compiled; live path manual)

- [x] 4.1 `RmuxAgent: Agent` over rmux-sdk: detached session, pane (0,0), workdir pinned; send_text intent; wait_for_text sentinel; snapshot. Blocking facade over the async SDK.
- [x] 4.2 Gate the live (daemon-needing) path behind a manual/`#[ignore]`d test or example; document the `qiosq-rts9` constraint in code.

## 5. Read-only viewer (qtui-ui)

- [x] 5.1 `AppState` carries `open_file_contents`; `set_open_file(path, contents)` host hook; `render` shows them read-only in `ViewFile`.
- [x] 5.2 Confirm (test) the viewer renders supplied contents and still exposes no edit verb; Ask AI emits the intent request for the open file.

## 6. Bridge integration tests (fake agent)

- [x] 6.1 Through the `Agent` trait with `FakeAgent`: `send_intent` injects the `@{path} …` prefix; `await_completion` resolves on the sentinel; `snapshot` returns expected text.
- [x] 6.2 Drive the `tests/bin/fake-agent` binary: feed stdin, assert it prints the sentinel and wrote its file.
- [x] 6.3 Assert sending an intent writes no site files.

## 7. Gate

- [x] 7.1 `cargo test --workspace` green; `cargo clippy --workspace --all-targets -- -D warnings` clean (dev shell).
- [x] 7.2 `cargo fmt --all --check` clean.
- [x] 7.3 `nix flake check` green (no live rmux daemon needed).

## 8. Done

- [x] 8.1 OpenSpec change `agent-bridge` validates.
- [x] 8.2 E6 beans closed; `qiosq-rts9` left open (blocked: real daemon in Nix); commit references `qiosq-me0f`; change archived.
