# 02 — Testing Strategy

Testing is not optional and not an afterthought. **No epic is "done" until its
tests are green in `nix flake check`.** The PoC's whole value is *proof*, so the
e2e layer matters as much as the code.

## Layers

### 1. Unit tests (fast, pure)
- `qtui-model`: parse + merge of `base.yaml` and `includes/*`. Use **golden
  files**: small example schemas in `crates/qtui-model/tests/fixtures/`, assert
  the produced `NavigationModel`. Cover partial/legacy/malformed schemas
  (tolerant parsing).
- `qtui-config`: valid config loads; each invalid field produces a clear error.
- `qtui-storage`: site enumeration + content-tree filtering over tempdir
  fixtures shaped like Quiqr sites (with `public/`, `.git`, `themes/` present and
  asserted hidden).

### 2. UI tests (ratatui TestBackend)
- Render each mode (`SiteList`, `Browse`, `ViewFile`, `Agent`) to a
  `TestBackend` buffer and snapshot-assert the layout + the **function-key
  legend** for that mode.
- Assert the read-only viewer offers "Ask AI" and never exposes an edit verb.
- Assert mode transitions on key events (state-machine tests).

### 3. Integration tests (own external processes)
- `qtui-preview`: start `hugo server` on a real (minimal) Hugo site fixture,
  assert it becomes reachable on the chosen port and that the URL is surfaced,
  then assert clean shutdown. Port-collision handling (skip `:13131`).
- `qtui-agent` with a **fake agent**: ship a tiny scripted CLI (e.g.
  `tests/bin/fake-agent`) that reads stdin and echoes a known **completion
  sentinel**. Drive it through rmux-sdk and assert:
  - `send_intent` injects `@{path} …` correctly,
  - `await_completion` resolves on the sentinel,
  - `snapshot()` returns the expected pane text.
  This proves the bridge without depending on a real LLM or network.

### 4. End-to-end (NixOS VM test — the proof)
The flake exposes a `nixosTest` (`checks.<system>.e2e`) that:

1. Boots a VM that runs **Quiqr Server** via the project's Quiqr Nix module
   (`TODO(author)` wire-in).
2. Provisions a sample Quiqr site (fixture) into the Quiqr data dir, including a
   real `quiqr/model/` schema with at least one Single and one Collection.
3. Runs `qtui` in **headless/scripted mode** (a `--script` or test harness flag
   the agent adds) against that server with the **fake agent** as the configured
   coding agent, exercising the full flow:
   - list sites → site visible,
   - open site → `hugo server` reachable, URL surfaced,
   - navigate schema Menu → Single + Collection present,
   - open a content file → read-only,
   - "Ask AI" → intent injected → fake agent writes a new content file with the
     sentinel,
   - assert the new file exists on disk and (optionally) appears in the served
     site.
4. Asserts exit code 0 and expected on-disk state.

Notes:
- Use the fake agent in CI (deterministic, offline). A real Claude Code run is a
  manual/optional check, not part of `flake check` (no network/keys in the
  sandbox).
- Keep the VM minimal; reuse the author's Quiqr module so the server matches
  production behavior.

## Coverage expectations for the PoC

- Every public function in `qtui-model`, `qtui-storage`, `qtui-config` has at
  least one test.
- Every UI mode has a snapshot test + at least one transition test.
- The agent bridge and preview lifecycle each have an integration test.
- The single e2e scenario above passes in the VM.

## Running

```bash
nix develop            # dev shell with rust, hugo, agent, beans, openspec
cargo test --workspace # unit + integration (host)
nix flake check        # everything, including the VM e2e test
```

## Test discipline for the agent

- Write the test first or alongside (the OpenSpec `tasks.md` should list test
  tasks explicitly).
- A bean is not closed until its tests are green.
- If a test is flaky, fix the flake or quarantine with a tracked bean — never
  delete to go green.
