# DSL operation vocabulary (AArch64)

All operations are `macro_rules!` macros in
`crates/lyng-js/vm/src/dsl/backend/aarch64/*.rs` that produce string
literals via `concat!`. The proc-macro lowerer
(`lyng-js-vm-dsl::lower`) interpolates them as `&'static str`
fragments into a single per-handler `core::arch::naked_asm!(...)`
block. Backend macros emit `{name}` placeholders for things only the
lowerer knows (handler length, slow-path shim symbols, struct field
offsets); the lowerer emits the matching `name = const ...` /
`name = sym ...` binding list inside the same `naked_asm!`.

## Pinned-register convention

From `crates/lyng-js/vm/src/dsl/reg_convention.rs`:

| Reg  | Pin                                                   |
| ---- | ----------------------------------------------------- |
| x19  | PC (`*const u8`)                                      |
| x20  | REGS (`*mut Value`)                                   |
| x21  | FV (`*mut FeedbackEntry`)                             |
| x22  | VM (`*mut Vm`)                                        |
| x23  | TABLE (`*const DslHandler`)                           |
| x24  | STATE (`*mut LlIntState`)                             |
| x9..x15 | scratch (`t0..t6`, caller-saved)                   |

Macros own `x9` (and sometimes `x10`) for internal arithmetic; the
proc-macro lowerer never assigns `t0..t6` to those registers when
operands are live.

## Bindings supplied by the proc-macro lowerer

Backend macros may reference these `{...}` placeholders. The lowerer
emits the corresponding `name = const ... / name = sym ...` entries
inside its `naked_asm!`.

| Placeholder              | Resolves to                                                              |
| ------------------------ | ------------------------------------------------------------------------ |
| `{length}`               | encoded length of the current handler (literal byte count)              |
| `{shim}`                 | `sym op_xxx_slow_rs` — Rust slow-path symbol for the current handler    |
| `{exit}`                 | `sym crate::dsl::entry::_interpreter_exit`                              |
| `{state_pc}`             | `const LLINT_STATE_FRAME_PC_OFFSET` (= 0)                               |
| `{state_pb}`             | `const LLINT_STATE_FRAME_PB_BASE` (= 8)                                 |
| `{state_regs}`           | `const LLINT_STATE_FRAME_REGS_BASE` (= 16)                              |
| `{state_fv}`             | `const LLINT_STATE_FRAME_FV_BASE` (= 24)                                |
| `{state_prefix}`         | `const LLINT_STATE_PREFIX` (= 48)                                       |
| `{vm_heap_pool}`         | `const VM_HEAP_POOL_OFFSET` (placeholder until Batch 7)                 |
| `{vm_poll}`              | `const VM_POLL_PENDING_OFFSET` (placeholder until Batch 7)              |
| `{vm_counter_base}`      | `const VM_OPCODE_COUNTER_OFFSET` (placeholder until Batch 7)            |
| `{entry_stride_shift}`   | log2(size_of::<FeedbackEntry>()) — proc-macro literal                   |
| `{entry_observed}`       | `const offset_of!(FeedbackEntry, state.observed)` — Batch 6             |
| `{record_shape}`         | `const offset_of!(ObjectRecord, shape)` — Batch 6                       |
| `{record_inline_slots}`  | `const offset_of!(ObjectRecord, inline_slots)` — Batch 6                |
| `{record_outline_slots}` | `const offset_of!(ObjectRecord, outline_slots)` — Batch 6               |
| `{arg_slot}`             | per-`mov` destination register index in `call_slow!` (lowerer-internal) |

## Operand decoding (narrow-form only — wide / extra-wide in Batch 7)

`backend/aarch64/operands.rs`

| Macro             | Layout    | Output regs                  | Fragment cost |
| ----------------- | --------- | ---------------------------- | ------------- |
| `decode_abc!`     | Abc       | a, b, c (3 byte operands)    | 3 ldrb        |
| `decode_abc_slot!`| AbcSlot   | a, b, c, slot (3 byte + u16) | 3 ldrb + ldrh |
| `decode_abx!`     | Abx       | a, bx (1 byte + 1 u16)       | ldrb + ldrh   |
| `decode_ax!`      | Ax        | ax (1 u32)                   | 1 ldr w       |
| `load_reg!`       | n/a       | Value at `[REGS + idx*8]`    | 1 ldr x       |
| `store_reg!`      | n/a       | `[REGS + idx*8] := Value`    | 1 str x       |
| `load_acc!`       | n/a       | Value at `[REGS]`            | 1 ldr x       |
| `store_acc!`      | n/a       | `[REGS] := Value`            | 1 str x       |

## Value tag checks (NaN-tagged Value)

`backend/aarch64/values.rs`

Per
[`reports/js/lyng-js/llint-dsl-value-layout.md`](../../../../reports/js/lyng-js/llint-dsl-value-layout.md),
`Value` is an 8-byte NaN-tag-space encoding:

```text
 63                51                  47                  31                                0
  |                 |                   |                   |                                 |
  0111 1111 1111 1000   kkkk kkkk kkkk kkkk   pppp pppp pppp pppp pppp pppp pppp pppp pppp pppp
  \__________ ____________/  \__________ _________/  \_______________ ______________/
             |                          |                            |
        TAG_HEADER                 TAG_KIND_MASK                  PAYLOAD_MASK
    0x7ff8_0000_0000_0000      0x0000_ffff_0000_0000          0x0000_0000_ffff_ffff
```

`TagKind` discriminator values (mirrors `lyng_js_types::Value`):

| Kind                     | Disc. | Pattern (high 32 bits)   |
| ------------------------ | -----:| ------------------------ |
| Undefined                | 1     | `0x7ff8_0001`            |
| Null                     | 2     | `0x7ff8_0002`            |
| Boolean                  | 3     | `0x7ff8_0003`            |
| Smi                      | 4     | `0x7ff8_0004`            |
| ObjectRef                | 5     | `0x7ff8_0005`            |
| StringRef                | 6     | `0x7ff8_0006`            |
| SymbolRef                | 7     | `0x7ff8_0007`            |
| BigIntRef                | 8     | `0x7ff8_0008`            |
| Sentinel                 | 9     | `0x7ff8_0009`            |
| SuspendedExecutionRef    | 10    | `0x7ff8_000a`            |

| Macro                 | Effect                                              | Fast-path cost (AArch64)     |
| --------------------- | --------------------------------------------------- | ---------------------------- |
| `check_smi!`          | Branch to label if `reg` is not an SMI              | mov + movk + and + mov + movk + cmp + b.ne (constants hoisted: AND + CMP + B.NE) |
| `check_object_ref!`   | Branch to label if `reg` is not an ObjectRef        | same shape, kind = 5         |
| `check_string_ref!`   | Branch to label if `reg` is not a StringRef         | same shape, kind = 6         |
| `check_undefined!`    | Branch to label if `reg` is not `undefined`         | mov + movk + cmp + b.ne (CMP + B.NE) |
| `check_null!`         | Branch to label if `reg` is not `null`              | same shape                   |
| `check_bool!`         | Branch to label if `reg` is not a Boolean           | AND + CMP + B.NE             |
| `check_double!`       | Branch to label if `reg` *is* tagged (i.e. not double) | LSR + CMP + B.EQ          |
| `untag_smi!`          | Sign-extend low 32 bits in-place                    | 1 sxtw                       |
| `untag_object_ref!`   | Zero-extend low 32 bits in-place                    | 1 uxtw                       |
| `untag_bool!`         | Mask low bit in-place                               | 1 and                        |
| `tag_smi!`            | OR header+kind with payload                         | mov + movk + orr (constants hoisted: 1 orr) |
| `tag_object_ref!`     | OR header+kind=5 with payload                       | same shape                   |
| `tag_undefined!`      | Materialize `0x7ff8_0001_0000_0000`                 | mov + movk                   |
| `tag_null!`           | Materialize `0x7ff8_0002_0000_0000`                 | mov + movk                   |
| `tag_bool_const!`     | Materialize a constant true/false bit pattern       | mov + movk + movk            |
| `tag_smi_const!`      | Materialize a tagged SMI carrying a literal payload | mov + movk + movk            |
| `tag_smi_from_signed_byte!` | Sign-extend an i8 (low byte of `$reg`, zero-extended by `ldrb`) to i32, then tag as SMI. Used by `op_load_smi8`. Distinct from `tag_smi!` (in-register i32 payload) and `tag_smi_const!` (compile-time literal payload). | sxtb + uxtw + movz + movk + orr (5 instr) |

## Object-record access (two-load indirection)

`backend/aarch64/objects.rs`

| Macro                        | Effect                                                | Cost   |
| ---------------------------- | ----------------------------------------------------- | ------ |
| `load_object_record!`        | `ObjectRef → *const ObjectRecord` via heap pool       | 2 ldr  |
| `load_record_shape!`         | Read 32-bit Shape ID from a record                    | 1 ldr w |
| `load_record_inline_slot!`   | Read Value from inline slot N                         | add + ldr x |
| `store_record_inline_slot!`  | Write Value into inline slot N                        | add + str x |
| `load_record_outline_slots!` | Read outline-slots base pointer from a record         | 1 ldr  |
| `load_outline_slot!`         | Read Value from outline slot N given base pointer     | 1 ldr  |

## Arithmetic (SMI fast path)

`backend/aarch64/arithmetic.rs`

All inputs **must already be untagged** (sign-extended i64s for the
signed ops, zero-extended u32s for the bitwise ops). Outputs leave a
sign-extended i64 ready for `tag_smi!`.

| Macro                  | Effect                                          | Cost |
| ---------------------- | ----------------------------------------------- | ---- |
| `add_smi_overflow!`    | 32-bit signed add; branch on overflow           | adds + b.vs + sxtw |
| `sub_smi_overflow!`    | 32-bit signed sub; branch on overflow           | subs + b.vs + sxtw |
| `mul_smi_overflow!`    | Widening signed multiply + overflow check       | smull + sxtw + cmp + b.ne |
| `bit_and_smi!`         | 32-bit bitwise AND                              | and + sxtw |
| `bit_or_smi!`          | 32-bit bitwise OR                               | orr + sxtw |
| `bit_xor_smi!`         | 32-bit bitwise XOR                              | eor + sxtw |
| `shift_left_smi!`      | `<<` with low-5 mask of shift count             | and + lsl + sxtw |
| `shift_right_smi!`     | `>>` (arithmetic) with low-5 mask               | and + asr + sxtw |
| `ushift_right_smi!`    | `>>>` (logical) with low-5 mask                 | and + lsr + uxtw |
| `neg_smi_overflow!`    | Negate; branch on `i32::MIN` overflow           | negs + b.vs + sxtw |
| `bit_not_smi!`         | Bitwise NOT (in place)                          | mvn + sxtw |
| `inc_smi_overflow!`    | Increment by 1 (12-bit imm); branch on overflow | adds + b.vs + sxtw |
| `dec_smi_overflow!`    | Decrement by 1 (12-bit imm); branch on overflow | subs + b.vs + sxtw |

## Dispatch and slow-path bridge

`backend/aarch64/control.rs`

| Macro                       | Effect                                                                                       | Fragment cost |
| --------------------------- | -------------------------------------------------------------------------------------------- | ------------- |
| `dispatch!()`               | Tail-jump with auto-advance by `{length}`                                                    | add + ldrb + ldr + br |
| `dispatch!(advance = N)`    | Tail-jump with explicit advance                                                              | same          |
| `call_slow!(shim, args)`    | Bridge to Rust slow-path; sync `state.frame_pc_offset`, mov a0..aN, `bl`                     | ldr + sub + str + mov*N + bl |
| `dispatch_after_slow!()`    | Branch on shim's return tag (Continue / Refresh / Exit)                                      | cbnz + ldr + add + ldrb + ldr + br (Continue path); refresh adds 2 ldr |
| `dispatch_prefixed!(kind=)` | Prefix-byte handler: stash kind, advance 1, dispatch next opcode                             | ldrb + cbnz + mov + strb + add + ldrb + ldr + br |
| `branch_zero!`              | `cbz xR, label`                                                                              | 1 cbz |
| `branch_nonzero!`           | `cbnz xR, label`                                                                             | 1 cbnz |
| `branch!`                   | Unconditional `b label`                                                                      | 1 b |
| `label!`                    | Emit a local label                                                                           | 0 instructions |

## Feedback (IC sites)

`backend/aarch64/feedback.rs`

`FV` (`x21`) holds the `Box<[FeedbackEntry]>` base for the current
function. Each `FeedbackEntry` carries `Option<FeedbackSiteState>` with
all Phase 3f packed sidecars.

| Macro                  | Effect                                                          | Cost |
| ---------------------- | --------------------------------------------------------------- | ---- |
| `load_feedback_site!`  | Compute pointer to FeedbackEntry at slot N                      | lsl + add |
| `record_smi!`          | OR observed-types bit 0 (SMI) in place                          | lsl + add + ldr + orr + str |
| `record_object!`       | OR observed-types bit 1 (Object) in place                       | same shape |
| `record_double!`       | OR observed-types bit 2 (Double) in place                       | same shape |

## Safepoint

`backend/aarch64/safepoint.rs`

| Macro             | Effect                                                  | Cost |
| ----------------- | ------------------------------------------------------- | ---- |
| `poll_safepoint!` | Branch to label if VM safepoint flag is set             | ldrb + cbnz |

## Raw memory loads / stores

`backend/aarch64/memory.rs` — utility fragments for one-off byte
fetches. Most callers prefer the named macros above (which carry
domain semantics) over these raw forms.

| Macro          | Effect                          | Cost |
| -------------- | ------------------------------- | ---- |
| `load_byte!`   | `ldrb wDst, [xBase, #off]`      | 1 ldrb |
| `load_half!`   | `ldrh wDst, [xBase, #off]`      | 1 ldrh |
| `load_word!`   | `ldr wDst, [xBase, #off]`       | 1 ldr w |
| `load_quad!`   | `ldr xDst, [xBase, #off]`       | 1 ldr x |
| `store_byte!`  | `strb wSrc, [xBase, #off]`      | 1 strb |
| `store_half!`  | `strh wSrc, [xBase, #off]`      | 1 strh |
| `store_word!`  | `str wSrc, [xBase, #off]`       | 1 str w |
| `store_quad!`  | `str xSrc, [xBase, #off]`       | 1 str x |

## Opcode counters (feature-gated)

`backend/aarch64/counters.rs`

| Macro          | `--features opcode-counters`           | Default                |
| -------------- | -------------------------------------- | ---------------------- |
| `inc_counter!` | 4 instr (ldr + ldr + add + str)        | empty string (0 instr) |

## Raw asm escape hatch

`backend/mod.rs`

`raw_asm!(literal)` expands to `core::arch::asm!(literal)` and is for
hand-written bridge code that needs to drop into asm at a *non*-naked
Rust call site (e.g. slow-path shims that hop briefly into asm before
returning). The proc-macro never uses this — it only emits
`naked_asm!`. Use sparingly.
