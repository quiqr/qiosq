## 1. Crate setup

- [x] 1.1 Add `serde` + `serde_yaml` deps to `qtui-model` (serde_yaml added to the workspace); keep it a pure crate.
- [x] 1.2 Define the model types: `NavigationModel`, `MenuGroup`, `MenuEntry`, `Single`, `Collection`, `Field`, plus a `warnings: Vec<String>` channel.

## 2. Schema parsing + merge

- [x] 2.1 Locate `quiqr/model/` (base.yaml + includes/); empty/absent model -> empty NavigationModel (no error).
- [x] 2.2 For each include root (menu, singles, collections, dynamics): read a `<root>.yaml` file OR a `<root>/` dir (files merged in name order).
- [x] 2.3 Tolerant projection via `serde_yaml::Value`: ignore unknown keys; skip a malformed file/entry into `warnings`; never panic; never fetch `_mergePartial`.

## 3. Navigation model + field schemas

- [x] 3.1 Parse singles (`key`, `title`?->key, `file`?, `fields`?, `_mergePartial`?) and collections (`key`, `title`?->key, `folder`, `fields`?).
- [x] 3.2 Parse fields recursively into `Field { key, title?->key, type_, fields }` (nested fields preserved).
- [x] 3.3 Build the menu: ordered groups -> ordered entries resolving keys to single/collection; drop unknown keys (warn).
- [x] 3.4 Expose lookups: singles/collections by key; their field lists.

## 4. Golden-file fixtures + tests

- [x] 4.1 Fixtures under `crates/qtui-model/tests/fixtures/`: `full/` (menu+singles+collections, nested + file/folder paths), `partial/` (collections only, no menu), `malformed/` (one bad include + one good), `dir-include/` (a `singles/` directory).
- [x] 4.2 Tests assert the NavigationModel: menu order + entry resolution; single.file / collection.folder; title fallback; nested fields; unknown-key tolerance; malformed include skipped with a warning; empty model for no schema.

## 5. Gate

- [x] 5.1 `cargo test --workspace` green; `cargo clippy --workspace --all-targets -- -D warnings` clean (dev shell).
- [x] 5.2 `cargo fmt --all --check` clean.
- [x] 5.3 `nix flake check` green.

## 6. Done

- [x] 6.1 OpenSpec change `quiqr-model` validates.
- [x] 6.2 E3 beans closed; commit references `qiosq-9m0x`; change archived.
