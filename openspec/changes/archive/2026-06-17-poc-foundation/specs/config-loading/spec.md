## ADDED Requirements

### Requirement: Load configuration from a TOML file

The system SHALL load its configuration from a single TOML file whose path is
provided by the caller, parsing the `[storage]`, `[preview]`, `[agent]`,
`[agent.sandbox]`, `[ui]`, and `[rmux]` sections into a typed configuration
value. The example configuration at `config/quiqr-tui.example.toml` SHALL load
without error.

#### Scenario: Example config loads successfully
- **WHEN** the loader is given the path to `config/quiqr-tui.example.toml`
- **THEN** it returns a configuration value with the storage data dir, agent
  command and args, hugo binary, preview port range, and completion sentinel
  populated from the file

#### Scenario: Missing file reports a clear error
- **WHEN** the loader is given a path that does not exist
- **THEN** it returns an error that names the missing path rather than panicking

#### Scenario: Malformed TOML reports a clear error
- **WHEN** the loader is given a file that is not valid TOML
- **THEN** it returns an error describing the parse failure and the file path

### Requirement: Resolve the Quiqr data directory

The system SHALL resolve `storage.quiqr_data_dir` to an absolute path and SHALL
refuse to start with a clear error if the value is empty.

#### Scenario: Relative data dir is resolved to absolute
- **WHEN** `storage.quiqr_data_dir` is a relative path
- **THEN** the loaded configuration exposes it as an absolute path

#### Scenario: Empty data dir errors
- **WHEN** `storage.quiqr_data_dir` is empty
- **THEN** validation returns a distinct, human-readable error identifying the
  `storage.quiqr_data_dir` field

### Requirement: Validate the preview port range

The system SHALL validate `preview.port_range` as an inclusive `[low, high]`
pair where `low <= high`, both ports are valid (1–65535), and the range MUST NOT
include Quiqr's default port `13131`.

#### Scenario: Range including 13131 errors
- **WHEN** `preview.port_range` includes `13131`
- **THEN** validation returns a distinct error explaining that `13131` is
  reserved for Quiqr and naming the `preview.port_range` field

#### Scenario: Inverted range errors
- **WHEN** `preview.port_range` has `low` greater than `high`
- **THEN** validation returns a distinct error naming the `preview.port_range`
  field

#### Scenario: Valid range passes
- **WHEN** `preview.port_range` is `[13140, 13200]`
- **THEN** validation succeeds

### Requirement: Validate the agent configuration

The system SHALL require a non-empty `agent.command` and a non-empty
`agent.completion_sentinel`, keeping the agent command swappable so a fake agent
can be substituted in tests.

#### Scenario: Empty agent command errors
- **WHEN** `agent.command` is empty
- **THEN** validation returns a distinct error naming the `agent.command` field

#### Scenario: Empty completion sentinel errors
- **WHEN** `agent.completion_sentinel` is empty
- **THEN** validation returns a distinct error naming the
  `agent.completion_sentinel` field

### Requirement: Surface distinct errors per invalid field

The system SHALL ensure that each independently invalid configuration field
produces a distinct, human-readable error message, so a misconfiguration can be
diagnosed without reading the source.

#### Scenario: Each invalid field has its own message
- **WHEN** validation is run across configs that are each invalid in exactly one
  field
- **THEN** each field's error message is distinct from the others and names the
  offending field
