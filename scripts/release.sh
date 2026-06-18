#!/usr/bin/env bash
# Release management for qiosq. Bumps the workspace version, promotes the
# CHANGELOG's Unreleased section, tags, pushes, and creates a GitHub release.
#
# Usage:
#   scripts/release.sh <patch|minor|major> [--dry-run] [--allow-dirty]
#
# Run it from the Nix dev shell (needs jj, git, gh, cargo). See the README's
# "Releasing" section. Refuses a dirty working copy unless --allow-dirty; never
# touches changes it did not make.
set -euo pipefail

die() { printf 'release: %s\n' "$1" >&2; exit 1; }
note() { printf '==> %s\n' "$1"; }

usage() {
  cat >&2 <<'EOF'
usage: scripts/release.sh <patch|minor|major> [--dry-run] [--allow-dirty]

  patch | minor | major   how to bump the current version (X.Y.Z)
  --dry-run               print the plan; change nothing
  --allow-dirty           proceed even if the working copy has changes
EOF
  exit 2
}

# ---- parse args -------------------------------------------------------------
LEVEL=""
DRY_RUN=0
ALLOW_DIRTY=0
for arg in "$@"; do
  case "$arg" in
    patch|minor|major) LEVEL="$arg" ;;
    --dry-run)         DRY_RUN=1 ;;
    --allow-dirty)     ALLOW_DIRTY=1 ;;
    -h|--help)         usage ;;
    *)                 printf 'release: unknown argument %q\n\n' "$arg" >&2; usage ;;
  esac
done
[ -n "$LEVEL" ] || usage

# ---- locate repo root -------------------------------------------------------
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
CARGO_TOML="$ROOT/Cargo.toml"
CHANGELOG="$ROOT/CHANGELOG.md"
[ -f "$CARGO_TOML" ] || die "no Cargo.toml at repo root ($ROOT)"

for tool in jj git gh cargo awk; do
  command -v "$tool" >/dev/null 2>&1 || die "missing tool: $tool (run inside 'nix develop')"
done

# ---- current + next version -------------------------------------------------
# Read the version line under [workspace.package]. We restrict to that section
# so dependency version strings are never matched.
CURRENT="$(awk '
  /^\[workspace\.package\]/ { in_sec=1; next }
  /^\[/                     { in_sec=0 }
  in_sec && /^[[:space:]]*version[[:space:]]*=/ {
    gsub(/.*=[[:space:]]*"/, ""); gsub(/".*/, ""); print; exit
  }
' "$CARGO_TOML")"
[ -n "$CURRENT" ] || die "could not read version from [workspace.package] in Cargo.toml"
[[ "$CURRENT" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || die "current version '$CURRENT' is not X.Y.Z"

IFS='.' read -r MAJ MIN PAT <<<"$CURRENT"
case "$LEVEL" in
  patch) PAT=$((PAT + 1)) ;;
  minor) MIN=$((MIN + 1)); PAT=0 ;;
  major) MAJ=$((MAJ + 1)); MIN=0; PAT=0 ;;
esac
NEXT="$MAJ.$MIN.$PAT"
TAG="v$NEXT"
DATE="$(date +%F)"

note "current version: $CURRENT"
note "next version:    $NEXT  (tag $TAG, $LEVEL bump)"

# ---- dirty guard (skipped for --dry-run, which mutates nothing) -------------
# jj prints "The working copy has no changes." when clean.
if [ "$DRY_RUN" -eq 0 ] && ! jj status 2>/dev/null | grep -q 'The working copy has no changes\.'; then
  if [ "$ALLOW_DIRTY" -eq 0 ]; then
    die "working copy has uncommitted changes; commit them or pass --allow-dirty"
  fi
  note "working copy is dirty; proceeding (--allow-dirty)"
fi

# ---- compute the changelog notes for this release --------------------------
# Capture the body under '## [Unreleased]' up to the next '## [' (or EOF).
NOTES=""
if [ -f "$CHANGELOG" ]; then
  NOTES="$(awk '
    /^## \[Unreleased\]/ { grab=1; next }
    grab && /^## \[/     { grab=0 }
    grab                 { print }
  ' "$CHANGELOG" | sed '/^[[:space:]]*$/d')"
fi
[ -n "$NOTES" ] || NOTES="Release $TAG."

# ---- dry run ----------------------------------------------------------------
if [ "$DRY_RUN" -eq 1 ]; then
  note "DRY RUN — no changes will be made"
  echo "would bump Cargo.toml: $CURRENT -> $NEXT"
  echo "would promote CHANGELOG '[Unreleased]' -> '## [$NEXT] - $DATE'"
  echo "would commit, tag $TAG, push, and 'gh release create $TAG'"
  echo "--- release notes preview ---"
  printf '%s\n' "$NOTES"
  echo "-----------------------------"
  exit 0
fi

# ---- bump Cargo.toml --------------------------------------------------------
note "bumping Cargo.toml"
awk -v ver="$NEXT" '
  /^\[workspace\.package\]/ { in_sec=1; print; next }
  /^\[/                     { in_sec=0 }
  in_sec && /^[[:space:]]*version[[:space:]]*=/ && !done {
    sub(/"[0-9]+\.[0-9]+\.[0-9]+"/, "\"" ver "\""); done=1
  }
  { print }
' "$CARGO_TOML" >"$CARGO_TOML.tmp" && mv "$CARGO_TOML.tmp" "$CARGO_TOML"

# Refresh Cargo.lock for the new workspace-member versions (best effort; a stale
# lock is caught by `nix flake check`, not here).
cargo update --workspace >/dev/null 2>&1 || true

# ---- promote the changelog --------------------------------------------------
if [ -f "$CHANGELOG" ]; then
  note "promoting CHANGELOG"
  awk -v ver="$NEXT" -v date="$DATE" '
    /^## \[Unreleased\]/ && !done {
      print "## [Unreleased]"
      print ""
      print "## [" ver "] - " date
      done=1
      next
    }
    { print }
  ' "$CHANGELOG" >"$CHANGELOG.tmp" && mv "$CHANGELOG.tmp" "$CHANGELOG"
fi

# ---- commit, tag, push, release ---------------------------------------------
note "committing the bump"
jj describe -m "release: $TAG"
# Seal the release commit and continue on a fresh working-copy commit.
jj new >/dev/null

# Tag the release commit (its parent, @-). jj is colocated, so tag via git on the
# exported rev, then push branch + tag.
REL_REV="$(jj log --no-graph -r '@-' -T 'commit_id' 2>/dev/null)"
[ -n "$REL_REV" ] || die "could not resolve the release commit revision"

note "tagging $TAG"
git tag -a "$TAG" -m "release $TAG" "$REL_REV"

note "pushing branch + tag"
jj bookmark set main -r "@-"
jj git push --bookmark main
git push origin "$TAG"

note "creating GitHub release $TAG"
NOTES_FILE="$(mktemp)"
printf '%s\n' "$NOTES" >"$NOTES_FILE"
gh release create "$TAG" --title "$TAG" --notes-file "$NOTES_FILE"
rm -f "$NOTES_FILE"

note "done: released $TAG"
