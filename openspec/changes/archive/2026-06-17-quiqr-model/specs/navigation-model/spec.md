## ADDED Requirements

### Requirement: Assemble the navigation model

The system SHALL produce a `NavigationModel` consisting of ordered menu groups,
each with a key, a title, and an ordered list of entries; each entry SHALL
reference either a Single or a Collection defined in the schema. The menu order
SHALL follow the order declared in the schema.

#### Scenario: Menu groups reference singles and collections
- **WHEN** the menu declares a group "Content" with items `pages` and `posts`,
  and those are defined as collections
- **THEN** the model has a "Content" group whose entries resolve to the `pages`
  and `posts` collections, in that order

#### Scenario: Menu entry referencing an unknown key is dropped
- **WHEN** a menu item references a key that is neither a single nor a collection
- **THEN** that entry is omitted from the group rather than producing a broken
  reference

### Requirement: Map Singles and Collections to their paths

The system SHALL map each Single to the content file it edits (its `file`) and
each Collection to the folder it manages (its `folder`), so the UI can open the
right path and the agent can be pointed at it.

#### Scenario: Single carries its file path
- **WHEN** a single `frontpage` declares `file: /content/_index.md`
- **THEN** the model's `frontpage` single exposes that file path

#### Scenario: Collection carries its folder path
- **WHEN** a collection `posts` declares `folder: content/post`
- **THEN** the model's `posts` collection exposes that folder path

### Requirement: Singles and Collections are addressable by key

The system SHALL key Singles and Collections by their schema `key` and expose a
human-facing title (falling back to the key when no title is given), so the UI
can render labels and the rest of the system can look entities up by key.

#### Scenario: Title falls back to key
- **WHEN** a single defines a `key` but no `title`
- **THEN** the model exposes that single with its title equal to its key
