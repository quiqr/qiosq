## Context

`qtui-model` (an E1 stub) must turn a site's `quiqr/model/` schema into the
read-only `NavigationModel` the UI renders and the field schemas the agent reads.
The shape is taken from real Quiqr sites:

- `base.yaml` — build/serve config (not needed for navigation; parsed but
  largely ignored in E3).
- `includes/menu.yaml` — ordered groups: `{key, title, menuItems: [{key}],
  matchRole?}`. `menuItems[].key` references a single or collection.
- `includes/singles.yaml` — `[{key, title?, file?, fields?, _mergePartial?}]`.
- `includes/collections.yaml` — `[{key, title?, folder, fields?, …}]`.
- `includes/dynamics.yaml` — present on some sites; parsed leniently, not yet
  surfaced in the menu (no UI need until later).

Each `includes/<root>` may be a file or a directory of files. Many entities use
`_mergePartial` (a local name or a URL) instead of inline fields.

## Goals / Non-Goals

**Goals:**
- A tolerant parser + merger producing a `NavigationModel` (menu → singles /
  collections → paths) and per-entity field schemas.
- Golden-file tests over realistic fixtures, including partial and malformed.

**Non-Goals:**
- Resolving `_mergePartial` (local or remote) — recorded only; no network, no
  filesystem partial lookup in E3. A later epic can resolve partials if needed.
- Validating field *values* or generating the agent prompt — that is E6.
- Rendering — E4. Writing or editing the schema — never (read-only).

## Decisions

- **Tolerant deserialization via an untyped intermediate.** Parse each YAML file
  to `serde_yaml::Value` first, then project into typed structs field-by-field,
  ignoring unknown keys and missing optionals. This is more robust on legacy
  schemas than `#[derive(Deserialize)]` with `deny_unknown_fields`, and lets a
  single malformed file be caught and skipped without aborting the parse.
  *Alternative considered:* strict typed structs with `serde(default)` — rejected
  because real schemas carry many keys we don't model and occasional shape drift.
- **Per-file isolation.** Each include file/entry is parsed independently; a
  parse error is logged into a `warnings` list on the result and skipped. The
  parse never returns `Err` for content problems — only genuinely unreadable
  inputs surface, and even those degrade to "omit that root".
- **Menu resolution.** Menu items reference entities by key; resolution looks the
  key up among singles then collections. Unresolved keys are dropped (with a
  warning) so the UI never holds a dangling reference.
- **Field model.** A recursive `Field { key, title, type_, fields: Vec<Field> }`
  captures nested fields (e.g. `bundle-manager` → `thumb`). `type_` is kept as a
  free string (Quiqr has many widget types); we do not enumerate them.
- **`_mergePartial`.** Stored as `Option<String>` on Single/Collection; never
  dereferenced.

## Risks / Trade-offs

- **Untyped projection is more verbose.** → Accept; it is the price of tolerance,
  and it is well covered by golden-file tests.
- **Dynamics under-modelled.** → Acceptable: no UI consumer yet; we parse them
  leniently and can promote them into the menu in a later change without a
  breaking spec edit.
- **Warnings are easy to ignore.** → Expose them on the result so tests (and a
  future diagnostics view) can assert on them; do not print to stderr from a
  library.

## Open Questions

- Whether to fold `dynamics` into the menu now. Deferred — no consumer in M1's
  UI scope. Left parsed-but-unsurfaced; revisit if E4 needs it.
