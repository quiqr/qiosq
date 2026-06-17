# site-enumeration Specification

## Purpose
TBD - created by archiving change quiqr-storage. Update Purpose after archive.
## Requirements
### Requirement: Enumerate sites under the data directory

The system SHALL enumerate the Quiqr sites under the configured data directory by
inspecting each immediate subdirectory, returning a list of sites identified by
their directory name and absolute path. Enumeration SHALL be read-only and SHALL
return the sites in a stable (name-sorted) order.

#### Scenario: Mixed directory yields only sites
- **WHEN** the data dir contains two valid Quiqr sites and one unrelated
  directory
- **THEN** enumeration returns exactly the two sites, sorted by name, each with
  its absolute path

#### Scenario: Empty data dir yields no sites
- **WHEN** the data dir exists but contains no site-shaped subdirectories
- **THEN** enumeration returns an empty list (not an error)

#### Scenario: Missing data dir errors
- **WHEN** the configured data dir does not exist or is not readable
- **THEN** enumeration returns an error that names the data dir path

### Requirement: Recognise a Quiqr/Hugo site by shape

The system SHALL treat a subdirectory as a Quiqr site if and only if it contains
both a Hugo site configuration (a `config.*` or `hugo.*` file, or a `config/`
directory) **and** a `quiqr/` directory. Directories missing either marker SHALL
be excluded.

#### Scenario: Site with config file + quiqr dir is recognised
- **WHEN** a subdirectory contains `config.toml` and a `quiqr/` directory
- **THEN** it is recognised as a site

#### Scenario: Hugo config variants are recognised
- **WHEN** a subdirectory uses `hugo.yaml` or a `config/` directory instead of
  `config.toml`, alongside a `quiqr/` directory
- **THEN** it is recognised as a site

#### Scenario: Missing quiqr dir is not a site
- **WHEN** a subdirectory has a Hugo config but no `quiqr/` directory
- **THEN** it is not recognised as a site

#### Scenario: Missing Hugo config is not a site
- **WHEN** a subdirectory has a `quiqr/` directory but no Hugo config
- **THEN** it is not recognised as a site

