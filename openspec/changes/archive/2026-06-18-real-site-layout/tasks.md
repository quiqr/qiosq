## 1. Crate setup

- [x] 1.1 Add `serde` + `serde_json` deps to `qtui-storage` (parse config.json). Keep it a pure crate.

## 2. Enumeration (real layout)

- [x] 2.1 Define a tolerant `SiteConfig` (serde): `source.path: Option<String>`, ignore unknown keys.
- [x] 2.2 `resolve_work_dir(site_dir, source.path)`: `"."`/`"./"`/empty/absent → site_dir; else `site_dir.join(path)`.
- [x] 2.3 `enumerate_sites(data_dir)`: read `<data>/sites/*`; a site = entry with a readable `config.json`; `Site.path` = resolved working copy; name = dir name; sort by name. Empty list when no `sites/`; error (names path) when data dir unreadable. Malformed config.json → exclude.

## 3. Downstream

- [x] 3.1 `content_tree` unchanged in signature (operates on `Site.path` = working copy now). `is_quiqr_site` kept for the working-copy `config.* + quiqr/` check used elsewhere.
- [x] 3.2 qtui binary host: site.path is the working copy → preview cwd, load_model, content_tree all use it directly (verify no assumption of a separate site dir).

## 4. Fixtures + tests

- [x] 4.1 Reshape the `real-site` fixture to `sites/examplesite/{config.json (source.path="main"), main/<the current contents>}`.
- [x] 4.2 Update qtui-storage unit tests + real_site test: build `<tmp>/sites/<name>/{config.json, main/…}`; assert enumeration finds them, resolves the working copy, excludes a `config.json`-less dir, and handles a `source.path="./"` flat site.
- [x] 4.3 Update qtui-model real-site golden test (path to the reshaped fixture's working copy).
- [x] 4.4 Update the e2e VM provisioning to the real layout (data dir gets `sites/examplesite/{config.json, main/…}`); `qtui --script` still passes.

## 5. Gate

- [x] 5.1 `cargo test --workspace` green; `cargo clippy --workspace --all-targets -- -D warnings` clean.
- [x] 5.2 `cargo fmt --all --check` clean.
- [x] 5.3 `nix flake check` green INCLUDING the VM e2e.

## 6. Done

- [x] 6.1 OpenSpec change `real-site-layout` validates.
- [x] 6.2 `qiosq-rblj` closed; commit references it; change archived. Unblocks `qiosq-r7gf`.
