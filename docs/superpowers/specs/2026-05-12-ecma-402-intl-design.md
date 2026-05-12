# ECMA-402 Intl Support Design

Date: 2026-05-12

## Goal

Add default-on ECMA-402 internationalization support to Lyng JS with a full roadmap for
the stable published ECMA-402 surface plus TC39 Intl proposals that are Stage 3 or later.

The implementation target is ECMA-402 12th edition, June 2025, with Stage 3+ Intl
proposals tracked as explicit follow-up scope as they exist at implementation time. The
current TC39 ECMA-402 proposals list has no active Stage 3 proposal rows, so the first
implementation plan should treat the stable edition as required and keep a tracking issue
ready for future Stage 3 movement.

## Non-Goals

- Do not add a Cargo feature gate for Intl. Intl is part of the normal default realm once
  the foundation lands.
- Do not add placeholder `en-US`-only behavior as the architecture. ICU4X/CLDR-backed
  behavior is the chosen direction.
- Do not move Intl semantics into the VM or compiler. VM/compiler work is only in scope if
  an existing locale-sensitive builtin entrypoint needs to call Intl-aware operations.
- Do not hide Intl work behind broad Test262 skips after a constructor family is
  implemented.

## Architecture

ECMA-402 becomes part of the default Lyng JS realm. The implementation still keeps a hard
internal boundary so Intl data and algorithms do not leak into VM hot paths.

### `lyng-js-host`

Define a small data-provider boundary only if ICU4X needs runtime provider plumbing. This
is not the semantic owner of Intl. It exists to keep locale data loading/configuration
explicit and embeddable.

### `lyng-js-env`

Own per-agent Intl data/provider state if runtime provider state is needed. Agents already
own realm/runtime substrate, atom tables, heaps, and shared tables, so immutable default
locale, available locale summaries, and provider handles fit here.

### `lyng-js-objects`

Add typed Intl object payload records for formatter instances and `Intl.Locale`, similar
in spirit to the existing Temporal payload records. These records store resolved
internal-slot state in compact structs rather than ad hoc guest-visible properties.

### `lyng-js-ops`

Own reusable ECMA-402 abstract operations:

- locale list canonicalization
- locale negotiation and matching
- Unicode extension key processing
- option extraction and option validation
- locale, numbering system, currency, unit, calendar, collation, and time-zone validation
- shared formatter setup and resolved-options helpers
- ICU4X glue that is semantic rather than bootstrap-only

Builtins should call these helpers instead of duplicating option and locale algorithms.

### `lyng-js-builtins`

Own the `Intl` global object, constructors/prototypes, builtin metadata, descriptor
installation, realm builtin caches, and native dispatch. Each constructor family should
have focused metadata, family installer, and dispatch modules.

### `tools/lyng-js-test262` and `reports/js/lyng-js`

Move from whole-suite `intl402/*` exclusion to intentional subset tracking. The broad
manifest exclusion should be narrowed only as constructor families become meaningful, so
Intl progress is visible without hiding failures.

## Milestones

1. Foundation and test harness
   - Add Intl architecture docs.
   - Add ICU4X dependency/data-provider foundation.
   - Add internal-slot payload model.
   - Add `Intl` bootstrap skeleton.
   - Add shared ECMA-402 option/locale operations.
   - Add Test262 subset/report plumbing for `intl402`.

2. Locale identity and negotiation
   - Implement `Intl.Locale`.
   - Implement `Intl.getCanonicalLocales`.
   - Implement `Intl.supportedValuesOf`.
   - Implement canonical BCP 47 handling, Unicode extension keys, likely subtags,
     supported locale lists, and default locale behavior.

3. Core formatter constructors
   - Implement `Intl.Collator`.
   - Implement `Intl.NumberFormat`.
   - Implement `Intl.DateTimeFormat`.
   - Include bound format functions, `resolvedOptions`, `supportedLocalesOf`, parts APIs,
     range APIs where required, and normative optional chain construction behavior.

4. Rules and text/list display constructors
   - Implement `Intl.PluralRules`.
   - Implement `Intl.RelativeTimeFormat`.
   - Implement `Intl.ListFormat`.
   - Implement `Intl.DisplayNames`.

5. Segmentation and duration
   - Implement `Intl.Segmenter`, segment iterator objects, and segments objects.
   - Implement `Intl.DurationFormat`.

6. Locale-sensitive ECMA-262 and Temporal hooks
   - Replace fallback behavior in locale-sensitive Number, BigInt, String, Date, Array,
     TypedArray, and Temporal methods with ECMA-402-backed paths.

7. Conformance closure
   - Run and track `intl402` Test262 subsets by constructor family.
   - Remove the broad `intl402/*` exclusion only when the whole suite is expected to be
     meaningful.

## dcat Epic Breakdown

Create one priority-1 epic:

- Add default-on ECMA-402 Intl support to Lyng JS

Create these child issues:

1. Design ECMA-402 architecture and verification workflow
2. Add ICU4X/CLDR Intl data provider foundation
3. Add Intl object bootstrap, metadata, and internal-slot payloads
4. Implement ECMA-402 locale parsing and negotiation operations
5. Implement Intl.Locale and Intl.getCanonicalLocales
6. Implement Intl.supportedValuesOf
7. Implement Intl.Collator
8. Implement Intl.NumberFormat
9. Implement Intl.DateTimeFormat
10. Implement Intl.PluralRules
11. Implement Intl.RelativeTimeFormat
12. Implement Intl.ListFormat
13. Implement Intl.DisplayNames
14. Implement Intl.Segmenter
15. Implement Intl.DurationFormat and Stage 3+ duration behavior
16. Wire locale-sensitive Number, BigInt, String, Date, Array, and TypedArray methods
17. Wire Temporal locale-sensitive formatting to Intl
18. Support normative optional legacy Intl constructor chaining
19. Unskip and report Intl Test262 subsets incrementally
20. Close ECMA-402 conformance gaps and remove broad intl402 exclusion

Foundation issues 1-4 block most constructor work. `Intl.Locale` and canonicalization
block formatter correctness. Formatter issues can then run in parallel by family.
Locale-sensitive ECMA-262 and Temporal hooks depend on the relevant formatter families.
Test262 unskip/report work runs throughout, with final broad exclusion removal waiting for
conformance closure.

## Verification Strategy

- Run focused crate tests for touched crates first:
  - `cargo test -p lyng-js-builtins`
  - `cargo test -p lyng-js-ops`
  - `cargo test -p lyng-js-objects`
  - `cargo test -p lyng-js-env`
  - `cargo test -p lyng-js-tests`
- Run targeted Intl Test262 subsets as they become runnable:
  - `cargo run --release -p lyng-js-test262 -- --filter intl402/Locale --report /tmp/lyng-js-test262-intl-locale.md -j 4`
  - `cargo run --release -p lyng-js-test262 -- --filter intl402/NumberFormat --report /tmp/lyng-js-test262-intl-numberformat.md -j 4`
  - `cargo run --release -p lyng-js-test262 -- --filter intl402/DateTimeFormat --report /tmp/lyng-js-test262-intl-datetimeformat.md -j 4`
- Run whole `intl402` sweeps before removing broad exclusions:
  - `cargo run --release -p lyng-js-test262 -- --filter intl402 --report /tmp/lyng-js-test262-intl402.md -j 12`
- Run Clippy before marking implementation issues in review:
  - `cargo clippy --all-targets --all-features -- -W clippy::pedantic -W clippy::nursery`

## Open Tracking Rule

At the start of implementation and before final conformance closure, check the official
TC39 ECMA-402 proposals list. Add or update child dcat issues for any active Stage 3+
Intl proposal that is not already included in the stable ECMA-402 target.

## References

- ECMA-402 12th edition, June 2025: https://402.ecma-international.org/12.0/
- Current ECMA-402 draft: https://tc39.es/ecma402/
- TC39 ECMA-402 proposals: https://github.com/tc39/proposals/blob/main/ecma402/README.md
- ICU4X Rust docs: https://docs.rs/icu
