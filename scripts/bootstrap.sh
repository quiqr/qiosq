#!/usr/bin/env bash
# Bootstrap the quiqr-tui PoC tooling. Safe to run inside `nix develop`.
# This script is intentionally conservative: it DISCOVERS the real CLI
# interfaces rather than assuming them, because both `beans` and `openspec`
# scaffold their own structures and exact syntax varies by version.
set -euo pipefail

say() { printf '\n=== %s ===\n' "$1"; }

say "Sanity: required tools"
for t in git cargo; do
  command -v "$t" >/dev/null || { echo "missing: $t (are you in 'nix develop'?)"; exit 1; }
done

say "beans"
if command -v beans >/dev/null; then
  echo "Discovering beans interface (prime/help) — the agent should READ this:"
  beans prime 2>/dev/null || beans --help || true
  echo
  echo "ACTION (agent): initialize beans if not already, then create the M1"
  echo "epics E1..E7 and their tasks from docs/epics-seed.md using the real CLI."
else
  echo "beans not found. Add it to flake.nix (E1, task T1.2) then re-run."
fi

say "OpenSpec"
if command -v openspec >/dev/null; then
  echo "Discovering openspec interface:"
  openspec --help || true
  echo
  echo "ACTION (agent): run the real init for Claude Code, confirm AGENTS.md is"
  echo "written, and turn openspec/changes/poc-foundation/ into a valid change."
else
  echo "openspec not found. Add it to flake.nix (E1, task T1.2) then re-run."
fi

say "Next steps for the agent"
cat <<'NEXT'
1. Read CLAUDE.md fully.
2. Initialize beans + openspec (above), seed E1..E7 from docs/epics-seed.md.
3. Start the poc-foundation (E1) change: scaffold the workspace + flake + config.
4. Keep beans, the openspec change, and code in sync. Tests gate every bean.
5. `nix flake check` must stay green.
NEXT
