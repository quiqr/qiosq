## Why

The WP-style left-pane navigation (the "Menu" of Singles and Collections) and the
constraints we feed the agent both come from a site's Quiqr schema
(`quiqr/model/base.yaml` + `quiqr/model/includes/*`). E2 can list sites and their
content files; E3 turns each site's schema into a `NavigationModel` the UI (E4)
renders and the agent adapter (E6) reads. Real sites carry partial, legacy, and
occasionally malformed schemas, so parsing MUST be tolerant — the browser can
never crash on a real site.

## What Changes

- Implement `qtui-model` (currently an empty stub):
  - Parse `quiqr/model/base.yaml` and merge the `includes/` roots — `menu`,
    `singles`, `collections`, `dynamics` — each of which may be a single
    `<name>.yaml` file **or** a `<name>/` directory of YAML files.
  - Build a read-only `NavigationModel`: ordered menu groups, each referencing
    Singles and Collections by key, with each Single mapped to its `file` and
    each Collection mapped to its `folder`.
  - Expose per-Single/Collection **field schemas** (`key`, `title`, `type`, and
    nested `fields`) so `qtui-agent` can constrain agent output to valid Quiqr
    front matter.
  - Be tolerant: unknown keys are ignored, malformed YAML in one include is
    skipped (not fatal), `_mergePartial` references are recorded but never
    fetched (no network), and a missing/empty model yields an empty
    `NavigationModel` rather than an error.
- Add golden-file fixtures (full, partial, malformed) and tests asserting the
  produced model.

## Capabilities

### New Capabilities
- `schema-parsing`: locate, parse, and merge a site's `quiqr/model/` YAML
  (base + includes, file-or-directory roots) tolerantly into raw schema data.
- `navigation-model`: assemble the parsed schema into an ordered Menu of Singles
  and Collections, each mapped to the content file/folder it represents.
- `field-schemas`: expose each Single's and Collection's field definitions
  (including nested fields) for downstream agent prompt constraints.

### Modified Capabilities
<!-- None — qtui-model was an empty stub from E1; no existing model spec. -->

## Impact

- `crates/qtui-model`: real implementation; adds `serde`/`serde_yaml` deps; stays
  a pure crate (no ratatui/rmux). Adds `serde_yaml` to the workspace.
- Adds golden-file fixtures under `crates/qtui-model/tests/fixtures/`.
- No change to other crates; the `qtui` binary wires this in with the UI (E4).
  Read-only: this crate never writes, and never mutates `quiqr/model/`.
