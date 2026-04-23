# Lyng JS Temporal Support Matrix

Source corpus: `testdata/test262/test/built-ins/Temporal/`

This document is the current-state scope and tail tracker for JS3's non-Intl `Temporal`
support. The core `Temporal` implementation landed during `6A2`; remaining misses are now
tracked under the active shared `6H` Temporal conformance tail.

Status meanings:

- `implemented`: the family exists in JS3 and is part of the supported core surface
- `shared 6H tail`: open Test262 misses still exist for the current family inside the shared
  `6H` Temporal conformance tail
- `out of core scope`: locale-sensitive or Intl-owned behavior that does not block core completion

Rules:

- `built-ins/Temporal/*` remains part of the core Phase 6 completion surface.
- Generic `unsupported feature: Temporal` skips are not allowed in the live JS3 reports.
- Every remaining Temporal miss must become either a real bug or an explicit exclusion in
  `reports/js/lyng-js/test262-exclusions.txt`.
- Host ownership is limited to live clock and named time-zone resolution. Parsing, balancing,
  canonicalization, comparison, rounding, and brand checks stay engine-owned.
- The current tail is tracked conservatively as a shared `6H` family-wide tail until the
  checked-in reports are re-baselined down to per-family clear rows.

## Family Matrix

| Family | Test262 rows | Implementation | Tail | Notes |
| --- | ---: | --- | --- | --- |
| `Temporal` root namespace | `5` | `implemented` | `shared 6H tail` | Namespace shape and property metadata are part of the live engine surface. |
| `Temporal.Now` | `66` | `implemented` | `shared 6H tail` | Uses host clock and default time-zone hooks through `lyng-js-host`. |
| `Temporal.Instant` | `465` | `implemented` | `shared 6H tail` | Engine-owned epoch-nanosecond parsing, arithmetic, rounding, and serialization. |
| `Temporal.Duration` | `533` | `implemented` | `shared 6H tail` | Engine-owned duration records, balancing, rounding, and comparison semantics. |
| `Temporal.PlainDate` | `646` | `implemented` | `shared 6H tail` | Engine-owned ISO civil-date parsing and calendar-sensitive core semantics. |
| `Temporal.PlainDateTime` | `767` | `implemented` | `shared 6H tail` | Engine-owned civil date-time parsing, arithmetic, comparison, and ISO serialization. |
| `Temporal.PlainTime` | `493` | `implemented` | `shared 6H tail` | Engine-owned clock-time parsing, balancing, rounding, and comparison semantics. |
| `Temporal.PlainYearMonth` | `505` | `implemented` | `shared 6H tail` | Engine-owned ISO year-month parsing and property-bag normalization. |
| `Temporal.PlainMonthDay` | `199` | `implemented` | `shared 6H tail` | Engine-owned ISO month-day parsing, normalization, and serialization. |
| `Temporal.ZonedDateTime` | `897` | `implemented` | `shared 6H tail` | Requires host-backed time-zone resolution while keeping zone/civil semantics in-engine. |
| Intl-sensitive locale behavior | n/a | `out of core scope` | n/a | Locale formatting and other ECMA-402 behavior do not block core JS3 completion. |

## Checked-In Exclusions

No permanent core-scope Temporal exclusions are claimed by this matrix.

If a Temporal row cannot be closed inside the active `6H` tail, it must be added to
`reports/js/lyng-js/test262-exclusions.txt` with:

- an exact `path` or narrow `suite` pattern
- the concrete reason it is excluded
- the active owner when the reason is a dependency rather than a scope cut

## Host Touchpoints

The following Temporal families may call `lyng-js-host`:

- `Temporal.Now.*`
- `Temporal.ZonedDateTime.*`
- any `from` or conversion path that resolves named time zones against a live host zone database

The following semantics remain in JS3 runtime code even when they consume host results:

- ISO string parsing and annotation parsing
- property-bag normalization
- balancing and rounding
- disambiguation mapping and error conversion
- brand checks and prototype semantics
- serialization and comparison
