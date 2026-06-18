# release-management Specification

## Purpose
TBD - created by archiving change release-tooling. Update Purpose after archive.
## Requirements
### Requirement: Single source of version truth

The workspace version SHALL be defined in one place (`Cargo.toml`), and the Nix
build SHALL derive its version from there rather than duplicating the literal.
A release bump SHALL therefore need to edit only `Cargo.toml`.

#### Scenario: Flake version tracks Cargo.toml
- **WHEN** the version in `Cargo.toml` changes
- **THEN** `nix build` and the checks report that same version without any
  separate edit to `flake.nix`

### Requirement: SemVer bump

`scripts/release.sh <level>` SHALL accept `patch`, `minor`, or `major` and
compute the next version from the current `Cargo.toml` version per SemVer:
`patch` increments Z, `minor` increments Y and zeroes Z, `major` increments X and
zeroes Y and Z. It SHALL update `Cargo.toml` (and refresh `Cargo.lock`).

#### Scenario: Patch/minor/major math
- **WHEN** the current version is `1.2.3` and the level is `patch` / `minor` /
  `major`
- **THEN** the next version is `1.2.4` / `1.3.0` / `2.0.0` respectively

#### Scenario: Invalid level is rejected
- **WHEN** the script is run with no level or an unrecognised one
- **THEN** it prints usage and exits non-zero without changing anything

### Requirement: Refuse a dirty working copy by default

The script SHALL refuse to run when the working copy has uncommitted changes,
exiting with a clear message, unless `--allow-dirty` is passed. It SHALL NOT
commit changes it did not make itself.

#### Scenario: Dirty repo aborts
- **WHEN** the working copy has uncommitted changes and `--allow-dirty` is not
  given
- **THEN** the script aborts with a message naming the dirty state and makes no
  changes

#### Scenario: --allow-dirty proceeds
- **WHEN** `--allow-dirty` is given on a dirty repo
- **THEN** the script proceeds and the commit it makes contains its own
  version/changelog edits

### Requirement: Promote the changelog

The script SHALL move the entries under the changelog's `Unreleased` section into
a new dated section headed `## [X.Y.Z] - YYYY-MM-DD`, leaving a fresh empty
`Unreleased` section, so each release captures the curated notes.

#### Scenario: Unreleased entries become a dated section
- **WHEN** the changelog has entries under `## [Unreleased]` and a release runs
- **THEN** those entries appear under a new `## [X.Y.Z] - <date>` section and
  `## [Unreleased]` is left empty above it

### Requirement: Tag, push, and create a GitHub release

After committing the bump, the script SHALL create an annotated tag `vX.Y.Z`,
push the branch and tag to the remote, and create a GitHub release for `vX.Y.Z`
whose notes are the new changelog section. It SHALL work in a `jj`-managed
(colocated git) repository.

#### Scenario: Release artifacts are produced
- **WHEN** a release completes (not a dry run)
- **THEN** a `vX.Y.Z` tag exists and is pushed, and a GitHub release for that tag
  is created with the changelog section as its notes

### Requirement: Dry run

The script SHALL support `--dry-run`, which prints the actions it would take (the
computed next version, the changelog promotion, the tag, the release) without
modifying the repository, committing, tagging, pushing, or calling GitHub.

#### Scenario: Dry run mutates nothing
- **WHEN** the script is run with `--dry-run`
- **THEN** it prints the planned next version and steps, and the working copy,
  tags, and remote are unchanged afterwards

