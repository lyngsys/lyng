# Lyng JS Regex Engine Research

This note records the state of regular-expression support in Lyng JS as of
2026-05-03 and proposes a tiered roadmap for taking the regex path from
"correct via `regress`" to "lightning fast and self-owned". It is research
only — no dependency or architectural change is approved by this document.

## Context

Lyng JS embeds the [`regress`](https://crates.io/crates/regress) crate
(`v0.11.1`, `utf16` feature) as its only regex backend. `regress` is the
single non-trivial third-party runtime dependency in the JS engine; the
remaining externals (`hashbrown`, `bitflags`, `unicode-id-start`) are passive
data-structure or table crates. Internalizing or replacing the matcher is
therefore the dominant lever both for shedding external code and for closing
the perf gap to V8/JSC, whose regex engines are bytecode VMs with optional
JIT.

The engine has already absorbed substantial regex-shaped logic on top of
`regress` to paper over backend gaps and to short-circuit common patterns.
That work is the natural foundation for an in-tree matcher: it shows where
the backend underperforms the spec and where we already prefer to bypass it.

## Current Architecture

### Crate layout

| Layer | Crate | Role |
| --- | --- | --- |
| Pattern data + compile | `lyng-js-objects` (`objects/src/regexp.rs`, 1398 lines) | `RegExpPayload`, flag parsing, pattern normalization, fast-pattern detection, `regress::Regex` wrapper |
| Runtime dispatch | `lyng-js-builtins` (`builtins/src/public/dispatch/regexp.rs`, 1475 lines + `.../regexp/symbols.rs`) | `RegExp.prototype.{exec,test,compile,...}`, `Symbol.{match,replace,split,search,matchAll}` |
| String integration | `lyng-js-builtins` (`builtins/src/public/dispatch/strings.rs`, 1410 lines) | `String.prototype.{match,matchAll,replace,replaceAll,search,split}` |
| Conversion | `builtins/src/public/dispatch/support.rs:1503` | `string_ref_code_units` — produces the `Vec<u16>` handed to `regress` |
| Backend | external `regress` crate | NFA construction, backtracking matcher, capture extraction |

### The `regress` API surface we use

Only a thin slice of `regress` is exercised:

```text
regress::Flags { icase, multiline, dot_all, unicode, unicode_sets, no_opt: false }
regress::Regex::with_flags(&str, Flags)
regex.find_from_utf16(&[u16], start_index)
regex.find_from_ucs2(&[u16], start_index)
Match { range, captures: Vec<Option<Range<usize>>>, named_groups() }
```

`no_opt` is always `false` (we let `regress` apply its own optimizations).
The `y` (sticky) flag is *not* forwarded: we run the matcher unsticky and
filter at `regexp.rs:778` (`!flags.sticky() || matched.start() == start_index`).

### Pattern normalization

Before reaching `regress`, every source pattern goes through
`normalize_backend_pattern` (`regexp.rs:306`). It performs:

- **Unknown-script substitution** (`regexp.rs:315-334`) — `\p{Script=Unknown}`
  and friends are rewritten into a hand-built set, because `regress` rejects
  the spec name.
- **Legacy identity-escape repair** (`normalize_legacy_identity_escapes`,
  `regexp.rs:343`) — without `/u` we have to convert legacy quantifiable
  assertions, escape stray `]{}`, and let braced quantifiers through.
- **Astral expansion to UCS-2** (`expand_astral_source_for_ucs2`,
  `regexp.rs:628`) — without `/u`, supplementary-plane code points are
  expanded to surrogate-pair literal sequences so the UCS-2 matcher does
  the right thing.

These three passes only exist because the backend is generic-Unicode rather
than ECMA-262-specific. They are pure string rewriting today (more
allocation than they need to be), and they would mostly disappear in an
ECMA-262-native engine.

### Fast-pattern shims

`RegExpFastPattern` (`regexp.rs:271-304`) enumerates 30 hand-recognized
patterns that bypass `regress` entirely and run as direct code-unit scans:

- **Spec-gap shims** (10 variants) — duplicate-named backreferences,
  `(?i:...)`/`(?-i:...)` scoped modifiers around backrefs, word-boundary
  scoped modifiers, scoped Unicode property classes. These exist because
  `regress` does not implement those ES features at parity.
- **Linear-scan accelerators** (~20 variants) — anchored ASCII / digit /
  hex / whitespace / word runs (`^[0-9]+$`, `^\s+$`, etc.) and unanchored
  single-class versions (`\d`, `\s`, `\w`).

The shims live in `regexp.rs:786-928` (the bypass dispatch) with detection
in `regexp.rs:1124-1181`. The pay-off is large: an anchored ASCII-digit run
match becomes a hand-rolled byte loop with no NFA construction, no backend
allocation, and a single `RegExpMatchRecord` write.

### Compilation and caching

- **Per-`RegExp`-object compilation.** `RegExpPayload::compile`
  (`regexp.rs:730`) parses flags, normalizes the pattern, runs fast-pattern
  detection, and constructs `regress::Regex`. The result is stored on the
  RegExp object and reused for the lifetime of that object.
- **Literal cache.** A separate compiled-literal cache exists at the runtime
  level — the `runtime.regexp-literal-cache` snapshot in
  `tools/lyng-js-bench/src/runtime.rs:519` confirms it is observable and
  reports retained bytes. Literal `RegExp` evaluation reuses the compiled
  payload across script invocations.
- **No compile cache for `new RegExp(str, flags)`** — every dynamic
  construction re-runs the full pipeline (`regexp_compile_builtin`,
  `regexp.rs:1409`).
- **No compile cache for implicit conversions.** `String.prototype.match(str)`
  allocates a fresh `RegExp` from `str` on every call
  (`strings.rs:780-781`). Same for `search` (`strings.rs:902`) and
  `matchAll` (`strings.rs:851`).

### Match record materialization

`RegExpMatchRecord` (`regexp.rs:206-250`) holds UTF-16 code-unit ranges, not
materialized substrings. Conversion to JS values happens in
`build_regexp_match_result` (`regexp.rs:662-748`):

- One Array allocation for the capture tuple (`regexp.rs:669`).
- One JS string allocation per capture (`code_unit_range_value` ->
  `string_from_code_units`, `regexp.rs:685`) — even unused captures.
- Named-group object: one `OrdinaryObjectData` allocation
  (`regexp.rs:525`) plus an atom intern per name (`regexp.rs:529`),
  iterated even when the regex has zero named groups.
- `/d` indices: one outer Array plus one inner `[start, end]` Array per
  capture (`regexp.rs:570-660`).

### Coverage and conformance

Current ECMA-262 surface:

- Flags: `d`, `g`, `i`, `m`, `s`, `u`, `v`, `y` (with `y` emulated).
- Named capture groups, named backrefs, duplicate names (via shim).
- Scoped modifiers `(?i:...)` / `(?-i:...)` (via shim where the backend
  fails).
- Unicode property escapes under `/u` and `/v`.
- Lookahead and lookbehind (delegated to `regress`).

Test262 results in `reports/js/lyng-js/test262.md` show RegExp-namespaced
files passing, with the slow tail dominated by generated property-escape
fixtures (~1.4–1.7s per fixture for the heaviest character-class and
script-extension files). That latency is not a hot-path concern on its own,
but it is a useful proxy for how long `regress` spends on Unicode set
construction.

### Current measured baseline

`reports/js/lyng-js/bench.md` row `regexp-heavy.runtime` (defined at
`tools/lyng-js-bench/src/runtime.rs:257`, source built by
`regexp_heavy_runtime_workload` at `runtime.rs:1090`):

| Metric | Value |
| --- | --- |
| ns / work-unit | **34,102.08** |
| Workload | repeated global `exec`, sticky matches, named-capture replacement |

That single number is our headline regex performance figure today. The
workload exercises the slow path (named captures, sticky, replacement
callback), not the fast-pattern shims, so it is a fair indicator of
backend + materialization cost.

## Performance Hotspots

The hottest cost centers, in order of expected impact on
`regexp-heavy.runtime`:

### 1. Input-string UTF-16 materialization on every call

Every regex operation against a JS string allocates a fresh `Vec<u16>` of
the full input before the matcher runs.

- `string_ref_code_units` (`support.rs:1503`) reserves capacity then
  appends, taking either the Latin-1 (`support.rs:1538`) or UTF-16-LE
  (`support.rs:1544`) heap path.
- This buffer is rebuilt for each of `String.prototype.{match, search,
  replace, split, matchAll}` (`strings.rs:1158, 1209, 1233, 1433`), even
  when the *same* string is matched against multiple regexes in
  succession, and even when the input is Latin-1 (where every byte
  becomes a `u16` via `u16::from(byte)` — a 2× memory blow-up).
- For a 1 MiB input matched twice, that is 4 MiB of throw-away
  allocation per pair of operations.

There is no per-string conversion cache, no scratch-buffer pool, and no
Latin-1-native matcher path. `regress` itself does support `&[u8]` for
ASCII patterns, but we never reach that path.

### 2. Capture string allocation on every match

`build_regexp_match_result` (`regexp.rs:662-748`) eagerly allocates one
JS string per capture group per match. For `matchAll` against a global
regex with N matches and K captures, the cost is `O(N*K)` JS string
allocations regardless of whether the caller actually reads each
capture.

JSC and V8 keep capture results as lazy substring views on the input
string and only materialize on indexed access. We materialize
unconditionally.

### 3. Implicit `RegExp` construction in `String` methods

`str.match(pattern)` where `pattern` is a string allocates a fresh
`RegExp` and runs the compile pipeline every call (`strings.rs:780-781`).
There is no cache keyed on `(pattern, flags)`. Same for `search`,
`matchAll`. For interactive code that calls `s.match("foo")` in a tight
loop, every call pays full compile cost.

### 4. Named-capture object cost when no captures exist

`regexp.rs:516-568` allocates the named-groups bag and walks the captures
even when the compiled regex declared zero named groups. Atom interning
runs unconditionally per call.

### 5. Sticky-flag emulation

We forward `y`-flagged matches as un-sticky and filter post-hoc
(`regexp.rs:778`). This is correct but disables any anchor-driven
optimization the backend could otherwise apply, and on a non-match it
costs whatever scan distance `regress` performed before reporting a
later match we then discard.

### 6. Pattern-normalization rewrites are string-allocating

`normalize_backend_pattern` (`regexp.rs:306`) does up to 16 substring
substitutions via `String::replace` for unknown-script handling
(`regexp.rs:315-334`), then a character-by-character rewrite for legacy
escapes, then another rewrite for astral expansion. For dynamically
constructed patterns this is a real cost; for literals it is amortized
by the literal cache.

### 7. Named-group iteration without short-circuit

`regress`'s `named_groups()` returns names as `&str`, so we re-intern on
every call site even though the regex's name set is known at compile
time and could be pre-interned in `RegExpPayload`.

## Optimization Roadmap

The roadmap is staged so that each tier delivers a measurable win on
`regexp-heavy.runtime` without committing the team to the next tier. Tier
0 and Tier 1 stay on `regress`. Tier 2 vendors and modifies it. Tier 3
replaces it.

### Tier 0 — Low-hanging fruit (days, no API changes)

Each of these is a self-contained change behind the existing
`RegExpPayload` API. None requires touching string representation or
backend selection. Together they are estimated to take
`regexp-heavy.runtime` from ~34 µs/work-unit into the high-teens.

1. **Pre-intern named-group atoms in `RegExpPayload`.** Walk the names
   once at compile (`regexp.rs:730`) and store them as `Atom` alongside
   the backend handle. Skip the entire named-group object allocation
   when the set is empty (`regexp.rs:516`). _Expected: removes a
   per-call allocation + N atom lookups for every `exec`/`match`._

2. **Skip capture materialization until accessed.** Change the result
   array population in `build_regexp_match_result` (`regexp.rs:683`) to
   store lazy capture handles backed by `(StringRef, Range<usize>)` and
   resolve to JS strings on first read. The match Array shape stays
   identical; only the slot contents change. Most regex consumers read
   `[0]` and ignore the rest. _Expected: 1.5–3× on `matchAll` and any
   capture-heavy pattern._

3. **Thread-local `Vec<u16>` scratch buffer for input conversion.**
   Replace the per-call `Vec::with_capacity` in `string_ref_code_units`
   (`support.rs:1503`) with a per-thread reusable buffer (clear, not
   drop). Most regex paths drop the `Vec` immediately after the call.
   _Expected: eliminates a malloc/free pair per regex call against a
   string input._

4. **Cache RegExp by `(pattern, flags)` for `String` method coercions.**
   Add a small bounded LRU keyed on `(pattern, flags)` for the
   string-arg paths in `string_match_builtin` (`strings.rs:780`),
   `string_search_builtin` (`strings.rs:902`), and `string_match_all`
   (`strings.rs:851`). Strict spec ordering still requires a fresh
   `RegExp` *object* each call, but the underlying compiled
   `RegExpPayload` can be shared. _Expected: removes compile-time cost
   from `s.match("foo")` patterns common in user code._

5. **Hoist `normalize_backend_pattern` substitution table** into a
   single regex-driven pass instead of 16 sequential `String::replace`
   calls (`regexp.rs:315-334`). _Expected: a small but measurable win
   on `new RegExp(str)` paths._

6. **Cheap fast-pattern extensions.** Add anchored decimal-number
   (`^-?\d+(\.\d+)?$`), C-identifier (`^[A-Za-z_][A-Za-z0-9_]*$`), and
   trim (`^\s+|\s+$`) to the fast-pattern table. These are common
   enough in real code to be worth the maintenance cost. _Expected:
   bypasses `regress` on three more high-frequency cases._

### Tier 1 — Backend-adjacent wins (weeks, still on `regress`)

These changes either pre-process inputs or post-process results in ways
the backend cannot do, and require deeper integration with the string
representation. They aim to roughly halve `regexp-heavy.runtime` again.

7. **Latin-1 fast matcher for ASCII-only patterns.** When both the
   pattern and the input are Latin-1 (the input check is cheap given
   the existing `view.latin1_bytes()` branch at `support.rs:1538`) and
   the pattern compiles to a Latin-1-safe NFA (no `\u{...}`, no
   `\p{...}`, no case-fold over non-ASCII), run a Latin-1 byte matcher
   in-house for the simple subset and fall back to `regress` otherwise.
   This is essentially a generalized fast-pattern dispatch with a real
   NFA executor instead of a pattern enum.

8. **Pre-compute UTF-16 view on the StringRef.** Cache the converted
   `Box<[u16]>` lazily on the heap-resident string the first time it is
   handed to a regex op. Invalidates naturally because JS strings are
   immutable. _Expected: collapses the cost of multiple regex
   operations against the same string to a single conversion._

9. **Native sticky support without backend cooperation.** Anchor the
   pattern at compile time when `y` is set by prepending an internal
   marker that maps to "must match at offset" inside our normalization
   layer, and skip the post-filter in `regexp.rs:778`. This still
   leaves room for the backend to bail out earlier on non-match.

10. **Literal-prefix prefilter.** Most real-world regexes start with a
    literal prefix (`^foo`, `https://`, `var\s+`). Extract longest
    literal prefix at compile (`regexp.rs:730`) and use a Boyer-Moore
    or two-way scan to position the matcher; skip backend invocation
    entirely when no prefix occurrence is found. This is a single-call
    optimization but it changes the asymptotic constant for unanchored
    matches.

11. **Capture-buffer pool.** `regress` returns a `Vec<Option<Range>>`
    for captures; we can reuse a single per-thread buffer across calls
    if we own the iteration loop in `matchAll`/`replace` (`regexp.rs`
    around `1158`). _Expected: cuts `matchAll` allocator pressure._

12. **Drop UCS-2 astral expansion when the pattern has no astral
    code points.** `expand_astral_source_for_ucs2` (`regexp.rs:628`)
    runs unconditionally for non-`/u` patterns; cheap pre-scan can
    skip it.

### Tier 2 — Vendor and modify `regress` (1–2 quarters)

At this point we have squeezed what we can out of the integration
boundary. The next improvements need backend changes:

- **Vendor `regress`** as a `crates/lyng-js/regex` member. Drop unused
  features (Wasm builds, non-UTF-16 paths we don't take), simplify the
  flag surface to ECMA-262, and re-export only what we need.
- **Add native ECMA-262 features** the upstream lacks: scoped
  modifiers, duplicate named groups, `\p{Script=Unknown}`. This kills
  the corresponding fast-pattern shims (`regexp.rs:271-284`), reducing
  the maintenance surface in `objects/src/regexp.rs` by an estimated
  20-30%.
- **Streaming string view interface.** Replace
  `find_from_utf16(&[u16], start)` with a trait that lets our heap
  string types be matched in place (one Latin-1 chunk, one UTF-16
  chunk, or a rope walk) without ever building the `Vec<u16>` from
  Tier 0 step 3. This is the biggest single perf win on the table —
  it removes the dominant allocation entirely for large inputs.
- **Compiled-bytecode cache on RegExp literals.** Persist the compiled
  matcher bytecode alongside the JS bytecode template so a re-evaluated
  literal in a hot script does not re-walk the parser at all. Today
  the literal *payload* is cached but parser/compiler work is repeated
  on cold-cache misses; with our own backend we can cache the bytecode
  itself.
- **Bytecode-VM matcher for the linear-time subset.** Most patterns in
  practice (no backreferences, no nested unbounded quantifiers) admit
  a linear-time NFA-simulation matcher (Thompson construction +
  bitset state). Implement this as a fast tier and fall back to the
  backtracker only when the pattern's features require it. RE2,
  Hyperscan, and (since 2021) V8's Irregexp interpreter all do this;
  it is the single biggest worst-case-input safety win as well as a
  perf win on the common case.

### Tier 3 — Lightning fast (multi-quarter)

These are the moves that take Lyng from "competitive with bare
`regress`" to "competitive with V8/JSC on regex". They should not be
started before Tier 2 stabilizes.

- **JIT regex bytecode to machine code.** Both V8 and JSC compile hot
  regexes to native code. This should ride on top of whatever JIT
  substrate is built per `jit-backend-research.md` (Cranelift spike
  recommended there). The regex bytecode VM is a much smaller and
  safer first JIT customer than full JS bytecode — fixed register
  layout, no GC, no deopt — so it is a natural early native target.
- **SIMD-accelerated character-class scans.** `[A-Za-z0-9_]` and the
  built-in classes (`\w`, `\s`, `\d`) admit aligned-load + shuffle
  classification on aarch64 NEON and x86 AVX2. `memchr` and `aho-
  corasick` already publish reusable SIMD primitives we can wrap; the
  scan loops in fast-pattern handlers (`regexp.rs:786-928`) and the
  bytecode VM's literal-prefix scan are the obvious customers.
- **Lazy DFA construction (RE2-style).** For patterns that hit the
  linear-time tier and run hot, build a DFA on demand and cache it
  per-pattern. RE2's hybrid NFA/DFA matcher is the reference
  implementation; we would adapt rather than copy because our
  ECMA-262 surface (capturing groups, lookbehind) needs care in DFA
  form.
- **Multi-pattern prefiltering.** When the same script holds many
  literal-prefixed regexes (lints, syntax highlighters, parser
  generators), an Aho–Corasick prefilter dispatches in one scan. Not
  a near-term need but a real ceiling.

## Risks and Constraints

- **Test262 is the floor.** Every change in Tier 0–1 must hold the
  current pass rate. The fast-pattern shims exist precisely because
  small wins regressed conformance; the same care is required when
  extending them.
- **Vendoring `regress` (Tier 2) is a maintenance commitment.** We
  inherit upstream bug fixes manually thereafter. The judgment here
  matches the JIT-backend doc's stance on Cranelift: do not vendor
  until the integration shape is settled by Tier 0/1 work.
- **A bytecode-VM rewrite (Tier 2) is months of correctness work.**
  Backreferences, named groups, `/v` set operations, and lookbehind
  each have spec edge cases that are easy to get subtly wrong. The
  tier-down to a backtracker for unsupported features must be
  watertight, with a fuzz harness that compares against `regress`.
- **JIT (Tier 3) is gated by the JS JIT substrate.** Per
  `jit-backend-research.md`, no native backend should land before
  tiering, safepoints, executable-memory policy, and feedback-vector
  contracts are documented. Regex JIT inherits all of those gates.

## Recommendation

Start Tier 0. Items 1, 2, 3, and 4 each independently target a
well-identified hot path in today's profile, are reversible, and do not
require any architectural decision the team has not already made. They
will move `regexp-heavy.runtime` materially without committing us to a
custom matcher.

Defer the question of "do we replace `regress`" until after Tier 1.
That order preserves the option to stay on the upstream backend if the
post-Tier-1 numbers are competitive, and sharpens the case for vendoring
if they are not. The Tier 2 plan is the one that captures the largest
single win (streaming string-view interface), so the decision deserves
real data behind it.

The lightning-fast end state in Tier 3 should be revisited only after
Tier 2 ships. By then the JIT-backend research will have produced a
working substrate, the bytecode-VM matcher will have a year of
production data, and the cost/benefit on regex JIT will be answerable
from evidence rather than analogy.
