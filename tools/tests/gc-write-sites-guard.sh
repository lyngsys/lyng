#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
checker="$repo_root/tools/check-lyng-js-gc-write-sites.sh"
tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

fixture_root="$tmpdir/repo"
source_dir="$fixture_root/crates/lyng-js/gc/src/arena"
doc_dir="$fixture_root/docs/lyng-js"
mkdir -p "$source_dir" "$doc_dir"

cat > "$source_dir/store_helpers.rs" <<'RS'
fn write_value(record: &mut PrimitiveValueCellRecord, value: Value) {
    record.stored_value = value;
}
RS

doc="$doc_dir/gc-write-sites.md"
cat > "$doc" <<'MD'
# GC Write Sites

<!-- gc-write-site-allowlist:start -->
<!-- gc-write-site-allowlist:end -->
MD

set +e
output="$(
    LYNG_GC_WRITE_SITES_ROOT="$fixture_root" \
    LYNG_GC_WRITE_SITES_DOC="$doc" \
    LYNG_GC_WRITE_SITES_SOURCE_ROOTS="crates/lyng-js/gc" \
    bash "$checker" 2>&1
)"
status=$?
set -e

if [[ "$status" -eq 0 ]]; then
    echo "expected checker to reject an undocumented write site" >&2
    exit 1
fi

if [[ "$output" != *"undocumented GC write site"* ]]; then
    echo "expected missing allowlist failure, got:" >&2
    echo "$output" >&2
    exit 1
fi

cat > "$doc" <<'MD'
# GC Write Sites

<!-- gc-write-site-allowlist:start -->
crates/lyng-js/gc/src/arena/store_helpers.rs	record[.]stored_value = value;
<!-- gc-write-site-allowlist:end -->
MD

LYNG_GC_WRITE_SITES_ROOT="$fixture_root" \
LYNG_GC_WRITE_SITES_DOC="$doc" \
LYNG_GC_WRITE_SITES_SOURCE_ROOTS="crates/lyng-js/gc" \
bash "$checker"
