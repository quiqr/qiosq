## Context

Cutting a release today is manual and error-prone: `0.0.0` is duplicated in
`Cargo.toml` and twice in `flake.nix`; there is no `CHANGELOG.md`, no tags, no
releases. The repo is `jj` (colocated git); `gh` is installed and authenticated.
We add a single bash script plus the small supporting changes (flake version
source, seed changelog, README docs).

## Goals / Non-Goals

**Goals:**
- One command: `scripts/release.sh <patch|minor|major>` → bump, changelog, tag,
  push, GitHub release.
- Version defined once (`Cargo.toml`); flake derives it.
- Safe by default: refuse a dirty tree unless `--allow-dirty`; `--dry-run`.
- Maintainer docs in the README.

**Non-Goals:**
- Auto-generating changelog entries from commits (maintainer curates
  `Unreleased`; chosen for quality over automation).
- Publishing crates to crates.io or building/uploading binaries (the GitHub
  release is notes-only for now).
- Running in `nix flake check` — this is operator tooling, not a CI gate.

## Decisions

- **Bash in `scripts/release.sh`**, run from the Nix dev shell (has `jj`, `git`,
  `gh`, `cargo`). `set -euo pipefail`; small functions; clear `die()` errors.
- **Version source = `Cargo.toml`.** `flake.nix` reads it via
  `(lib.importTOML ./Cargo.toml).workspace.package.version` for both
  `packages.default` and `checks.unit`, removing the two literals. The script
  edits only the `version = "X.Y.Z"` line under `[workspace.package]` (anchored
  sed/awk so it doesn't touch dependency version strings), then `cargo update
  -p` is unnecessary — instead run a lockfile refresh (`cargo generate-lockfile`
  is too broad; the workspace members' version is path-based, so just
  `cargo update --workspace --offline` or a targeted refresh). *Decision:* after
  editing Cargo.toml, run `cargo metadata`/build is overkill; the simplest
  correct refresh is `cargo update -p qtui --precise <ver>`-free — the path
  members don't pin a registry version, so `Cargo.lock` only needs the workspace
  members' `version` field updated, which `cargo` rewrites on the next build.
  The script runs a quiet `cargo update --workspace 2>/dev/null || true` then
  stages `Cargo.lock` if it changed. (Kept best-effort; a stale lock is caught by
  `nix flake check`, not the release path.)
- **Changelog promotion** with awk: find `## [Unreleased]`, capture lines until
  the next `## [` (or EOF), emit a fresh empty `## [Unreleased]` then
  `## [X.Y.Z] - <date>` with the captured body, then the rest. The captured body
  becomes the `gh release` notes. The date is `date +%F` (UTC-ish local; fine for
  a changelog).
- **VCS via jj.** Dirty check: `jj status` shows working-copy changes; treat
  "Working copy changes:" with entries as dirty. Commit: `jj describe -m` the
  current empty/working commit isn't right — the script makes its edits then
  `jj commit -m "release: vX.Y.Z"` (or describe + new). Tag: jj has no first-class
  tags, so create the git tag on the commit's git rev (`jj` is colocated) via
  `git tag -a vX.Y.Z -m`, then `jj git push` the bookmark and `git push origin
  vX.Y.Z`. *Decision:* use `git tag` + `git push origin <tag>` for the tag (jj
  exports the commit to git), and `jj bookmark set main -r @- && jj git push
  --bookmark main` for the branch.
- **GitHub release**: `gh release create vX.Y.Z --title vX.Y.Z --notes-file
  <tmp>` with the changelog section. Requires `gh` auth (documented).
- **Dry run** short-circuits before any mutation, printing the computed version,
  the changelog diff preview, and the tag/release it would create.

## Risks / Trade-offs

- **jj tag story is thin.** → Use the colocated git tag directly; documented.
  The script asserts it's in a jj repo and that `@` is exported to git before
  tagging.
- **Cargo.lock refresh nuance.** → Best-effort update + stage; `nix flake check`
  is the backstop for a stale lock. Avoid a hard cargo dependency in the hot
  path.
- **Destructive-ish (push + release).** → Mitigated by the dirty-guard,
  `--dry-run`, and printing each step; the script never force-pushes and never
  deletes.
- **Verification can't cut a real release.** → Test the pure parts (version math,
  changelog promotion, dirty detection, dry-run output) without pushing/tagging
  for real.

## Open Questions

- Whether to also bump versions in example configs/docs — none reference the
  crate version today, so no.
- Signing tags — out of scope; can add `-s` later.
