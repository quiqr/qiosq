# field-schemas Specification

## Purpose
TBD - created by archiving change quiqr-model. Update Purpose after archive.
## Requirements
### Requirement: Expose field definitions per entity

The system SHALL expose, for each Single and Collection, its ordered list of
field definitions. Each field definition SHALL carry at least its `key`, a
`title` (falling back to the key), and its `type`. These definitions are the
basis for constraining agent output to valid Quiqr front matter.

#### Scenario: Collection fields are exposed in order
- **WHEN** a collection `posts` defines fields `title` (string), `date` (date),
  and `tags` (chips)
- **THEN** the model exposes those three fields for `posts`, in that order, each
  with its key and type

#### Scenario: Field title falls back to key
- **WHEN** a field defines a `key` and `type` but no `title`
- **THEN** the exposed field's title equals its key

### Requirement: Preserve nested fields

The system SHALL preserve nested field definitions (a field that itself contains
`fields`, e.g. a `bundle-manager`), so a complete picture of the entity's shape
is available downstream.

#### Scenario: Nested fields are retained
- **WHEN** a field `images` of type `bundle-manager` contains a nested field
  `thumb`
- **THEN** the exposed `images` field retains its nested `thumb` field

### Requirement: Entities without fields are valid

The system SHALL treat a Single or Collection with no field definitions as valid,
exposing an empty field list rather than failing — common for singles that defer
fields to a `_mergePartial`.

#### Scenario: Single deferring fields to a partial has no inline fields
- **WHEN** a single declares only a `key` and `_mergePartial` with no inline
  `fields`
- **THEN** the model exposes that single with an empty field list and its
  recorded partial reference

