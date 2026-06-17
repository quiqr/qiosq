## 1. Crate setup

- [x] 1.1 `qtui-preview` deps: `qtui-config` (port range, hugo bin, timeout) + `thiserror`; keep it free of ratatui/rmux.
- [x] 1.2 Define `PreviewError` (no free port; spawn failure; readiness timeout) and a `PreviewServer` handle type exposing `url()`.

## 2. Port selection

- [x] 2.1 `pick_port(range)`: scan the inclusive range, skip `13131`, probe each with `TcpListener::bind(127.0.0.1:p)`, return the first free one.
- [x] 2.2 Error naming the range when none is free.

## 3. Lifecycle + readiness

- [x] 3.1 `PreviewServer::start(config, site_dir)`: pick a port, spawn `hugo server` (cwd=site_dir, --bind 127.0.0.1, --port, --renderToMemory, --baseURL), pipe stdout.
- [x] 3.2 Stream stdout on a thread; signal readiness on the "Web Server is available at" line and parse the URL (fallback http://localhost:<port>/); block up to ready_timeout_ms then error + kill.
- [x] 3.3 `stop(self)` kills + reaps the child; `Drop` does the same best-effort (no orphan). One server at a time (owning handle).

## 4. UI surfacing

- [x] 4.1 Confirm `qtui-ui` Browse view surfaces `preview_url` (set via `AppState::set_preview_url`); add/extend a test that the URL appears in the Browse render. Keep qtui-preview OUT of qtui-ui's deps (host wires them).

## 5. Tests

- [x] 5.1 Unit: `pick_port` returns in-range + free; skips an occupied low port; skips 13131; errors when range exhausted.
- [x] 5.2 Integration: minimal Hugo site fixture in a tempdir; start preview; assert URL surfaced + reachable (TCP/HTTP connect); stop; assert process gone + port re-bindable. Gate on `hugo` being present.
- [x] 5.3 Readiness timeout path produces an error and no orphan (e.g. impossibly short timeout).

## 6. Gate

- [x] 6.1 `cargo test --workspace` green; `cargo clippy --workspace --all-targets -- -D warnings` clean (dev shell).
- [x] 6.2 `cargo fmt --all --check` clean.
- [x] 6.3 `nix flake check` green (hugo available in the sandbox).

## 7. Done

- [x] 7.1 OpenSpec change `hugo-preview` validates.
- [x] 7.2 E5 beans closed; commit references `qiosq-77q5`; change archived.
