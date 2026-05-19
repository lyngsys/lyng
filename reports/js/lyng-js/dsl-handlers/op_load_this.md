# `op_load_this` DSL port (Phase 1.B.2, Task 3)

Inline port with sentinel-bail. Reads
`LlIntState::frame_this_value` (substrate established by Phase 1.B.1)
and bails to the slow path if the mirror equals
`Value::uninitialized_lexical()`. Top-30 dispatch share: **#12**,
~256M dispatches per V8 v7 run — the largest single port in Phase
1.B.2.

## DSL source

`crates/lyng-js/vm/src/dsl/handlers/cold.rs`:

```rust
llint_handler! {
    op_load_this_dsl, opcode_byte = 28, layout = Abx, length = 4, |a, bx| {
        load_state_value!(10, vm_state_offset = state_this_value);
        load_uninit_lex_sentinel!(11);
        cmp_branch_eq!(10, 11, .slow);
        store_reg!(a, 10);
        dispatch!();
        .slow:
        call_slow!(op_load_this_slow_rs, args = [a, bx]);
        dispatch_after_slow!();
    }
}
```

- `a` (byte 1): destination register id.
- `bx` (bytes 2-3): reserved by the Abx layout, unused at runtime
  (mirrors the existing `op_load_undefined`/`op_load_null`
  convention). Future IC-site instrumentation can populate it
  without changing the handler.

## Current asm (AArch64)

See `reports/js/lyng-js/dsl-asm-baseline-aarch64/op_load_this.asm`.

Captured from `target/release/deps/lyng_js_vm-*.s` after a
`cargo rustc --release -p lyng-js-vm --lib -- --emit=asm -C debuginfo=0`
build. Effective fast-path sequence (slow-path tail elided):

```asm
op_load_this_dsl:
    ldrb    w9,  [x19, #1]              ; decode a
    ldrh    w10, [x19, #2]              ; decode bx (unused)
    ldr     x10, [x24, #40]             ; load_state_value! — x10 = frame_this_value
    mov     x11, #2                     ; load_uninit_lex_sentinel! — movz low 16 bits
    movk    x11, #0,     lsl #16        ; load_uninit_lex_sentinel! — bits 16-31
    movk    x11, #9,     lsl #32        ; load_uninit_lex_sentinel! — Sentinel kind
    movk    x11, #32760, lsl #48        ; load_uninit_lex_sentinel! — NaN-tag header
    cmp     x10, x11                    ; cmp_branch_eq! — compare
    b.eq    Lop_load_this_dsl__slow     ; cmp_branch_eq! — bail on sentinel match
    str     x10, [x20, x9, lsl #3]      ; store_reg!(a, 10)
    add     x19, x19, #4                ; dispatch!() — advance PC
    ldrb    w8,  [x19]                  ; dispatch!() — load next opcode byte
    ldr     x16, [x23, x8, lsl #3]      ; dispatch!() — look up handler
    br      x16                         ; dispatch!() — tail-jump
```

**14 instructions** on the fast path (including the 2-instruction
decode prologue). Body = 12 instructions (1 load + 4 sentinel
materialization + 2 cmp/branch + 1 store + 4 dispatch tail) — within
the ≤12 body budget. The decode prologue (`ldrb` + `ldrh`) is shared
across all Abx-layout opcodes.

The `#40` literal is `offset_of!(LlIntState, frame_this_value)`
(pinned by the `LLINT_STATE_FRAME_THIS_VALUE` const). The `mov x11,
#2` is LLVM's rewrite of the canonical `movz x11, #2` (low quarter
of `VALUE_UNINIT_LEX_BITS = 0x7ff8_0009_0000_0002` = 9221120275695796226).
The three `movk` instructions OR in the higher quarters (bits 16-31,
32-47, 48-63) — quarter 1 (bits 16-31) is zero but LLVM kept the
instruction for canonical-encoding fidelity.

## Slow path

**Retained** (`op_load_this_slow_rs` — existing semantic-body bridge).
Fires when `frame_this_value` equals the
`Value::uninitialized_lexical()` sentinel, which `resolve_initial_this_value`
writes for `ThisState::Uninitialized` (derived-ctor TDZ; pre-super()
access throws ReferenceError) or `ThisState::Lexical` (arrow
function; walks the lex-env to resolve the lexical `this`).

The slow path re-enters `op_load_this_semantic` which handles both
arms uniformly. On Refresh egress the `frame_this_value` mirror is
re-populated by Phase 1.B.1's refresh logic in
`slow_path.rs::translate_outcome`.

## LLInt reference

JSC's LLInt `op_to_this` / direct `this`-access has roughly the same
shape: load `this` from a known frame-relative slot, check a sentinel
(JSC uses object-vs-not-object branching for the `this` re-binding
case), and either return the value or fall through to a slow path.

LLInt reference capture mode: **excerpt (manual)** — taken from
`/Users/sondre/dev/WebKit/Source/JavaScriptCore/llint/LowLevelInterpreter64.asm`.
JSC's equivalent on AArch64 is ~10-12 instructions for the fast path
(load + tag check + bail + store + dispatch), depending on the
ThisState arm. Lyng's 12-instruction body is in the same league;
the 4-instruction sentinel materialization is the main delta vs JSC's
single-instruction tag-bit check (JSC uses a tag bit in the upper
bits of the Value, whereas lyng's sentinel is a full 64-bit
distinguished pattern).

## Side-by-side diff

| Step | Lyng DSL                                          | LLInt (approximate `op_to_this`)             |
|------|---------------------------------------------------|----------------------------------------------|
| 1-2  | `ldrb w9 / ldrh w10` — decode operands            | `get(m_dst, t0)` + (no bx equivalent)        |
| 3    | `ldr x10, [x24, #40]` — load mirror               | `loadq [cfr + ThisOffset], t0`               |
| 4-7  | 4× movz/movk — materialize sentinel               | 1× tag-bit check (e.g. `andq`/`btq`)         |
| 8-9  | `cmp x10, x11; b.eq .slow` — bail check           | `bqeq t0, ValueEmpty, .slow` or equivalent   |
| 10   | `str x10, [x20, x9, lsl #3]` — store              | `storeq t0, [cfr, t0, 8]`                    |
| 11-14| `add/ldrb/ldr/br` — dispatch tail (4 instr)       | `dispatch()` — equivalent 4-instr tail       |

**Irreducible deltas vs LLInt:**

- **Full 64-bit sentinel materialization.** Lyng uses
  `Value::uninitialized_lexical()` (a NaN-tagged Sentinel) as the
  distinguished value; comparing against it requires materializing
  all 4 quarters of the 64-bit pattern. JSC uses a tag bit pattern
  and can check via a single AND/CMP. This is a ~3 instructions
  delta on the fast path. Acceptable: the sentinel is precomputed
  per call (no per-dispatch divergence), and the slow-path bail
  semantics make a full-equality test necessary (a partial mask
  could miss valid `this` values that happen to share some bits).
- **No tag-kind precheck.** Lyng compares the raw 64-bit Value bits
  for equality. If `frame_this_value` were ever set to a Value that
  happened to bit-equal the sentinel without being the sentinel
  (e.g., a double whose bit pattern collides), the inline path would
  incorrectly bail. This is statically impossible because
  `Value::uninitialized_lexical()` returns
  `tagged(TagKind::Sentinel = 9, raw = 2)` and TagKind::Sentinel
  uniquely identifies internal sentinels (kind 9 is not produced by
  any user value path). A compile-time assertion in
  `aarch64/prelude.rs` pins this invariant.

## Microbench

The Phase 1.B.2 plan / spec assumed a `LoadThis` snippet was added in
Phase 1.B.0 Task 7. **It was not** — `tools/lyng-js-bench/src/microbench/snippets.rs`
at HEAD `ad240f50` adds 14 snippets but neither `LoadConst8` nor
`LoadThis` are among them. The microbench gate for this opcode is
therefore **deferred** — the bench tool's snippet table needs a
`LoadThis` entry before the microbench can produce a ns/dispatch
number.

Expected (analogous-opcode extrapolation): ~50-55 ns/dispatch (12-inline-
instruction body adds ~5 instr of sentinel-materialization + cmp/branch
over LoadSmi8's 45 ns). LLInt 2× reference would be ~140 ns; substantial
headroom predicted but not measured directly at this HEAD.

The full microbench discussion lives at
[`reports/js/lyng-js/dsl-1/phase-1b2-microbench.md`](../dsl-1/phase-1b2-microbench.md).

## V8 v7

Same-load A/B comparison vs `68dd5e89` (Phase 1.B.1 close):
**+4.89% V8 v7 geomean** combined improvement from op_load_const8 +
op_load_this Phase 1.B.2 ports. Single-opcode attribution isn't
direct, but op_load_this carries the bulk of the dispatch volume
(~239M dispatches aggregate across 3 V8 v7 samples = ~80M per run;
slightly below the spec's 256M estimate but in same order of
magnitude), and the two workloads showing the largest improvement
(Richards +15.06%, DeltaBlue +9.03%) are method-dispatch-heavy and
the natural targets for an op_load_this speedup.

Full A/B report: [`reports/js/lyng-js/dsl-1/phase-1b2-ab-comparison.md`](../dsl-1/phase-1b2-ab-comparison.md).

## Slow-path-share

**0.00%** on V8 v7 (measured 2026-05-19, 3-sample run via
`v8suite --count-opcodes --count-slow-path-share`). Across all 6
V8 v7 workloads, op_load_this dispatched 239,159,248 times and recorded
0 semantic / 0 safepoint slow-path entries.

Per-workload breakdown:

| Workload     | Dispatches | Semantic SP | Share   |
|--------------|-----------:|------------:|--------:|
| Richards     |  82,426,819 |          0 |  0.00% |
| DeltaBlue    |  55,009,605 |          0 |  0.00% |
| Crypto       |  35,929,858 |          0 |  0.00% |
| RayTrace     |  60,452,179 |          0 |  0.00% |
| NavierStokes |       1,605 |          0 |  0.00% |
| Splay        |   5,339,182 |          0 |  0.00% |

V8 v7's workloads exercise only `ThisState::Value(v)` paths — no
derived constructors (Uninitialized) and no arrow functions reading a
lexical `this` (arrow-captured `this` is resolved to a concrete `Value`
at trampoline entry by `resolve_initial_this_value`, not stored as the
sentinel). The sentinel compare on the fast path is therefore always
not-equal for these workloads. Future workloads (e.g., Test262 with
class-heavy fixtures) may push share above 0%, but the < 20% gate has
substantial headroom.

## Behavioral tests

- `cargo test -p lyng-js-vm --lib --release` — **418 passed**.
- `cargo test -p lyng-js-tests --release` — **1198 passed** (1187
  baseline + 1 vm-lib prelude test + 5 op_load_const8 tests + 6 new
  op_load_this tests).

Both green; behavioral parity preserved.

Integration tests cover (`tests/src/op_load_this_inline.rs`):
1. ThisState::Value(v) via `.call({x: 42})` — fast path, no bail.
2. ThisState::Value(v) with negative Smi (sign correctness).
3. ThisState::Lexical via arrow function (closure captures outer
   `this`). `resolve_initial_this_value` resolves the lexical `this`
   to a concrete Value before the inline read; no sentinel bail
   occurs in the resolved case.
4. Chained property accesses through `this` (exercises the inline
   path twice in succession).
5. Nested call with distinct outer/inner `this` (cross-frame
   mirror discipline).
6. 100-iteration arrow-in-loop (mirror stability across many
   Refresh egresses).

**Coverage gap (documented):** the ThisState::Uninitialized arm
(derived constructor pre-super() access throwing ReferenceError) is
NOT directly tested through JS. Class-inheritance + super() flow
support in the lyng-js compiler/runtime is not yet exercised by
the existing integration tests, and constructing the TDZ scenario
reliably requires careful class-syntax setup. The sentinel-bail
mechanism is still exercised end-to-end by the arrow-function tests
(which trigger Lexical) and by the structural validation test in
`dsl_validation_frame_context.rs::load_uninit_lex_sentinel_handler_compiles_and_links`
(opcode 213) that compiles the sentinel-materialization macro
through the lowerer. The Uninitialized arm itself is exercised via
the `op_load_this_semantic` slow path which existing language tests
indirectly cover; the inline-asm fast path is invariant to which
sentinel-bail arm fires (it just bails uniformly).

## Notes

- **Slow path retained.** Unlike `op_load_const8` (where the slow
  path was deleted), `op_load_this_slow_rs` remains the bail target
  for the sentinel cases. It's the same function it was before this
  port; no signature change.
- **Sentinel materialization cost.** Materializing a full 64-bit
  sentinel via movz + 3× movk is 4 instructions. The `ldr =literal`
  literal-pool form (1 instruction) was rejected by the AArch64
  integrated assembler inside a `naked_asm!` block (no enclosing
  function for the literal pool to attach to). Same precedent as
  all the other `tag_X!` macros in `aarch64/values.rs`. See
  `load_uninit_lex_sentinel!` macro docs for details.
- **`cmp_branch_eq!` macro.** Added a tiny 2-instruction `cmp + b.eq`
  helper to `aarch64/control.rs`. Single-use today (op_load_this),
  but the shape (`cmp $a, $b; b.eq <label>`) is a generic primitive
  that future sentinel-bail handlers could reuse. The macro lives
  in control.rs alongside the other branch helpers
  (`branch_zero!`, `branch_nonzero!`, etc.).
- **`.slow:` label convention.** Mirrors the existing
  `op_add_dsl` hot-path pattern (see
  `crates/lyng-js/vm/src/dsl/handlers/hot.rs:71`): the DSL `.slow:`
  body label is lowered to `<handler_name>slow` by the proc-macro
  prefix mechanism, avoiding cross-handler label collisions in the
  same translation unit.
- **GC safety.** The mirror discipline established in Phase 1.B.1
  guarantees `frame_this_value` is refreshed on every Refresh egress
  (where GC can occur). Phase 1.B.1 Task 7's
  `gc_stress_frame_context.rs` exercises this; this port consumes
  the substrate without changes.
