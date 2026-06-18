## 1. Version source + changelog seed

- [x] 1.1 `flake.nix`: derive `packages.default` + `checks.unit` version from `(lib.importTOML ./Cargo.toml).workspace.package.version`; remove the two `"0.0.0"` literals.
- [x] 1.2 Add `CHANGELOG.md` (Keep-a-Changelog) with a `## [Unreleased]` section (seed it with the work done so far).
- [x] 1.3 `nix flake check` still green (version now sourced from Cargo.toml).

## 2. scripts/release.sh

- [x] 2.1 Arg parsing: `<patch|minor|major>` + flags `--allow-dirty`, `--dry-run`; usage + non-zero exit on missing/invalid level. `set -euo pipefail`; `die()` helper.
- [x] 2.2 Read current version from `[workspace.package]` in Cargo.toml; compute next per SemVer (patch/minor/major).
- [x] 2.3 Dirty guard: abort if `jj status` shows working-copy changes, unless `--allow-dirty`.
- [x] 2.4 Bump: rewrite the anchored `version = "X.Y.Z"` line under `[workspace.package]`; best-effort `cargo update --workspace` to refresh Cargo.lock.
- [x] 2.5 Changelog promotion (awk): move `## [Unreleased]` body into `## [X.Y.Z] - <date>`, leave a fresh empty Unreleased; capture the body for release notes.
- [x] 2.6 Commit (jj), tag `vX.Y.Z` (git, colocated), push branch + tag, `gh release create vX.Y.Z` with the notes. Print each step.
- [x] 2.7 `--dry-run`: print computed version + changelog preview + planned tag/release; mutate nothing.
- [x] 2.8 `chmod +x scripts/release.sh`.

## 3. Maintainer docs

- [x] 3.1 README "Releasing" section: prerequisites (dev shell, `gh auth`), curate CHANGELOG Unreleased, `scripts/release.sh <level>`, `--dry-run`/`--allow-dirty`, what it does.

## 4. Verify (no real release)

- [x] 4.1 `--dry-run patch|minor|major` prints the correct next version (1.2.3 -> 1.2.4 / 1.3.0 / 2.0.0 on a temp fixture) and the changelog promotion preview, mutating nothing.
- [x] 4.2 Dirty-repo refusal works; `--allow-dirty` bypasses (dry-run).
- [x] 4.3 shellcheck clean (if available in shell); script is executable.
- [x] 4.4 cargo/clippy/fmt unaffected; `nix flake check` green.

## 5. Done

- [x] 5.1 OpenSpec change `release-tooling` validates.
- [x] 5.2 `qiosq-c0j2` closed; commit references it; change archived. (Do NOT cut a real release as part of this.)
