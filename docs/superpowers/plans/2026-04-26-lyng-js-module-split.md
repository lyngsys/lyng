# Lyng JS Oversized Module Split Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reduce the largest Lyng JS source files into ownership-focused modules without changing public APIs, crate boundaries, builtin identities, or runtime behavior.

**Architecture:** Start with mechanically separable representation and table data, then move semantic files only after direct owner-layer tests are in place. Each split keeps the parent module as the public orchestration surface and moves cohesive domains into child files re-exported through the existing crate API.

**Tech Stack:** Rust workspace crates under `crates/lyng-js/*`, dcat issue `lyng-kb1k`, Cargo unit tests, focused crate checks, and targeted Test262 slices for semantic dispatch/bootstrap moves.

---

## File Structure

- `crates/lyng-js/types/src/lib.rs`: remains the thin public facade for copyable runtime-facing types.
- `crates/lyng-js/types/src/ids.rs`: owns typed runtime handles, native function lane IDs, and well-known symbol IDs.
- `crates/lyng-js/types/src/builtin_ids.rs`: owns stable builtin ID namespace constants and re-exports constructor helpers.
- `crates/lyng-js/types/src/builtin_ids/*.rs`: owns builtin helper constructors by namespace or object family.
- `crates/lyng-js/types/src/builtin_ids/binary_data/*.rs`: owns binary-data, object-reflection, JSON, Proxy, Reflect, and Atomics helper constructors by builtin family.
- `crates/lyng-js/types/src/builtin_ids/collections/*.rs`: owns collection and weak/finalization helper constructors by builtin family.
- `crates/lyng-js/types/src/builtin_ids/core_ids/*.rs`: owns core public builtin helper constructors by builtin family.
- `crates/lyng-js/types/src/builtin_ids/disposal/*.rs`: owns explicit-resource-management helper constructors by builtin family.
- `crates/lyng-js/types/src/builtin_ids/internal/*.rs`: owns internal builtin helper constructors by VM/runtime owner.
- `crates/lyng-js/types/src/builtin_ids/promises/*.rs`: owns promise, aggregate-error, async-function, and async-generator helper constructors.
- `crates/lyng-js/types/src/builtin_ids/temporal/*.rs`: owns Temporal builtin helper constructors by Temporal object family.
- `crates/lyng-js/types/src/builtin_ids/typed_arrays/*.rs`: owns abstract typed-array and typed-array instance helper constructors.
- `crates/lyng-js/types/src/marker.rs`: owns `TypeOwnershipMarker`.
- `crates/lyng-js/builtins/src/public/metadata.rs`: remains the public metadata lookup facade.
- `crates/lyng-js/builtins/src/public/metadata/*.rs`: owns metadata rows by builtin family.
- `crates/lyng-js/builtins/src/public/metadata/core/*.rs`: owns core metadata row tables by builtin family.
- `crates/lyng-js/builtins/src/public/metadata/temporal/*.rs`: owns Temporal metadata lookup by Temporal object family.
- `crates/lyng-js/builtins/src/public/families/binary_data.rs`: remains the binary-data family bootstrap facade.
- `crates/lyng-js/builtins/src/public/families/binary_data/*.rs`: owns binary-data bootstrap lookup, installation, and descriptor-table wiring.
- `crates/lyng-js/builtins/src/public/dispatch/binary_data.rs`: remains the binary-data public dispatch facade.
- `crates/lyng-js/builtins/src/public/dispatch/binary_data/*.rs`: owns ArrayBuffer, DataView, Atomics, and typed-array dispatch families.
- `crates/lyng-js/builtins/src/public/temporal.rs`: remains the Temporal public bootstrap coordinator while object-family installers move into child modules.
- `crates/lyng-js/builtins/src/public/temporal/*.rs`: owns Temporal object-family public bootstrap wiring.
- `crates/lyng-js/builtins/src/public/dispatch/temporal.rs`: planned follow-on split into Temporal object-family dispatch modules after helper visibility is audited.
- `crates/lyng-js/vm/src/vm/builtin_dispatch.rs`: planned follow-on split by VM builtin owner after builtin/public dispatch is stable.
- `crates/lyng-js/gc/src/arena.rs`: planned follow-on split for allocation, tracing, weak/finalization, and backing-store ownership after direct GC tests are identified.

## Task 1: Split `lyng-js-types` Facade

**Files:**
- Modify: `crates/lyng-js/types/src/lib.rs`
- Create: `crates/lyng-js/types/src/ids.rs`
- Create: `crates/lyng-js/types/src/builtin_ids.rs`
- Create: `crates/lyng-js/types/src/builtin_ids/*.rs`
- Create: `crates/lyng-js/types/src/marker.rs`

- [x] Move `define_runtime_id!`, handle declarations, `NativeFunctionId`, and `WellKnownSymbolId` into `ids.rs`.
- [x] Move builtin namespace constants and `is_*_builtin` classifiers into `builtin_ids.rs`.
- [x] Move builtin helper constructors into `builtin_ids/*.rs` by namespace or object family.
- [x] Further split `builtin_ids/binary_data.rs` into `builtin_ids/binary_data/*.rs` by builtin family.
- [x] Further split `builtin_ids/collections.rs` into `builtin_ids/collections/*.rs` by builtin family.
- [x] Further split `builtin_ids/core_ids.rs` into `builtin_ids/core_ids/*.rs` by core family.
- [x] Further split `builtin_ids/disposal.rs` into `builtin_ids/disposal/*.rs` by builtin family.
- [x] Further split `builtin_ids/internal.rs` into `builtin_ids/internal/*.rs` by VM/runtime owner.
- [x] Further split `builtin_ids/promises.rs` into `builtin_ids/promises/*.rs` by builtin family.
- [x] Further split `builtin_ids/temporal.rs` into `builtin_ids/temporal/*.rs` by Temporal family.
- [x] Further split `builtin_ids/typed_arrays.rs` into `builtin_ids/typed_arrays/*.rs` by builtin family.
- [x] Move `TypeOwnershipMarker` into `marker.rs`.
- [x] Re-export the same public names from `lib.rs`.
- [x] Verify with `cargo test -p lyng-js-types`.

## Task 2: Split Public Builtin Metadata Tables

**Files:**
- Modify: `crates/lyng-js/builtins/src/public/metadata.rs`
- Create: `crates/lyng-js/builtins/src/public/metadata/core.rs`
- Create: `crates/lyng-js/builtins/src/public/metadata/core/*.rs`
- Create: `crates/lyng-js/builtins/src/public/metadata/binary_data.rs`
- Create: `crates/lyng-js/builtins/src/public/metadata/temporal.rs`
- Create: `crates/lyng-js/builtins/src/public/metadata/temporal/*.rs`

- [x] Keep `PublicBuiltinMetadataRow`, lookup helpers, and `public_builtin_metadata` in `metadata.rs`.
- [x] Move object/function/array/collection/weak-ref/object-reflection/text/regexp/date/primitive/module/language-support rows into `metadata/core.rs`.
- [x] Further split core metadata rows into `metadata/core/*.rs` by builtin family.
- [x] Move `ArrayBuffer`, `DataView`, typed-array, shared-memory, and Atomics rows into `metadata/binary_data.rs`.
- [x] Move Temporal metadata out of `public/temporal.rs` into `metadata/temporal.rs`.
- [x] Further split Temporal metadata lookup into `metadata/temporal/*.rs` by Temporal family.
- [x] Verify with `cargo test -p lyng-js-builtins --lib`.

## Task 3: Split Temporal Public Bootstrap By Object Family

**Files:**
- Modify: `crates/lyng-js/builtins/src/public/temporal.rs`
- Create: `crates/lyng-js/builtins/src/public/temporal/instant.rs`
- Create: `crates/lyng-js/builtins/src/public/temporal/duration.rs`
- Create: `crates/lyng-js/builtins/src/public/temporal/plain_date.rs`
- Create: `crates/lyng-js/builtins/src/public/temporal/plain_time.rs`
- Create: `crates/lyng-js/builtins/src/public/temporal/plain_date_time.rs`
- Create: `crates/lyng-js/builtins/src/public/temporal/plain_year_month.rs`
- Create: `crates/lyng-js/builtins/src/public/temporal/plain_month_day.rs`
- Create: `crates/lyng-js/builtins/src/public/temporal/zoned_date_time.rs`

- [x] Use the existing Temporal bootstrap smoke coverage through the integration surface.
- [x] Extract one object family at a time into a small installer that receives the already allocated Temporal namespace, prototypes, root shape, realm, and shared builtin prototypes.
- [x] Extract `Temporal.Instant` public bootstrap wiring into `public/temporal/instant.rs`.
- [x] Extract `Temporal.Duration` public bootstrap wiring into `public/temporal/duration.rs`.
- [x] Extract `Temporal.PlainDate` public bootstrap wiring into `public/temporal/plain_date.rs`.
- [x] Extract `Temporal.PlainTime` public bootstrap wiring into `public/temporal/plain_time.rs`.
- [x] Extract `Temporal.PlainDateTime` public bootstrap wiring into `public/temporal/plain_date_time.rs`.
- [x] Extract `Temporal.PlainYearMonth` public bootstrap wiring into `public/temporal/plain_year_month.rs`.
- [x] Extract `Temporal.PlainMonthDay` public bootstrap wiring into `public/temporal/plain_month_day.rs`.
- [x] Extract `Temporal.ZonedDateTime` public bootstrap wiring into `public/temporal/zoned_date_time.rs`.
- [x] Preserve constructor/prototype allocation order and `PublicRealmBuiltins` field assignment order.
- [x] Verify with `cargo test -p lyng-js-builtins --lib` and `cargo test -p lyng-js-tests temporal`.

## Task 4: Split Temporal Dispatch By Object Family

**Files:**
- Modify: `crates/lyng-js/builtins/src/public/dispatch/temporal.rs`
- Create: `crates/lyng-js/builtins/src/public/dispatch/temporal/instant.rs`
- Create: `crates/lyng-js/builtins/src/public/dispatch/temporal/duration.rs`
- Create: `crates/lyng-js/builtins/src/public/dispatch/temporal/plain_date.rs`
- Create: `crates/lyng-js/builtins/src/public/dispatch/temporal/plain_time.rs`
- Create: `crates/lyng-js/builtins/src/public/dispatch/temporal/plain_date_time.rs`
- Create: `crates/lyng-js/builtins/src/public/dispatch/temporal/plain_year_month.rs`
- Create: `crates/lyng-js/builtins/src/public/dispatch/temporal/plain_month_day.rs`
- Create: `crates/lyng-js/builtins/src/public/dispatch/temporal/zoned_date_time.rs`
- Create: `crates/lyng-js/builtins/src/public/dispatch/temporal/now.rs`

- [x] Add focused Temporal dispatch tests or Test262 slices for the families being moved.
- [x] Move shared parsing, rounding, calendar, and time-zone helpers into private child modules before moving family algorithms.
  - [x] Move constructor/prototype, integer/property, month-code parsing, overflow, and ISO calendar validation helpers needed by `PlainMonthDay` and `PlainYearMonth` into `dispatch/temporal/support.rs`.
  - [x] Move shared `PlainTime` time-field parsing, construction, conversion, nanosecond, and precision formatting helpers into `dispatch/temporal/support.rs`.
  - [x] Move shared Instant string-precision, exact-time rounding, exact-duration conversion, string-option, and civil date-time formatting helpers into `dispatch/temporal/support.rs`.
  - [x] Move remaining duration/date-time rounding helpers into the owning duration and PlainDateTime modules, and move time-zone helpers with ZonedDateTime.
- [x] Extract `Instant` dispatch, allocation, and builtin algorithms into `dispatch/temporal/instant.rs`; keep `Temporal.Now` in the parent dispatcher for now.
- [x] Extract `Duration` dispatch, builtin algorithms, and cross-family duration conversion/rounding helpers into `dispatch/temporal/duration.rs`.
- [x] Extract `PlainMonthDay` dispatch, bag-field parsing, allocation, and builtin algorithms into `dispatch/temporal/plain_month_day.rs`.
- [x] Extract `PlainYearMonth` dispatch, bag-field parsing, allocation, and builtin algorithms into `dispatch/temporal/plain_year_month.rs`.
- [x] Extract `PlainTime` dispatch, allocation, and builtin algorithms into `dispatch/temporal/plain_time.rs`.
- [x] Extract `PlainDate` dispatch, allocation, bag-field parsing, and builtin algorithms into `dispatch/temporal/plain_date.rs`.
- [x] Extract `PlainDateTime` dispatch, allocation, date-time rounding/difference helpers, and builtin algorithms into `dispatch/temporal/plain_date_time.rs`.
- [x] Extract `ZonedDateTime` dispatch, allocation, time-zone parsing helpers, and builtin algorithms into `dispatch/temporal/zoned_date_time.rs`.
- [x] Extract `Temporal.Now` dispatch and builtin algorithms into `dispatch/temporal/now.rs`.
- [x] Expose only family dispatch entrypoints back to `dispatch/temporal.rs`.
  - [x] Route all Temporal object families and `Temporal.Now` through family entrypoints; keep allocators and cross-family helpers visible only where conversions require them.
- [ ] Verify each family with `cargo test -p lyng-js-builtins --lib` and `cargo run --release -p lyng-js-test262 -- --filter built-ins/Temporal/<Family> --report /tmp/lyng-js-test262-temporal-<family>.md -j 4`.
  - [x] Verified `Instant` with `cargo test -p lyng-js-builtins --lib`, `cargo test -p lyng-js-tests temporal::instant`, `cargo test -p lyng-js-tests temporal`, and Test262 `built-ins/Temporal/Instant` (906 passed, 24 failed, 0 panicked).
  - [x] Verified `Duration` with `cargo test -p lyng-js-builtins --lib`, `cargo test -p lyng-js-tests temporal::duration`, and Test262 `built-ins/Temporal/Duration` (868 passed, 198 failed, 0 panicked).
  - [x] Verified `PlainMonthDay` with `cargo test -p lyng-js-builtins --lib`, `cargo test -p lyng-js-tests temporal::plain_month_day`, `cargo test -p lyng-js-tests temporal::plain_shared`, `cargo test -p lyng-js-tests temporal::plain_date::temporal_plain_date_converts_to_partial_plain_dates`, and Test262 `built-ins/Temporal/PlainMonthDay` (392 passed, 6 failed, 0 panicked).
  - [x] Verified `PlainYearMonth` with `cargo test -p lyng-js-builtins --lib`, `cargo test -p lyng-js-tests temporal::plain_year_month`, `cargo test -p lyng-js-tests temporal::plain_shared`, `cargo test -p lyng-js-tests temporal::plain_date::temporal_plain_date_converts_to_partial_plain_dates`, and Test262 `built-ins/Temporal/PlainYearMonth` (954 passed, 56 failed, 0 panicked).
  - [x] Verified `PlainTime` with `cargo test -p lyng-js-builtins --lib`, `cargo test -p lyng-js-tests temporal::plain_time`, `cargo test -p lyng-js-tests temporal::plain_date_time`, `cargo test -p lyng-js-tests temporal::zoned_date_time`, and Test262 `built-ins/Temporal/PlainTime` (974 passed, 12 failed, 0 panicked).
  - [x] Verified `PlainDate` with `cargo test -p lyng-js-builtins --lib`, `cargo test -p lyng-js-tests temporal::plain_date`, and Test262 `built-ins/Temporal/PlainDate` (1244 passed, 48 failed, 0 panicked).
  - [x] Verified `PlainDateTime` with `cargo test -p lyng-js-builtins --lib`, `cargo test -p lyng-js-tests temporal::plain_date_time`, and Test262 `built-ins/Temporal/PlainDateTime` (1378 passed, 156 failed, 0 panicked).
  - [x] Verified `ZonedDateTime` with `cargo test -p lyng-js-builtins --lib`, `cargo test -p lyng-js-tests temporal::zoned_date_time`, and Test262 `built-ins/Temporal/ZonedDateTime` (1482 passed, 312 failed, 0 panicked).
  - [x] Verified `Temporal.Now` with `cargo test -p lyng-js-builtins --lib` and `cargo test -p lyng-js-tests temporal_now` (4 passed).

## Task 5: Split VM Builtin Dispatch By Runtime Owner

**Files:**
- Modify: `crates/lyng-js/vm/src/vm/builtin_dispatch.rs`
- Create: `crates/lyng-js/vm/src/vm/builtin_dispatch/*.rs`

- [x] Map existing match arms to owning semantic areas: public builtins, internal helpers, dynamic compilation, generators/async, promises/jobs, and object/private-field helpers.
- [x] Move `import.meta`, dynamic import capability creation, import-attribute normalization, dynamic-import job settlement, and dynamic-import error conversion into `builtin_dispatch/dynamic_import.rs`.
- [x] Move object/class internal helpers for method/accessor definition, home/private environment binding, public instance field keys, private fields, `super` property access, and `super()` construction into `builtin_dispatch/class_helpers.rs`.
- [x] Move bound-function construction, array-like bound-argument collection, dynamic function materialization, and Function source reconstruction helpers into `builtin_dispatch/function_helpers.rs`.
- [x] Move ordinary-object allocation, descriptor materialization, object integrity checks, and internal `instanceof` helper dispatch into `builtin_dispatch/object_helpers.rs`.
- [x] Move template literal string conversion and template-object cache materialization into `builtin_dispatch/template_helpers.rs`.
- [x] Move the `VmBuiltinDispatch` bridge type and its `ToPrimitiveContext`, `InternalBuiltinDispatchContext`, and `PublicBuiltinDispatchContext` implementations into `builtin_dispatch/dispatch_context.rs`.
- [x] Move builtin and embedding function object allocation into `builtin_dispatch/function_allocation.rs`.
- [x] Move `VmBuiltinDispatch` support helpers and `ToPrimitiveContext` implementation into `builtin_dispatch/dispatch_context/support.rs`.
- [x] Move `InternalBuiltinDispatchContext` routing into `builtin_dispatch/dispatch_context/internal.rs`.
- [x] Move `PublicBuiltinDispatchContext` routing into `builtin_dispatch/dispatch_context/public.rs`.
- [x] Move private environment, instance field key, and private field dispatch helpers into `builtin_dispatch/class_helpers/private_fields.rs`.
- [x] Move `super` property and `super()` construction helper dispatch into `builtin_dispatch/class_helpers/super_ops.rs`.
- [ ] Move one owner group per patch while keeping `BuiltinInvocation` and VM error propagation unchanged.
- [ ] Verify with `cargo test -p lyng-js-vm`, `cargo test -p lyng-js-tests`, and targeted Test262 slices for moved dynamic behavior.
  - [x] Verified the dynamic-import and class-helper owner split with `cargo test -p lyng-js-vm` (241 passed, doc-tests 0).
  - [x] Verified the function-helper owner split with `cargo test -p lyng-js-vm` (241 passed, doc-tests 0), `cargo test -p lyng-js-tests` (880 passed, doc-tests 0), and Test262 `built-ins/Function` (885 passed, 0 failed, 0 panicked).
  - [x] Verified the object-helper owner split with `cargo test -p lyng-js-vm` (241 passed, doc-tests 0), `cargo test -p lyng-js-tests` (880 passed, doc-tests 0), and Test262 `built-ins/Object` (6734 passed, 0 failed, 0 panicked).
  - [x] Verified the template-helper owner split with `cargo test -p lyng-js-vm` (241 passed, doc-tests 0), `cargo test -p lyng-js-tests template` (14 passed), `cargo test -p lyng-js-tests` (880 passed, doc-tests 0), Test262 `language/expressions/template-literal` (114 passed, 0 failed, 0 panicked), and Test262 `language/expressions/tagged-template` (44 passed, 4 failed, 0 panicked; unchanged from pre-move baseline).
  - [x] Verified the dispatch-context bridge split with `cargo test -p lyng-js-vm` (241 passed, doc-tests 0) and `cargo test -p lyng-js-tests` (880 passed, doc-tests 0).
  - [x] Verified the function-allocation owner split with `cargo test -p lyng-js-vm` (241 passed, doc-tests 0) and `cargo test -p lyng-js-tests` (880 passed, doc-tests 0).
  - [x] Verified the dispatch-context support split with `cargo test -p lyng-js-vm` (241 passed, doc-tests 0) and `cargo test -p lyng-js-tests` (880 passed, doc-tests 0).
  - [x] Verified the dispatch-context internal routing split with `cargo test -p lyng-js-vm` (241 passed, doc-tests 0) and `cargo test -p lyng-js-tests` (880 passed, doc-tests 0).
  - [x] Verified the dispatch-context public routing split with `cargo test -p lyng-js-vm` (241 passed, doc-tests 0) and `cargo test -p lyng-js-tests` (880 passed, doc-tests 0).
  - [x] Verified the class/private-field helper split with `cargo test -p lyng-js-vm` (241 passed, doc-tests 0) and `cargo test -p lyng-js-tests` (880 passed, doc-tests 0).
  - [x] Verified the class `super` helper split with `cargo test -p lyng-js-vm` (241 passed, doc-tests 0) and `cargo test -p lyng-js-tests` (880 passed, doc-tests 0).

## Task 6: Split GC Arena By Storage Responsibility

**Files:**
- Modify: `crates/lyng-js/gc/src/arena.rs`
- Create: `crates/lyng-js/gc/src/arena/*.rs`

- [x] Identify direct tests for allocation handles, tracing roots, weak/finalization records, and backing stores before moving code.
- [x] Move storage record definitions separately from allocation APIs.
- [x] Move generic slot-page, side-allocation, and value-slot storage internals into `gc/src/arena/storage.rs`.
- [x] Move weak-reference and finalization registry state APIs into `gc/src/arena/weak_state.rs`.
- [x] Keep barrier-ready store helper APIs stable.
- [x] Verify with `cargo test -p lyng-js-gc`, `cargo test -p lyng-js-env`, and targeted weak-reference/shared-memory integration tests when affected.
  - [x] Verified the GC arena record split with `cargo test -p lyng-js-gc` (29 passed, doc-tests 0) and `cargo test -p lyng-js-env` (37 passed, doc-tests 0).
  - [x] Verified the GC arena storage split with `cargo test -p lyng-js-gc` (29 passed, doc-tests 0) and `cargo test -p lyng-js-env` (37 passed, doc-tests 0).
  - [x] Verified the GC weak/finalization state split with `cargo test -p lyng-js-gc` (29 passed, doc-tests 0) and `cargo test -p lyng-js-env` (37 passed, doc-tests 0).
  - [x] Verified the GC store-helper split with `cargo test -p lyng-js-gc` (29 passed, doc-tests 0) and `cargo test -p lyng-js-env` (37 passed, doc-tests 0).

## Task 7: Split Builtin Bootstrap By Ownership Layer

**Files:**
- Modify: `crates/lyng-js/builtins/src/bootstrap.rs`
- Create: `crates/lyng-js/builtins/src/bootstrap/*.rs`

- [x] Map bootstrap responsibilities before moving code: intrinsic handles, public registry allocation, prototype linking, and installation helpers.
- [x] Move descriptor table installation and resolution helpers into `bootstrap/descriptors.rs`.
- [x] Move default global descriptor table construction into `bootstrap/globals.rs`.
- [x] Move oversized bootstrap unit tests into `bootstrap/tests.rs`.
- [x] Move stable builtin registry/metadata helpers out of the bootstrap coordinator.
- [x] Move public family prototype-link helper into `public/families/prototype_links.rs`.
- [x] Move prototype and intrinsic initialization helpers into owner-focused child modules.
- [x] Keep exported bootstrap APIs as the orchestration facade.
- [x] Verify with `cargo test -p lyng-js-builtins --lib` and `cargo test -p lyng-js-tests`.
  - [x] Verified the descriptor installer split with `cargo test -p lyng-js-builtins --lib` (32 passed) and `cargo test -p lyng-js-tests` (880 passed, doc-tests 0).
  - [x] Verified the global descriptor split with `cargo test -p lyng-js-builtins --lib` (32 passed), `cargo test -p lyng-js-tests` (880 passed, doc-tests 0), and `cargo fmt --all --check`.
  - [x] Verified the bootstrap test-module split with `cargo test -p lyng-js-builtins --lib` (32 passed), `cargo test -p lyng-js-tests` (880 passed, doc-tests 0), and `cargo fmt --all --check`.
  - [x] Verified the metadata facade split with `cargo test -p lyng-js-builtins --lib` (32 passed), `cargo test -p lyng-js-tests` (880 passed, doc-tests 0), and `cargo fmt --all --check`.
  - [x] Verified the public family prototype-link split with `cargo test -p lyng-js-builtins --lib` (32 passed), `cargo test -p lyng-js-tests` (880 passed, doc-tests 0), and `cargo fmt --all --check`.
  - [x] Verified the public family intrinsic split with `cargo test -p lyng-js-builtins --lib` (32 passed), `cargo test -p lyng-js-tests` (880 passed, doc-tests 0), and `cargo fmt --all --check`.

## Task 8: Split Object Internal Methods By Operation Family

**Files:**
- Modify: `crates/lyng-js/objects/src/internal_methods.rs`
- Create: `crates/lyng-js/objects/src/internal_methods/*.rs`

- [x] Map ordinary, function, proxy, array/exotic, descriptor-related, and named-property-cache internal method ownership.
- [x] Move named-property cache planning/load/store and cache-validation helpers into `internal_methods/property_cache.rs`.
- [x] Move module-namespace, string-exotic, typed-array index, and engine-array length/index helpers into dedicated `internal_methods/{module_namespace,string_exotics,typed_arrays,engine_arrays}.rs` modules.
- [x] Move integrity flag recomputation helpers into `internal_methods/integrity.rs`.
- [x] Move indexed-element slot helpers, element define/set/delete, index descriptors, and element-key collection into `internal_methods/elements.rs`.
- [x] Move named-property slot helpers, dictionary transitions, named descriptors, and named-key collection into `internal_methods/named_properties.rs`.
- [x] Move ordinary prototype, extensibility, descriptor, get/set/delete, and own-key algorithms into `internal_methods/ordinary.rs`.
- [x] Move one operation family per patch while preserving `ObjectInternalMethods` dispatch behavior.
- [x] Verify with `cargo test -p lyng-js-objects`, `cargo test -p lyng-js-vm`, and focused object/proxy/array Test262 slices when behavior-affecting.
  - [x] Verified the named-property cache split with `cargo test -p lyng-js-objects` (49 passed, doc-tests 0), `cargo test -p lyng-js-vm` (241 passed, doc-tests 0), and `cargo fmt --all --check`.
  - [x] Verified the exotic-object helper split with `cargo test -p lyng-js-objects` (49 passed, doc-tests 0), `cargo test -p lyng-js-vm` (241 passed, doc-tests 0), and `cargo fmt --all --check`.
  - [x] Verified the integrity helper split with `cargo test -p lyng-js-objects` (49 passed, doc-tests 0), `cargo test -p lyng-js-vm` (241 passed, doc-tests 0), and `cargo fmt --all --check`.
  - [x] Verified the indexed-element split with `cargo test -p lyng-js-objects define_own_property`, `cargo test -p lyng-js-objects own_property_keys`, `cargo test -p lyng-js-objects element`, `cargo test -p lyng-js-objects` (49 passed, doc-tests 0), `cargo test -p lyng-js-vm` (241 passed, doc-tests 0), `cargo test -p lyng-js-tests objects` (45 passed, 835 filtered), and `cargo fmt --all --check`.
  - [x] Verified the named-property split with `cargo test -p lyng-js-objects named_property`, `cargo test -p lyng-js-objects define_own_property`, `cargo test -p lyng-js-objects own_property_keys`, `cargo test -p lyng-js-objects` (49 passed, doc-tests 0), `cargo test -p lyng-js-vm` (241 passed, doc-tests 0), and `cargo fmt --all --check`.
  - [x] Verified the ordinary algorithm split with `cargo test -p lyng-js-objects ordinary_internal_methods`, `cargo test -p lyng-js-objects proxy_internal_methods`, `cargo test -p lyng-js-objects define_own_property`, `cargo test -p lyng-js-objects` (49 passed, doc-tests 0), `cargo test -p lyng-js-vm` (241 passed, doc-tests 0), `cargo test -p lyng-js-tests objects` (45 passed, 835 filtered), and `cargo fmt --all --check`.
  - [x] Verified the exotic-family split with `cargo test -p lyng-js-objects` (49 passed, doc-tests 0), `cargo test -p lyng-js-vm` (241 passed, doc-tests 0), `cargo test -p lyng-js-tests objects` (45 passed, 835 filtered), `cargo test -p lyng-js-tests shared_memory` (4 passed, 876 filtered), and `cargo fmt --all --check`.

## Task 9: Split Agent Runtime State By Domain

**Files:**
- Modify: `crates/lyng-js/env/src/agent.rs`
- Create: `crates/lyng-js/env/src/agent/*.rs`

- [x] Map heap access, realms/contexts, jobs/promises, module state, shared memory, and accounting responsibilities.
- [x] Move cluster backing-store/shared-memory handles and Agent forwarding methods into `agent/cluster_handles.rs`.
- [x] Move Agent job queue facade methods into `agent/jobs.rs`.
- [x] Move Agent disposal capability and async-disposal facade methods into `agent/disposal.rs`.
- [x] Move Agent promise record, reaction, capability, resolving/finally function, and combinator facade methods into `agent/promises.rs`.
- [x] Move environment layout, allocation, slot, function/global binding, and environment metadata helpers into `agent/environments.rs`.
- [x] Move Agent module-record cache facade methods into `agent/modules.rs`.
- [x] Move Agent execution-context stack facade methods into `agent/execution_contexts.rs`.
- [x] Move Agent realm shell, metadata, and bootstrap-state facade methods into `agent/realms.rs`.
- [x] Move Agent weak-reference, finalization cleanup, and forced-collection facade methods into `agent/weak_finalization.rs`.
- [x] Move Agent atom, runtime string, well-known symbol, and global symbol registry facade methods into `agent/symbols.rs`.
- [x] Move Agent phase-6 accounting facade methods into `agent/accounting.rs`.
- [x] Move side-table/state helpers into owner modules without changing `Agent` construction or embedding-facing APIs.
- [x] Verify with `cargo test -p lyng-js-env`, `cargo test -p lyng-js-vm`, and `cargo test -p lyng-js-tests`.
  - [x] Verified the cluster-handle split with `cargo test -p lyng-js-env backing_store` (7 passed, 30 filtered), `cargo test -p lyng-js-env shared_memory` (1 passed, 36 filtered), `cargo test -p lyng-js-env` (37 passed, doc-tests 0), and `cargo fmt --all --check`.
  - [x] Verified the job facade split with `cargo test -p lyng-js-env runtime_job_ids_fail_loudly_on_overflow` (1 passed, 36 filtered), `cargo test -p lyng-js-env` (37 passed, doc-tests 0), `cargo test -p lyng-js-vm promise` (23 passed, 218 filtered), and `cargo fmt --all --check`.
  - [x] Verified the disposal facade split with `cargo test -p lyng-js-env disposal` (2 passed, 35 filtered), `cargo test -p lyng-js-env` (37 passed, doc-tests 0), `cargo test -p lyng-js-vm using` (9 passed, 232 filtered), and `cargo fmt --all --check`.
  - [x] Verified the promise facade split with `cargo test -p lyng-js-env promise` (6 passed, 31 filtered), `cargo test -p lyng-js-env` (37 passed, doc-tests 0), `cargo test -p lyng-js-vm promise` (23 passed, 218 filtered), and `cargo fmt --all --check`.
  - [x] Verified the environment facade split with `cargo test -p lyng-js-env environment` (8 passed, 29 filtered), `cargo test -p lyng-js-env module_environment` (2 passed, 35 filtered), `cargo test -p lyng-js-env` (37 passed, doc-tests 0), `cargo test -p lyng-js-vm environment` (7 passed, 234 filtered), `cargo test -p lyng-js-tests environment` (3 passed, 877 filtered), and `cargo fmt --all --check`.
  - [x] Verified the module-record facade split with `cargo test -p lyng-js-env module` (3 passed, 34 filtered), `cargo test -p lyng-js-vm module` (36 passed, 205 filtered), `cargo test -p lyng-js-tests module` (6 passed, 874 filtered), `cargo test -p lyng-js-env` (37 passed, doc-tests 0), and `cargo fmt --all --check`.
  - [x] Verified the execution-context facade split with `cargo test -p lyng-js-env execution_context` (2 passed, 35 filtered), `cargo test -p lyng-js-env eval_execution_context` (1 passed, 36 filtered), `cargo test -p lyng-js-vm environment` (7 passed, 234 filtered), `cargo test -p lyng-js-tests environment` (3 passed, 877 filtered), `cargo test -p lyng-js-env` (37 passed, doc-tests 0), and `cargo fmt --all --check`.
  - [x] Verified the realm facade split with `cargo test -p lyng-js-env realm` (2 passed, 35 filtered), `cargo test -p lyng-js-env default_realm` (1 passed, 36 filtered), `cargo test -p lyng-js-builtins --lib` (32 passed), `cargo test -p lyng-js-vm environment` (7 passed, 234 filtered), `cargo test -p lyng-js-env` (37 passed, doc-tests 0), and `cargo fmt --all --check`.
  - [x] Verified the weak/finalization facade split with `cargo test -p lyng-js-env` (37 passed, doc-tests 0), `cargo test -p lyng-js-vm` (241 passed, doc-tests 0), `cargo test -p lyng-js-builtins --lib` (32 passed), and `cargo fmt --all --check`.
  - [x] Verified the symbol/string facade split with `cargo test -p lyng-js-env symbol` (1 passed, 36 filtered), `cargo test -p lyng-js-vm symbol` (5 passed, 236 filtered), `cargo test -p lyng-js-vm string` (12 passed, 229 filtered), `cargo test -p lyng-js-env` (37 passed, doc-tests 0), `cargo test -p lyng-js-builtins --lib` (32 passed), and `cargo fmt --all --check`.
  - [x] Verified the accounting facade split and final agent split state with `cargo test -p lyng-js-env accounting` (3 passed, 34 filtered), `cargo test -p lyng-js-env` (37 passed, doc-tests 0), `cargo test -p lyng-js-vm` (241 passed, doc-tests 0), `cargo test -p lyng-js-tests` (880 passed, doc-tests 0), and `cargo fmt --all --check`.

## Task 10: Split Object Operations By Semantic Group

**Files:**
- Modify: `crates/lyng-js/ops/src/object.rs`
- Create: `crates/lyng-js/ops/src/object/*.rs`

- [x] Map property access, descriptor conversion, construction/calls, prototype/integrity, and private-field operation groups.
- [x] Move embedded object-operation tests into `object/tests.rs` before behavior-owner splits.
- [x] Move private class-element bridge operations into `object/private_elements.rs`.
- [x] Move Date and Temporal receiver payload validators into `object/receiver_payloads.rs`.
- [x] Move primitive wrapper allocation, wrapping, payload validation, and string element cache helpers into `object/primitive_wrappers.rs`.
- [x] Move object-to-primitive, numeric conversion bridge, and BigInt conversion/formatting helpers into `object/conversions.rs` and `object/bigint.rs`.
- [x] Move typed-array index descriptor and backing-store read helpers into `object/typed_array_indices.rs`.
- [x] Move ordinary/bootstrap object operations plus call/construct wrappers into `object/ordinary.rs`.
- [x] Move one semantic group per patch while keeping public operation names re-exported through `object.rs`.
- [x] Verify with `cargo test -p lyng-js-ops`, `cargo test -p lyng-js-vm`, and focused object/class Test262 slices when behavior-affecting.
  - [x] Verified the object-operation test-module split with `cargo test -p lyng-js-ops object::tests` (9 passed, 65 filtered) and `cargo fmt --all --check`.
  - [x] Verified the private-element bridge split with `cargo test -p lyng-js-ops object::tests` (9 passed, 65 filtered), `cargo test -p lyng-js-vm` (241 passed, doc-tests 0), `cargo test -p lyng-js-tests class` (71 passed, 809 filtered), and `cargo fmt --all --check`.
  - [x] Verified the receiver-payload split with `cargo test -p lyng-js-ops object::tests` (9 passed, 65 filtered), `cargo test -p lyng-js-builtins --lib` (32 passed), `cargo test -p lyng-js-tests temporal` (165 passed, 715 filtered), and `cargo fmt --all --check`.
  - [x] Verified the primitive-wrapper split with `cargo test -p lyng-js-ops object::tests` (9 passed, 65 filtered), `cargo test -p lyng-js-builtins --lib` (32 passed), `cargo test -p lyng-js-tests temporal` (165 passed, 715 filtered), `cargo test -p lyng-js-vm` (241 passed, doc-tests 0), and `cargo fmt --all --check`.
  - [x] Verified the conversion and BigInt split with `cargo test -p lyng-js-ops object::tests` (9 passed, 65 filtered), `cargo test -p lyng-js-vm` (241 passed, doc-tests 0), `cargo test -p lyng-js-tests script_core` (165 passed, 715 filtered), and `cargo fmt --all --check`.
  - [x] Verified the typed-array index split with `cargo test -p lyng-js-ops object::tests` (9 passed, 65 filtered), `cargo test -p lyng-js-vm` (241 passed, doc-tests 0), `cargo test -p lyng-js-tests shared_memory` (4 passed, 876 filtered), and `cargo fmt --all --check`.
  - [x] Verified the ordinary-operation split with `cargo test -p lyng-js-ops object::tests` (9 passed, 65 filtered), `cargo test -p lyng-js-vm` (241 passed, doc-tests 0), `cargo test -p lyng-js-tests objects` (45 passed, 835 filtered), and `cargo fmt --all --check`.
  - [x] Verified final Task 10 object-ops coverage with `cargo test -p lyng-js-ops` (74 passed, doc-tests 0).

## Task 11: Split Binary Data Builtins By Runtime Family

**Files:**
- Modify: `crates/lyng-js/builtins/src/public/families/binary_data.rs`
- Create: `crates/lyng-js/builtins/src/public/families/binary_data/*.rs`
- Modify: `crates/lyng-js/builtins/src/public/dispatch/binary_data.rs`
- Create: `crates/lyng-js/builtins/src/public/dispatch/binary_data/*.rs`

- [x] Map binary-data bootstrap and dispatch responsibilities across ArrayBuffer/SharedArrayBuffer, DataView, Atomics, and typed arrays.
- [x] Move binary-data family bootstrap lookup, installation, and descriptor-table wiring into `public/families/binary_data/{lookup,install,descriptors}.rs`.
- [x] Move ArrayBuffer and SharedArrayBuffer dispatch, allocation, species, byte-length, and slice helpers into `public/dispatch/binary_data/buffers.rs`.
- [x] Move DataView dispatch, allocation, receiver validation, byte-offset access, and get/set helpers into `public/dispatch/binary_data/data_view.rs`.
- [x] Move Atomics dispatch, shared typed-array validation, read/modify/write, notify, wait, and waitAsync helpers into `public/dispatch/binary_data/atomics.rs`.
- [x] Move typed-array dispatch, validation, storage conversion, construction, iteration, mutation, and search helpers into typed-array owner modules.
  - [x] Move typed-array storage conversion and backing-store read/write helpers into `public/dispatch/binary_data/typed_arrays.rs`.
  - [x] Move typed-array allocation, receiver validation, default constructor/prototype lookup, and species-create helpers into `public/dispatch/binary_data/typed_arrays.rs`.
  - [x] Move typed-array constructor dispatch, `TypedArray.from`/`of`, concrete typed-array constructors, and construction-from-buffer/source algorithms into `public/dispatch/binary_data/typed_arrays/construction.rs`.
  - [x] Move typed-array prototype accessors, iterator entrypoints, `at`, `toString`, `toLocaleString`, and `@@toStringTag` dispatch into `public/dispatch/binary_data/typed_arrays/access.rs`.
  - [x] Move callback iteration methods (`every`, `some`, `find*`, `filter`, `forEach`, `map`, `reduce`, and `reduceRight`) into `public/dispatch/binary_data/typed_arrays/iteration.rs`.
  - [x] Move typed-array mutation, copy, sort, `set`, `slice`, and `subarray` dispatch into `public/dispatch/binary_data/typed_arrays/mutation.rs`.
  - [x] Move typed-array `join`, `includes`, `indexOf`, and `lastIndexOf` dispatch into `public/dispatch/binary_data/typed_arrays/search.rs`.
- [x] Further split oversized binary-data descriptor wiring by buffer, DataView, Atomics, and typed-array descriptor families.
  - [x] Split ArrayBuffer/SharedArrayBuffer, DataView, and Atomics descriptor tables into `public/families/binary_data/descriptors/{buffers,data_view,atomics}.rs`; typed-array descriptors remain in the parent pending a follow-up split.
  - [x] Split typed-array descriptor tables into `public/families/binary_data/descriptors/typed_arrays.rs`.
- [x] Verify current bootstrap and buffer dispatch split.
  - [x] Verified with `cargo test -p lyng-js-builtins --lib` (32 passed), `cargo test -p lyng-js-vm` (241 passed, doc-tests 0), `cargo test -p lyng-js-tests shared_memory` (4 passed, 876 filtered), `cargo fmt --all --check`, Test262 `built-ins/ArrayBuffer` (158 passed, 4 failed, 0 panicked), and Test262 `built-ins/SharedArrayBuffer` (118 passed, 2 failed, 0 panicked).
  - [x] Verified the DataView dispatch split with `cargo test -p lyng-js-builtins --lib` (32 passed), `cargo test -p lyng-js-vm` (241 passed, doc-tests 0), `cargo test -p lyng-js-tests shared_memory` (4 passed, 876 filtered), `cargo fmt --all --check`, and Test262 `built-ins/DataView` (884 passed, 156 failed, 0 panicked).
  - [x] Verified the Atomics dispatch split with `cargo test -p lyng-js-builtins --lib` (32 passed), `cargo test -p lyng-js-vm` (241 passed, doc-tests 0), `cargo test -p lyng-js-tests shared_memory` (4 passed, 876 filtered), `cargo fmt --all --check`, and Test262 `built-ins/Atomics` (416 passed, 4 failed, 0 panicked).
  - [x] Verified the buffer/DataView/Atomics descriptor split with `cargo test -p lyng-js-builtins --lib` (32 passed), `cargo fmt --all --check`, and combined binary-data follow-up checks before commit.
  - [x] Verified the typed-array storage helper split with `cargo test -p lyng-js-builtins --lib` (32 passed), `cargo test -p lyng-js-vm` (241 passed, doc-tests 0), `cargo test -p lyng-js-tests shared_memory` (4 passed, 876 filtered), `cargo fmt --all --check`, and Test262 `built-ins/TypedArray` (2438 passed, 6 failed, 0 panicked, 416 skipped).
  - [x] Verified the typed-array support and descriptor split with `cargo test -p lyng-js-builtins --lib` (32 passed), `cargo test -p lyng-js-vm` (241 passed, doc-tests 0), `cargo test -p lyng-js-tests shared_memory` (4 passed, 876 filtered), `cargo fmt --all --check`, `git diff --check`, and Test262 `built-ins/TypedArray` (2438 passed, 6 failed, 0 panicked, 416 skipped).
  - [x] Verified the typed-array constructor split with `cargo test -p lyng-js-builtins --lib` (32 passed), `cargo test -p lyng-js-vm` (241 passed, doc-tests 0), `cargo test -p lyng-js-tests shared_memory` (4 passed, 876 filtered), `cargo fmt --all --check`, `git diff --check`, and Test262 `built-ins/TypedArray` (2438 passed, 6 failed, 0 panicked, 416 skipped).
  - [x] Verified the typed-array access split with `cargo test -p lyng-js-builtins --lib` (32 passed), `cargo test -p lyng-js-vm` (241 passed, doc-tests 0), `cargo test -p lyng-js-tests shared_memory` (4 passed, 876 filtered), `cargo fmt --all --check`, `git diff --check`, and Test262 `built-ins/TypedArray` (2438 passed, 6 failed, 0 panicked, 416 skipped).
  - [x] Verified the typed-array iteration split with `cargo test -p lyng-js-builtins --lib` (32 passed), `cargo test -p lyng-js-vm` (241 passed, doc-tests 0), `cargo test -p lyng-js-tests shared_memory` (4 passed, 876 filtered), `cargo fmt --all --check`, `git diff --check`, and Test262 `built-ins/TypedArray` (2438 passed, 6 failed, 0 panicked, 416 skipped).
  - [x] Verified the typed-array mutation split with `cargo test -p lyng-js-builtins --lib` (32 passed), `cargo test -p lyng-js-vm` (241 passed, doc-tests 0), `cargo test -p lyng-js-tests shared_memory` (4 passed, 876 filtered), `cargo fmt --all --check`, `git diff --check`, and Test262 `built-ins/TypedArray` (2438 passed, 6 failed, 0 panicked, 416 skipped).
  - [x] Verified the typed-array search split with `cargo test -p lyng-js-builtins --lib` (32 passed), `cargo test -p lyng-js-vm` (241 passed, doc-tests 0), `cargo test -p lyng-js-tests shared_memory` (4 passed, 876 filtered), `cargo fmt --all --check`, `git diff --check`, and Test262 `built-ins/TypedArray` (2438 passed, 6 failed, 0 panicked, 416 skipped).

## Task 12: Split Env Execution Intrinsics

**Files:**
- Modify: `crates/lyng-js/env/src/execution.rs`
- Create: `crates/lyng-js/env/src/execution/intrinsics.rs`

- [x] Move the realm-owned `Intrinsics` table and accessors into `execution/intrinsics.rs`.
- [x] Keep `execution.rs` as the public facade by re-exporting `Intrinsics`.
- [x] Verify with `cargo test -p lyng-js-env` (37 passed), `cargo test -p lyng-js-builtins --lib` (32 passed), `cargo test -p lyng-js-vm environment` (7 passed, 234 filtered), `cargo fmt --all --check`, and `git diff --check`.

## Task 13: Split Array Iteration Dispatch

**Files:**
- Modify: `crates/lyng-js/builtins/src/public/dispatch/arrays.rs`
- Create: `crates/lyng-js/builtins/src/public/dispatch/arrays/iteration.rs`

- [x] Move Array callback iteration, search, reduce, filter, flat/flatMap, forEach, and map dispatch into `public/dispatch/arrays/iteration.rs`.
- [x] Keep `arrays.rs` as the Array family router and re-export `array_index_of_builtin` for internal shim callers.
- [x] Verify with `cargo test -p lyng-js-builtins --lib` (32 passed), `cargo test -p lyng-js-tests array` (53 passed, 827 filtered), `cargo test -p lyng-js-tests script_core` (165 passed, 715 filtered), `cargo fmt --all --check`, `git diff --check`, and Test262 `built-ins/Array` (5921 passed, 30 failed, 0 panicked, 164 skipped).

## Task 14: Split Primitive Math Dispatch

**Files:**
- Modify: `crates/lyng-js/builtins/src/public/dispatch/primitives.rs`
- Create: `crates/lyng-js/builtins/src/public/dispatch/primitives/math.rs`

- [x] Move `Math` dispatcher, numeric builtin algorithms, float16 rounding, and `Math.sumPrecise` helpers into `public/dispatch/primitives/math.rs`.
- [x] Keep `primitives.rs` as the primitive-family router and leave Number, BigInt, Boolean, and Symbol dispatch in the parent.
- [x] Verify with `cargo test -p lyng-js-builtins --lib` (32 passed), `cargo test -p lyng-js-tests script_core` (165 passed, 715 filtered), `cargo fmt --all --check`, `git diff --check`, and Test262 `built-ins/Math` (654 passed, 0 failed, 0 panicked, 0 skipped).

## Task 15: Split String Basic Dispatch

**Files:**
- Modify: `crates/lyng-js/builtins/src/public/dispatch/strings.rs`
- Create: `crates/lyng-js/builtins/src/public/dispatch/strings/basic.rs`

- [x] Move String constructor/static dispatch, String iterator dispatch, primitive wrapper access, concatenation, char/at/code-point, and `String.raw` helpers into `public/dispatch/strings/basic.rs`.
- [x] Keep `strings.rs` as the String family router and leave search/regexp and transform algorithms in the parent for follow-up splits.
- [x] Verify with `cargo test -p lyng-js-builtins --lib` (32 passed), `cargo test -p lyng-js-tests string` (85 passed, 795 filtered), `cargo fmt --all --check`, `git diff --check`, and Test262 `built-ins/String` (2429 passed, 0 failed, 0 panicked, 14 skipped).

## Task 16: Split String Normalization Dispatch

**Files:**
- Modify: `crates/lyng-js/builtins/src/public/dispatch/strings.rs`
- Create: `crates/lyng-js/builtins/src/public/dispatch/strings/normalization.rs`

- [x] Move `String.prototype.localeCompare`, `String.prototype.normalize`, and the normalization/collation helper cluster into `public/dispatch/strings/normalization.rs`.
- [x] Keep search/regexp and remaining transform algorithms in `strings.rs` for follow-up owner splits.
- [x] Verify with `cargo test -p lyng-js-builtins --lib` (32 passed), `cargo test -p lyng-js-tests string` (85 passed, 795 filtered), Test262 `built-ins/String/prototype/normalize` (28 passed, 0 failed, 0 panicked, 0 skipped), Test262 `built-ins/String/prototype/localeCompare` (26 passed, 0 failed, 0 panicked, 0 skipped), `cargo fmt --all --check`, and `git diff --check`.

## Task 17: Split RegExp Escape Dispatch

**Files:**
- Modify: `crates/lyng-js/builtins/src/public/dispatch/regexp.rs`
- Create: `crates/lyng-js/builtins/src/public/dispatch/regexp/escape.rs`

- [x] Move `RegExp.escape` and its encoding helper cluster into `public/dispatch/regexp/escape.rs`.
- [x] Keep `regexp.rs` as the RegExp family router and leave constructor/prototype/symbol algorithms in the parent.
- [x] Verify with `cargo test -p lyng-js-builtins --lib` (32 passed), Test262 `built-ins/RegExp/escape` (40 passed, 0 failed, 0 panicked, 0 skipped), `cargo fmt --all --check`, and `git diff --check`.

## Task 18: Split Collection Iteration Dispatch

**Files:**
- Modify: `crates/lyng-js/builtins/src/public/dispatch/collections.rs`
- Create: `crates/lyng-js/builtins/src/public/dispatch/collections/iteration.rs`

- [x] Move Map/Set `forEach`, iterator factory, and iterator `next` algorithms into `public/dispatch/collections/iteration.rs`.
- [x] Keep `collections.rs` as the collection-family router and leave collection construction, storage mutation, weak collections, WeakRef, and FinalizationRegistry dispatch in the parent.
- [x] Verify with `cargo test -p lyng-js-builtins --lib` (32 passed), Test262 `built-ins/Map` (316 passed, 24 failed, 0 panicked, 65 skipped), Test262 `built-ins/Set` (392 passed, 0 failed, 0 panicked, 372 skipped), `cargo fmt --all --check`, and `git diff --check`.

## Task 19: Split Primitive BigInt Dispatch

**Files:**
- Modify: `crates/lyng-js/builtins/src/public/dispatch/primitives.rs`
- Create: `crates/lyng-js/builtins/src/public/dispatch/primitives/bigint.rs`

- [x] Move BigInt constructor, width-limiting, wrapper, string conversion, and helper algorithms into `public/dispatch/primitives/bigint.rs`.
- [x] Keep `primitives.rs` as the primitive-family router and expose only BigInt dispatch plus the BigInt-to-Number conversion helper back to the parent.
- [x] Verify with `cargo test -p lyng-js-builtins --lib` (32 passed), `cargo test -p lyng-js-tests script_core` (165 passed, 715 filtered), Test262 `built-ins/BigInt` (154 passed, 0 failed, 0 panicked, 0 skipped), `cargo fmt --all --check`, and `git diff --check`.

## Task 20: Split Date Parsing Support

**Files:**
- Modify: `crates/lyng-js/builtins/src/public/dispatch/date.rs`
- Create: `crates/lyng-js/builtins/src/public/dispatch/date/parsing.rs`

- [x] Move `Date.parse` ISO/text parsing helpers into `public/dispatch/date/parsing.rs`.
- [x] Keep date calendar, local-time conversion, formatting, getters, setters, and object allocation in `date.rs` for follow-up splits.
- [x] Verify with `cargo test -p lyng-js-builtins --lib` (32 passed), `cargo test -p lyng-js-tests date` (91 passed, 789 filtered), `cargo test -p lyng-js-tests script_core_date` (3 passed, 877 filtered), `cargo test -p lyng-js-tests script_core_installs_date` (1 passed, 879 filtered), Test262 `built-ins/Date` (1188 passed, 0 failed, 0 panicked, 0 skipped), `cargo fmt --all --check`, and `git diff --check`.

## Task 21: Split RegExp Symbol Methods

**Files:**
- Modify: `crates/lyng-js/builtins/src/public/dispatch/regexp.rs`
- Create: `crates/lyng-js/builtins/src/public/dispatch/regexp/symbols.rs`

- [x] Move RegExp well-known symbol dispatch and `@@match`, `@@replace`, `@@search`, `@@split`, and `@@matchAll` entrypoints into `public/dispatch/regexp/symbols.rs`.
- [x] Keep RegExp construction, prototype accessors, execution helpers, and shared match/search/replace algorithms in `regexp.rs` for follow-up splits.
- [x] Verify with `cargo test -p lyng-js-builtins --lib` (32 passed), `cargo test -p lyng-js-tests regexp` (13 passed, 867 filtered), `cargo test -p lyng-js-tests string_match` (6 passed, 874 filtered), Test262 `built-ins/RegExp` (2701 passed, 371 failed, 0 panicked, 684 skipped), `cargo fmt --all --check`, and `git diff --check`.

## Task 22: Split RegExp Accessor Methods

**Files:**
- Modify: `crates/lyng-js/builtins/src/public/dispatch/regexp.rs`
- Create: `crates/lyng-js/builtins/src/public/dispatch/regexp/accessors.rs`

- [x] Move RegExp prototype flag, source, flags, and has-indices accessor algorithms into `public/dispatch/regexp/accessors.rs`.
- [x] Keep RegExp construction, execution, symbol methods, and shared matcher helpers in their existing owner modules for follow-up splits.
- [x] Verify with `cargo test -p lyng-js-builtins --lib` (32 passed), `cargo test -p lyng-js-tests regexp` (13 passed, 867 filtered), `cargo test -p lyng-js-tests regexp_source` (1 passed, 879 filtered), Test262 `built-ins/RegExp/prototype/flags` (30 passed, 0 failed, 0 panicked, 2 skipped), `source` (22 passed, 2 failed, 0 panicked), `global` (20 passed, 0 failed), `hasIndices` (16 passed, 0 failed), `ignoreCase` (20 passed, 0 failed), `dotAll` (16 passed, 0 failed), `unicode` (16 passed, 0 failed), `sticky` (16 passed, 0 failed), `cargo fmt --all --check`, and `git diff --check`.

## Task 23: Split RegExp Construction

**Files:**
- Modify: `crates/lyng-js/builtins/src/public/dispatch/regexp.rs`
- Create: `crates/lyng-js/builtins/src/public/dispatch/regexp/construction.rs`

- [x] Move the RegExp constructor algorithm, constructor pattern normalization, and species getter into `public/dispatch/regexp/construction.rs`.
- [x] Keep `RegExp.escape` routing in the parent constructor dispatcher and leave allocation/execution helpers in `regexp.rs` for follow-up splits.
- [x] Verify with `cargo test -p lyng-js-builtins --lib` (32 passed), `cargo test -p lyng-js-tests regexp_constructor` (2 passed, 878 filtered), `cargo test -p lyng-js-tests regexp` (13 passed, 867 filtered), `cargo test -p lyng-js-tests function_to_string_formats_regexp_species_getter` (1 passed, 879 filtered), Test262 `built-ins/RegExp/Symbol.species` (8 passed, 0 failed, 0 panicked), `built-ins/RegExp/call` (2 passed, 4 failed, 0 panicked), `built-ins/RegExp/is-a-constructor` (2 passed, 0 failed), `built-ins/RegExp/prototype/unicodeSets/uv-flags-constructor` (0 runnable, 2 skipped), `cargo fmt --all --check`, and `git diff --check`.

## Operating Rules

- Keep public APIs stable with `pub use` facades from parent modules.
- Do not change `Value`, handle widths, object layouts, bytecode encoding, or VM frame layout in this cleanup.
- Do not add third-party dependencies.
- Run the narrowest owner test after each split, then widen only when a semantic owner changes.
- Set `lyng-kb1k` to `in_review` after verified code and documentation changes; do not close it without explicit user approval.
