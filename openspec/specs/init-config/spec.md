# init-config Specification

## Purpose
TBD - created by archiving change init-config. Update Purpose after archive.
## Requirements
### Requirement: The init flow chooses a data folder and writes the config

The `qtui init` flow SHALL present the discovered candidates (each with its path,
source, and site count), let the user choose one or enter a custom path, and write
a valid qtui configuration with the chosen `storage.quiqr_data_dir`. The written
config SHALL load and validate via the normal config loader.

#### Scenario: User chooses a discovered candidate
- **WHEN** `qtui init` runs interactively and the user selects a discovered
  candidate
- **THEN** a config is written whose `storage.quiqr_data_dir` is that candidate's
  path, and it round-trips through the config loader

#### Scenario: User enters a custom path
- **WHEN** the user opts to enter a path not among the candidates
- **THEN** the written config uses that path

### Requirement: Default config location and non-clobbering

The system SHALL write (and by default read) the config at
`~/.config/qiosq/config.toml` (the XDG config dir). `qtui init` SHALL NOT
overwrite an existing config without explicit consent.

#### Scenario: Init writes to the XDG config path
- **WHEN** `qtui init` completes with no pre-existing config
- **THEN** the config is created at `~/.config/qiosq/config.toml`

#### Scenario: Existing config is not clobbered silently
- **WHEN** `qtui init` runs and a config already exists at the default path
- **THEN** it does not overwrite it without the user's explicit consent

### Requirement: Headless init is non-interactive

In headless/non-interactive mode the init flow SHALL NOT prompt. It SHALL
auto-select when exactly one valid candidate exists, and otherwise SHALL fail with
a clear error that lists the candidates it found.

#### Scenario: Single valid candidate is auto-selected
- **WHEN** init runs non-interactively and exactly one candidate is a valid Quiqr
  library
- **THEN** it selects that candidate without prompting and writes the config

#### Scenario: Ambiguous or empty discovery errors clearly
- **WHEN** init runs non-interactively and there is not exactly one valid
  candidate
- **THEN** it returns an error listing the candidates found, without prompting

### Requirement: Bare launch resolves the default config

When `qtui` is launched without `--config`, it SHALL load the config from the
default path if present; if absent, it SHALL run or offer the init flow rather
than failing with a missing-argument error. An explicit `--config <path>` SHALL
take precedence over the default.

#### Scenario: Bare launch loads the default config
- **WHEN** `qtui` is run with no `--config` and the default config exists
- **THEN** it loads that config and proceeds

#### Scenario: --config overrides the default
- **WHEN** `qtui --config <path>` is run and a default config also exists
- **THEN** the explicitly given config is used

