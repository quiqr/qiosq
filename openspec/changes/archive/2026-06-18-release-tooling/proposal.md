## Why

There is no repeatable way to cut a release: the version (`0.0.0`) is hand-set in
three places (`Cargo.toml` + two copies in `flake.nix`), there is no changelog,
no tags, and no GitHub releases. A maintainer cutting a release would have to
remember each step and could easily forget the flake copies or push a dirty tree.
A single `scripts/release.sh` makes releases one command and documents the
process for maintainers.

## What Changes

- **Single source of version truth**: make `flake.nix` read the version from the
  workspace `Cargo.toml` instead of hardcoding `0.0.0`, so a release bumps one
  file.
- **Seed `CHANGELOG.md`** in Keep-a-Changelog format with an `## [Unreleased]`
  section maintainers add to between releases.
- **`scripts/release.sh <patch|minor|major>`**: compute the next SemVer from the
  current `Cargo.toml` version; refuse to run on a dirty working copy unless
  `--allow-dirty`; bump `Cargo.toml` (+ refresh `Cargo.lock`); move the
  `Unreleased` changelog entries into a new dated `## [X.Y.Z] - YYYY-MM-DD`
  section; commit the bump (via `jj`); create an annotated tag `vX.Y.Z`; push the
  branch + tag; and create a GitHub release (`gh`) with the new section's notes.
  A `--dry-run` prints the plan without mutating anything.
- **Maintainer docs**: a "Releasing" section in the README describing the flow.

## Capabilities

### New Capabilities
- `release-management`: the `scripts/release.sh` workflow — version bump,
  changelog promotion, tag, push, and GitHub release — jj-aware and dirty-repo
  safe.

### Modified Capabilities
<!-- nix-foundation is touched (version sourced from Cargo.toml) but its
     requirements (dev shell, build, checks) don't change behaviour. -->

## Impact

- New `scripts/release.sh` (bash; runs in the Nix dev shell, uses `jj`, `git`,
  `gh`, `cargo`).
- New `CHANGELOG.md`.
- `flake.nix`: `packages.default` + `checks.unit` versions derived from
  `Cargo.toml` (via `lib.importTOML`) rather than literal `"0.0.0"`.
- `README.md`: a Releasing section.
- No change to crate code or runtime behaviour; `nix flake check` stays green.
  The script is operator tooling — it is not run by `nix flake check`.
