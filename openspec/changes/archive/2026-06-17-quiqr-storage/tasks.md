## 1. Crate setup

- [x] 1.1 Add `tempfile` as a workspace dev-dependency; wire it into `qtui-storage` dev-deps.
- [x] 1.2 Define error type (`StorageError`) in `qtui-storage` with a missing/unreadable-data-dir variant that names the path.

## 2. Site enumeration

- [x] 2.1 Define the `Site { name, path }` type.
- [x] 2.2 Implement `is_quiqr_site(dir)`: Hugo config (`config.*`/`hugo.*` file or `config/` dir) AND a `quiqr/` dir.
- [x] 2.3 Implement `enumerate_sites(data_dir)`: read immediate subdirs, keep sites, sort by name; error on missing/unreadable data dir; empty list when none.

## 3. Content tree

- [x] 3.1 Define `ContentNode` (Dir{name, rel_path, children} | File{name, rel_path}).
- [x] 3.2 Implement `content_tree(site, hidden_dirs)`: recursive walk of `content/`, dirs-first then name order; empty tree when no `content/`.
- [x] 3.3 Hide directories whose base name is in `hidden_dirs`, at every depth; do not follow symlinks.

## 4. Fixtures + tests

- [x] 4.1 Test helper that builds a Quiqr-site-shaped tempdir (config + quiqr/ + content/).
- [x] 4.2 Enumeration tests: mixed dir → only sites (sorted); empty → none; missing data dir → error; config variants recognised; missing-marker dirs excluded.
- [x] 4.3 Content-tree tests: nested tree shape; missing content → empty; hidden dirs omitted at top level and nested; custom hidden set honoured.
- [x] 4.4 Assert the crate never writes (operations on a read-only-ish fixture leave it unchanged).

## 5. Gate

- [x] 5.1 `cargo test --workspace` green; `cargo clippy --workspace --all-targets -- -D warnings` clean (in dev shell).
- [x] 5.2 `cargo fmt --all --check` clean.
- [x] 5.3 `nix flake check` green.

## 6. Done

- [x] 6.1 OpenSpec change `quiqr-storage` validates.
- [x] 6.2 E2 beans closed; commit references `qiosq-8z4l`; change archived.
