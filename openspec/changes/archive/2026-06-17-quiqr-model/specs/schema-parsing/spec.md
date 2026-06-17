## ADDED Requirements

### Requirement: Locate and parse the model

The system SHALL read a site's schema from `quiqr/model/base.yaml` and the
`quiqr/model/includes/` directory. When the model directory is absent or empty,
the system SHALL return an empty model rather than an error, so a site without a
schema still loads.

#### Scenario: Site without a model yields an empty model
- **WHEN** a site has no `quiqr/model/` directory
- **THEN** parsing succeeds and yields a model with no menu groups, singles, or
  collections

#### Scenario: base.yaml and includes are read
- **WHEN** a site has `quiqr/model/base.yaml` and an `includes/` directory
- **THEN** both are read and contribute to the parsed model

### Requirement: Merge include roots from file or directory

The system SHALL recognise the include roots `menu`, `singles`, `collections`,
and `dynamics`. Each root MAY be provided either as a single file
(`includes/<root>.yaml`) or as a directory (`includes/<root>/`) whose YAML files
are merged. When a root is a directory, its files SHALL be merged in a stable
(name-sorted) order.

#### Scenario: Root provided as a single file
- **WHEN** `includes/collections.yaml` defines two collections
- **THEN** both collections appear in the parsed model

#### Scenario: Root provided as a directory
- **WHEN** `includes/singles/` contains two YAML files each defining a single
- **THEN** the singles from both files appear in the parsed model, in
  name-sorted file order

### Requirement: Tolerate partial, legacy, and malformed schemas

The system SHALL never panic on a real site's schema. A malformed or unreadable
include SHALL be skipped without aborting the rest of the parse; unknown keys
SHALL be ignored; and a `_mergePartial` reference SHALL be recorded on the entity
but never fetched (no network access).

#### Scenario: One malformed include does not abort parsing
- **WHEN** `includes/collections.yaml` is valid but `includes/menu.yaml`
  contains invalid YAML
- **THEN** parsing yields the collections and simply omits the menu (no panic,
  no overall error)

#### Scenario: Unknown keys are ignored
- **WHEN** a collection entry carries keys the model does not model (e.g.
  `hideIndex`, `dataformat`)
- **THEN** the collection is parsed using the keys it understands and the rest
  are ignored

#### Scenario: _mergePartial is recorded, not fetched
- **WHEN** a single declares `_mergePartial: some-partial`
- **THEN** the single is retained with its partial reference recorded and no
  network request is made
