## MODIFIED Requirements

### Requirement: Enumerate sites under the data directory

The system SHALL enumerate the Quiqr sites under the configured data directory by
inspecting the entries of its `sites/` subdirectory, returning a list of sites
identified by their directory name and absolute path. Enumeration SHALL be
read-only and SHALL return the sites in a stable (name-sorted) order. When the
data directory has no `sites/` subdirectory, enumeration SHALL return an empty
list (not an error).

#### Scenario: Mixed sites directory yields only sites
- **WHEN** `<data>/sites/` contains two valid Quiqr sites and one unrelated
  directory
- **THEN** enumeration returns exactly the two sites, sorted by name, each with
  its absolute path

#### Scenario: Empty or absent sites directory yields no sites
- **WHEN** the data dir exists but has no `sites/` subdirectory, or `sites/` is
  empty of site-shaped entries
- **THEN** enumeration returns an empty list (not an error)

#### Scenario: Missing data dir errors
- **WHEN** the configured data dir does not exist or is not readable
- **THEN** enumeration returns an error that names the data dir path

### Requirement: Recognise a Quiqr/Hugo site by shape

The system SHALL treat an entry `<data>/sites/<name>/` as a Quiqr site if and
only if it contains a readable `config.json` site descriptor. The site's working
copy SHALL be resolved from that descriptor's `source.path` (relative to the site
directory): a value like `"main"` resolves to `<data>/sites/<name>/main/`, and
`"./"` (or an absent/empty path) resolves to the site directory itself. The
resolved working copy is where `quiqr/` and `content/` live. Entries without a
readable `config.json` SHALL be excluded.

#### Scenario: Standard site with a main working copy
- **WHEN** `sites/blog/config.json` has `source.path = "main"` and
  `sites/blog/main/quiqr/` exists
- **THEN** `blog` is recognised as a site whose working copy is `sites/blog/main`

#### Scenario: Flat site with a current-directory working copy
- **WHEN** `sites/old/config.json` has `source.path = "./"` and `sites/old/quiqr/`
  exists
- **THEN** `old` is recognised as a site whose working copy is `sites/old`

#### Scenario: Entry without a descriptor is not a site
- **WHEN** `sites/scratch/` exists but has no `config.json`
- **THEN** it is not recognised as a site

#### Scenario: Working copy is exposed for content + schema
- **WHEN** a site is enumerated
- **THEN** the returned site exposes the resolved working-copy path, so the
  content tree and schema model read from where `quiqr/` and `content/` are
