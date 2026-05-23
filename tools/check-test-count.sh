#!/usr/bin/env bash
# Guard against unchecked growth of the `#[test]` count.
#
# Tests are cheap to add and expensive to remove. Every new test pays for itself
# in CI time on every commit, and a slow suite makes the inner dev loop
# noticeably worse. The audit at reports/lyng/test-suite-audit-2026-05-23.md
# established the duplication patterns that produced the previous bloat;
# this guard makes regressions visible.
#
# A PR that pushes the count above the threshold either needs a one-line
# justification in the PR description (and a threshold bump in this script),
# or trimming.
#
# Override with $LYNG_TEST_COUNT_THRESHOLD when intentionally raising the bar.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

threshold="${LYNG_TEST_COUNT_THRESHOLD:-2600}"

count=$(
    cd "$repo_root"
    grep -RE "^\s*#\[test\]" --include="*.rs" crates tools 2>/dev/null | wc -l | tr -d ' '
)

if [[ "$count" -gt "$threshold" ]]; then
    cat >&2 <<EOF
test-count guard tripped:
  current #[test] functions: $count
  threshold:                 $threshold

If the new tests are genuinely needed, raise the threshold in
tools/check-test-count.sh and explain in the PR description.

Before adding new tests, check crates/AGENTS.md for the canonical-home
rule and table-driven test pattern. Most growth bursts come from
near-clone tests that should be one parameterized test instead.
EOF
    exit 1
fi

echo "test-count guard: $count / $threshold"
