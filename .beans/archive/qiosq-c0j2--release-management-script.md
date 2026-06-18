---
# qiosq-c0j2
title: release management script
status: completed
type: task
priority: normal
created_at: 2026-06-18T07:41:13Z
updated_at: 2026-06-18T08:37:19Z
---

- [x] manage changelog
- [x] patch, minor, major versions
- [x] publish tag and create release on github
- [x] work from jj and git repo
- [x] fix uncommitted stuff if repo is dirty (refuse unless --allow-dirty)
- [x] add maintainer docs in readme

## Summary of Changes
Added scripts/release.sh (bash, dev shell): `release.sh <patch|minor|major> [--dry-run] [--allow-dirty]`.
- SemVer bump from Cargo.toml (single source of truth — flake.nix now derives version via lib.importTOML, removing the two hardcoded 0.0.0 literals). Anchored awk edits only the [workspace.package] version line (leaves dependency versions untouched); best-effort cargo update for Cargo.lock.
- CHANGELOG.md seeded (Keep-a-Changelog); the script promotes [Unreleased] into a dated ## [X.Y.Z] - DATE section and uses it as the gh release notes.
- jj-aware: refuses a dirty working copy unless --allow-dirty; jj commit; git tag vX.Y.Z (colocated); jj git push + git push origin tag; gh release create. --dry-run mutates nothing.
- README Releasing section; Status refreshed; gh + shellcheck added to the dev shell.
- Verified: version math 0.0.0->{0.0.1,0.1.0,1.0.0} and 1.2.3->{1.2.4,1.3.0,2.0.0}; invalid exit 2; dirty refusal; fixture bump edits only the workspace version + promotes changelog; shellcheck clean; nix flake check green (incl VM e2e). Did NOT cut a real release.
- Found + fixed a real bug: the bump awk used `next` as a -v var (gawk reserved word) and silently skipped the version bump until renamed to `ver`.

OpenSpec change release-tooling archived (capability release-management).
