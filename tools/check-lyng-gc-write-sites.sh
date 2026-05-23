#!/usr/bin/env bash
set -euo pipefail

repo_root="${LYNG_GC_WRITE_SITES_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
doc="${LYNG_GC_WRITE_SITES_DOC:-$repo_root/docs/lyng-js/gc-write-sites.md}"
source_roots="${LYNG_GC_WRITE_SITES_SOURCE_ROOTS:-crates/lyng-js/gc crates/lyng-js/objects crates/lyng-js/env crates/lyng-js/vm}"

if [[ ! -f "$doc" ]]; then
    echo "missing GC write-site audit document: $doc" >&2
    exit 1
fi

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT
allowlist="$tmpdir/allowlist"
candidates="$tmpdir/candidates"
matched_allowlist="$tmpdir/matched-allowlist"
touch "$allowlist" "$candidates" "$matched_allowlist"

awk '
    /^<!-- gc-write-site-allowlist:start -->$/ { in_block = 1; next }
    /^<!-- gc-write-site-allowlist:end -->$/ { in_block = 0; next }
    in_block && $0 !~ /^[[:space:]]*($|#|<!--)/ { print }
' "$doc" > "$allowlist"

while IFS= read -r entry; do
    if [[ "$entry" != *$'\t'* ]]; then
        echo "invalid GC write-site allowlist entry, expected <path><tab><regex>: $entry" >&2
        exit 1
    fi
done < "$allowlist"

read -r -a roots <<< "$source_roots"
existing_roots=()
for root in "${roots[@]}"; do
    if [[ -e "$repo_root/$root" ]]; then
        existing_roots+=("$root")
    fi
done

if [[ "${#existing_roots[@]}" -eq 0 ]]; then
    echo "no GC write-site source roots exist under $repo_root" >&2
    exit 1
fi

write_pattern='(record[.](description|stored_value|linked_string|prototype|shape|named_slots|elements|private_slots|home_object|environment|private_env|outer|function_object|this_value|new_target|parent|realm|global_object|global_env|bootstrap_code|root_shape|prototype_guard|result|promise|resolve|reject|pending_error)[[:space:]]=[^=]|self[.](code|environment|namespace|deferred_namespace|import_meta_object|resolved_exports|evaluation_error)[[:space:]]=[^=]|self[.](promise_by_object|resolving_function_by_object|finally_function_by_object|combinator_element_by_object|capability_by_object|async_resume_by_object)\[[^]]+\][[:space:]]=[^=]|[*]target[[:space:]]=[^=]|self[.](entries|named_entries|dense_entries|feedback_vectors)\[[^]]+\][[:space:]]=[^=]|self[.](entries|named_entries|dense_entries|feedback_vectors)\[[^]]+\][[:space:]]=$|self[.](entries|root_shapes)[.]insert[(]|entries[.]insert[(].*(entry|payload|SparseElementEntry)|[.]transitions[[:space:]]*[.]insert[(]|properties[.]push[(]property[)]|lexical_bindings[.]push[(]binding[)]|self[.](promises|reactions|capabilities|resolving_functions|finally_functions|combinators|combinator_elements|async_operations|async_resumes)[.]push[(]Some[(]|record[.]resources[.]push[(]resource[)]|self[.]kept_objects[.]push[(]target[)]|self[.]environment_layouts[.]push[(]Some[(]layout[)])'

set +e
(
    cd "$repo_root"
    rg --line-number --no-heading \
        --glob '!**/tests/**' \
        --glob '!**/tests.rs' \
        --glob '!target/**' \
        -e "$write_pattern" \
        "${existing_roots[@]}"
) > "$candidates"
rg_status=$?
set -e

if [[ "$rg_status" -ne 0 && "$rg_status" -ne 1 ]]; then
    echo "GC write-site search failed" >&2
    exit "$rg_status"
fi

failures=0
while IFS= read -r candidate; do
    [[ -z "$candidate" ]] && continue
    path="${candidate%%:*}"
    rest="${candidate#*:}"
    line="${rest%%:*}"
    text="${rest#*:}"
    matched=0

    while IFS=$'\t' read -r allow_path allow_regex; do
        [[ -z "$allow_path" ]] && continue
        if [[ "$path" == "$allow_path" && "$text" =~ $allow_regex ]]; then
            matched=1
            printf '%s\t%s\n' "$allow_path" "$allow_regex" >> "$matched_allowlist"
            break
        fi
    done < "$allowlist"

    if [[ "$matched" -eq 0 ]]; then
        echo "undocumented GC write site: $path:$line:$text" >&2
        failures=1
    fi
done < "$candidates"

while IFS=$'\t' read -r allow_path allow_regex; do
    [[ -z "$allow_path" ]] && continue
    if ! grep -Fqx "$allow_path"$'\t'"$allow_regex" "$matched_allowlist"; then
        echo "stale GC write-site allowlist entry: $allow_path	$allow_regex" >&2
        failures=1
    fi
done < "$allowlist"

if [[ "$failures" -ne 0 ]]; then
    exit 1
fi

echo "GC write-site audit guard passed"
