# content-tree Specification

## Purpose
TBD - created by archiving change quiqr-storage. Update Purpose after archive.
## Requirements
### Requirement: Build a read-only content tree

The system SHALL build a tree of a site's `content/` directory, where each node
is either a directory (with children) or a file (a leaf), carrying its name and
path relative to `content/`. Children SHALL be ordered directories-first then by
name, so the UI can render a stable navigation. The operation SHALL only read the
filesystem and SHALL NOT create, modify, or delete any file.

#### Scenario: Nested content is represented as a tree
- **WHEN** `content/` contains `posts/hello.md` and `about.md`
- **THEN** the tree has a `posts` directory node containing `hello.md` and a
  top-level `about.md` file node

#### Scenario: Missing content dir yields an empty tree
- **WHEN** a site has no `content/` directory
- **THEN** the tree is empty (the root has no children) rather than an error

### Requirement: Hide derived, generated, and VCS directories

The system SHALL omit from the content tree any directory whose name matches the
configured hidden set (default: `public`, `resources`, `.quiqr-cache`, `.git`,
`themes`), at every depth, so authors never see build output or VCS internals.

#### Scenario: Hidden directories are omitted
- **WHEN** `content/` contains a `posts/` directory and a `.git/` directory
- **THEN** the tree includes `posts` and omits `.git`

#### Scenario: Hidden names are filtered at any depth
- **WHEN** a `resources/` directory appears nested inside a content
  subdirectory
- **THEN** that nested `resources/` directory is omitted from the tree

#### Scenario: Configured hidden set overrides the default
- **WHEN** the configuration sets `hidden_dirs` to a custom list
- **THEN** the tree hides exactly the directories named in that list

