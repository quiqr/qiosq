## ADDED Requirements

### Requirement: Discover candidate data folders from both editions

The system SHALL discover candidate Quiqr data folders by reading the Electron
desktop app's `instance_settings.json` (`storage.dataFolder`) from the per-OS
application config directory, plus a fixed set of fallback paths
(`~/Quiqr`, `~/Quiqr Data`). A leading `~` in any discovered path SHALL be
expanded to the user's home directory. Discovery SHALL be read-only.

#### Scenario: Reads the Electron-recorded data folder
- **WHEN** the app config dir contains `instance_settings.json` with
  `storage.dataFolder` set
- **THEN** that path (with `~` expanded) appears among the candidates, tagged as
  coming from the Electron settings

#### Scenario: Includes fallback paths
- **WHEN** discovery runs
- **THEN** the fallback paths `~/Quiqr` and `~/Quiqr Data` are included as
  candidates (tagged as fallbacks) in addition to any settings-derived path

#### Scenario: Tolerates missing or malformed settings
- **WHEN** there is no `instance_settings.json`, or it is not valid JSON, or it
  lacks `storage.dataFolder`
- **THEN** discovery does not fail; it simply contributes no Electron candidate
  and still returns the fallback candidates

### Requirement: De-duplicate and annotate candidates

The system SHALL return a de-duplicated list of candidates (by resolved absolute
path), each annotated with its source (Electron settings or fallback), whether it
is a valid Quiqr library (a directory containing a `sites/` directory with at
least one enumerable site), and its site count.

#### Scenario: The same path from two sources appears once
- **WHEN** the Electron-recorded data folder equals a fallback path after
  resolution
- **THEN** it appears exactly once in the candidate list

#### Scenario: A valid library reports its site count
- **WHEN** a candidate directory contains `sites/<name>/` entries with valid
  Quiqr sites
- **THEN** that candidate is marked valid with the correct site count

#### Scenario: A non-existent or empty candidate is marked invalid
- **WHEN** a candidate path does not exist, or exists but contains no Quiqr sites
- **THEN** it is included but marked not-valid with a site count of zero
