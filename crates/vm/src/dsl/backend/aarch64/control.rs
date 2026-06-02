//! Dispatch, branches, and slow-path bridge fragments for `AArch64`.
//!
//! This module is the most-used backend file: every inline `LLInt` handler
//! ends in [`dispatch!`], every slow-path handler ends in
//! [`dispatch_after_slow!`], and the prefix opcodes (`op_wide` /
//! `op_extra_wide`) use [`dispatch_prefixed!`].
//!
//! ## Pinned-register convention (from `reg_convention.rs`)
//!
//! | Reg  | Pin                                  |
//! | ---- | ------------------------------------ |
//! | x19  | PC (`*const u8`)                     |
//! | x20  | REGS (`*mut Value`)                  |
//! | x21  | MT (`*mut u8`, `MetadataTable` buffer) |
//! | x22  | VM (`*mut Vm`)                       |
//! | x23  | TABLE (`*const DslHandler`)          |
//! | x24  | STATE (`*mut LlIntState`)            |
//!
//! Internal scratch use is on `x16`/`x17` (AAPCS64 IP0/IP1) and
//! `w8`/`x8` (call-clobbered). The proc-macro lowerer never assigns
//! live operand bindings to those registers.
//!
//! ## Bindings expected from the proc-macro lowerer
//!
//! Each macro emits a fragment with `{...}` placeholders that the
//! enclosing `naked_asm!(...)` must resolve. Bindings used here:
//!
//! - `{length}`     — encoded length of the current handler.
//! - `<shim>`       — `sym op_xxx_slow_rs` (per-call-site, lowerer
//!   collects shim refs from `call_slow!(...)` invocations and emits
//!   one `<shim> = sym <shim>` binding for each).
//! - `{exit}`       — `sym crate::dsl::entry::_interpreter_exit`.
//! - `{state_pc}`   — `const LLINT_STATE_FRAME_PC_OFFSET`.
//! - `{state_pb}`   — `const LLINT_STATE_FRAME_PB_BASE`.
//! - `{state_regs}` — `const LLINT_STATE_FRAME_REGS_BASE`.
//! - `{state_mt}`   — `const LLINT_STATE_FRAME_METADATA_TABLE_BASE`.
//! - `{state_prefix}` — `const LLINT_STATE_PREFIX`.

// ===========================================================================
// Tail-jump dispatch.
// ===========================================================================

/// End-of-handler dispatch with auto-advance by `{length}` bytes (the
/// handler's encoded length, supplied by the proc-macro at lower time).
///
/// Compiles to 4 instructions:
/// ```text
///     add  x19, x19, {length}    ; advance PC
///     ldrb w8, [x19]             ; load next opcode byte
///     ldr  x16, [x23, x8, lsl #3] ; look up handler addr
///     br   x16                    ; tail-jump
/// ```
#[macro_export]
macro_rules! dispatch {
    () => {
        concat!(
            "add    x19, x19, {length}\n",
            "ldrb   w8, [x19]\n",
            "ldr    x16, [x23, x8, lsl #3]\n",
            "br     x16\n",
        )
    };
    (advance = $n:literal) => {
        concat!(
            "add    x19, x19, #",
            stringify!($n),
            "\n",
            "ldrb   w8, [x19]\n",
            "ldr    x16, [x23, x8, lsl #3]\n",
            "br     x16\n",
        )
    };
}

// ===========================================================================
// Slow-path bridge.
// ===========================================================================

/// Call into a Rust slow-path shim. The shim signature is
/// `extern "C" fn(state: *mut LlIntState, op0: u32, ...) -> SlowPathReturn`.
///
/// **Pre-call refresh** (asm-visible discipline per design §7):
///
/// 1. Compute `pc_offset = PC - pb_base` and store at
///    `state.frame_pc_offset` so the shim can inspect the bytecode
///    cursor.
/// 2. Move `STATE` to a0 (`x0`).
/// 3. Move each operand reg into the corresponding argument slot
///    (a1..a5 = `w1..w5`).
/// 4. `bl {<shim>}`. The lowerer emits a `<shim> = sym <shim>` named
///    binding so the asm references the linker symbol.
///
/// Up to 5 positional args after the `STATE` pointer. The macro
/// expands one match arm per arity, hardcoding `w1..w5` so the
/// per-operand mov has the correct destination register.
///
/// ## Opcode-byte injection
///
/// The lowerer appends `, opcode_byte = <N>` to every `call_slow!(...)`
/// invocation. The macro emits `inc_slow_semantic_counter!(N)` before
/// the bridge, bumping the slow-semantic bank slot for this opcode.
///
/// Each arity has two match arms:
/// - With `opcode_byte = N` (preferred — emitted by the lowerer).
/// - Without `opcode_byte` (fallback for hand-written call sites; emits
///   no counter inc).
///
/// Bindings: `<shim>` (per call site), `{state_pb}`, `{state_pc}`,
/// `{vm_counter_base}` (when `diagnostic-counters` feature is on).
#[macro_export]
macro_rules! call_slow {
    // ---- 0 args ----
    ($shim:ident, args = [], opcode_byte = $op:literal) => {
        concat!(
            $crate::inc_slow_semantic_counter!($op),
            "ldr    x16, [x24, {state_pb}]\n",
            "sub    x17, x19, x16\n",
            "str    w17, [x24, {state_pc}]\n",
            "mov    x0, x24\n",
            "bl     {",
            stringify!($shim),
            "}\n",
        )
    };
    ($shim:ident, args = []) => {
        concat!(
            "ldr    x16, [x24, {state_pb}]\n",
            "sub    x17, x19, x16\n",
            "str    w17, [x24, {state_pc}]\n",
            "mov    x0, x24\n",
            "bl     {",
            stringify!($shim),
            "}\n",
        )
    };
    // ---- 1 arg ----
    ($shim:ident, args = [$a:tt], opcode_byte = $op:literal) => {
        concat!(
            $crate::inc_slow_semantic_counter!($op),
            "ldr    x16, [x24, {state_pb}]\n",
            "sub    x17, x19, x16\n",
            "str    w17, [x24, {state_pc}]\n",
            "mov    x0, x24\n",
            "mov    w1, w",
            stringify!($a),
            "\n",
            "bl     {",
            stringify!($shim),
            "}\n",
        )
    };
    ($shim:ident, args = [$a:tt]) => {
        concat!(
            "ldr    x16, [x24, {state_pb}]\n",
            "sub    x17, x19, x16\n",
            "str    w17, [x24, {state_pc}]\n",
            "mov    x0, x24\n",
            "mov    w1, w",
            stringify!($a),
            "\n",
            "bl     {",
            stringify!($shim),
            "}\n",
        )
    };
    // ---- 2 args ----
    ($shim:ident, args = [$a:tt, $b:tt], opcode_byte = $op:literal) => {
        concat!(
            $crate::inc_slow_semantic_counter!($op),
            "ldr    x16, [x24, {state_pb}]\n",
            "sub    x17, x19, x16\n",
            "str    w17, [x24, {state_pc}]\n",
            "mov    x0, x24\n",
            "mov    w1, w",
            stringify!($a),
            "\n",
            "mov    w2, w",
            stringify!($b),
            "\n",
            "bl     {",
            stringify!($shim),
            "}\n",
        )
    };
    ($shim:ident, args = [$a:tt, $b:tt]) => {
        concat!(
            "ldr    x16, [x24, {state_pb}]\n",
            "sub    x17, x19, x16\n",
            "str    w17, [x24, {state_pc}]\n",
            "mov    x0, x24\n",
            "mov    w1, w",
            stringify!($a),
            "\n",
            "mov    w2, w",
            stringify!($b),
            "\n",
            "bl     {",
            stringify!($shim),
            "}\n",
        )
    };
    // ---- 3 args ----
    ($shim:ident, args = [$a:tt, $b:tt, $c:tt], opcode_byte = $op:literal) => {
        concat!(
            $crate::inc_slow_semantic_counter!($op),
            "ldr    x16, [x24, {state_pb}]\n",
            "sub    x17, x19, x16\n",
            "str    w17, [x24, {state_pc}]\n",
            "mov    x0, x24\n",
            "mov    w1, w",
            stringify!($a),
            "\n",
            "mov    w2, w",
            stringify!($b),
            "\n",
            "mov    w3, w",
            stringify!($c),
            "\n",
            "bl     {",
            stringify!($shim),
            "}\n",
        )
    };
    ($shim:ident, args = [$a:tt, $b:tt, $c:tt]) => {
        concat!(
            "ldr    x16, [x24, {state_pb}]\n",
            "sub    x17, x19, x16\n",
            "str    w17, [x24, {state_pc}]\n",
            "mov    x0, x24\n",
            "mov    w1, w",
            stringify!($a),
            "\n",
            "mov    w2, w",
            stringify!($b),
            "\n",
            "mov    w3, w",
            stringify!($c),
            "\n",
            "bl     {",
            stringify!($shim),
            "}\n",
        )
    };
    // ---- 4 args ----
    ($shim:ident, args = [$a:tt, $b:tt, $c:tt, $d:tt], opcode_byte = $op:literal) => {
        concat!(
            $crate::inc_slow_semantic_counter!($op),
            "ldr    x16, [x24, {state_pb}]\n",
            "sub    x17, x19, x16\n",
            "str    w17, [x24, {state_pc}]\n",
            "mov    x0, x24\n",
            "mov    w1, w",
            stringify!($a),
            "\n",
            "mov    w2, w",
            stringify!($b),
            "\n",
            "mov    w3, w",
            stringify!($c),
            "\n",
            "mov    w4, w",
            stringify!($d),
            "\n",
            "bl     {",
            stringify!($shim),
            "}\n",
        )
    };
    ($shim:ident, args = [$a:tt, $b:tt, $c:tt, $d:tt]) => {
        concat!(
            "ldr    x16, [x24, {state_pb}]\n",
            "sub    x17, x19, x16\n",
            "str    w17, [x24, {state_pc}]\n",
            "mov    x0, x24\n",
            "mov    w1, w",
            stringify!($a),
            "\n",
            "mov    w2, w",
            stringify!($b),
            "\n",
            "mov    w3, w",
            stringify!($c),
            "\n",
            "mov    w4, w",
            stringify!($d),
            "\n",
            "bl     {",
            stringify!($shim),
            "}\n",
        )
    };
    // ---- 5 args ----
    ($shim:ident, args = [$a:tt, $b:tt, $c:tt, $d:tt, $e:tt], opcode_byte = $op:literal) => {
        concat!(
            $crate::inc_slow_semantic_counter!($op),
            "ldr    x16, [x24, {state_pb}]\n",
            "sub    x17, x19, x16\n",
            "str    w17, [x24, {state_pc}]\n",
            "mov    x0, x24\n",
            "mov    w1, w",
            stringify!($a),
            "\n",
            "mov    w2, w",
            stringify!($b),
            "\n",
            "mov    w3, w",
            stringify!($c),
            "\n",
            "mov    w4, w",
            stringify!($d),
            "\n",
            "mov    w5, w",
            stringify!($e),
            "\n",
            "bl     {",
            stringify!($shim),
            "}\n",
        )
    };
    ($shim:ident, args = [$a:tt, $b:tt, $c:tt, $d:tt, $e:tt]) => {
        concat!(
            "ldr    x16, [x24, {state_pb}]\n",
            "sub    x17, x19, x16\n",
            "str    w17, [x24, {state_pc}]\n",
            "mov    x0, x24\n",
            "mov    w1, w",
            stringify!($a),
            "\n",
            "mov    w2, w",
            stringify!($b),
            "\n",
            "mov    w3, w",
            stringify!($c),
            "\n",
            "mov    w4, w",
            stringify!($d),
            "\n",
            "mov    w5, w",
            stringify!($e),
            "\n",
            "bl     {",
            stringify!($shim),
            "}\n",
        )
    };
}

/// Call into a Rust probe without incrementing semantic slow-path
/// counters. The probe returns `tag == 0` on hit with the
/// next PC offset in `payload`; non-zero means the handler should fall
/// through to its counted semantic slow path.
///
/// This mirrors `call_slow!`'s pre-call PC sync so the probe can inspect
/// the current bytecode offset, but it deliberately does not bump the
/// semantic slow-path counter.
#[macro_export]
macro_rules! call_rust_probe {
    ($shim:ident, args = [$a:tt, $b:tt]) => {
        concat!(
            "ldr    x16, [x24, {state_pb}]\n",
            "sub    x17, x19, x16\n",
            "str    w17, [x24, {state_pc}]\n",
            "mov    x0, x24\n",
            "mov    w1, w",
            stringify!($a),
            "\n",
            "mov    w2, w",
            stringify!($b),
            "\n",
            "bl     {",
            stringify!($shim),
            "}\n",
        )
    };
    ($shim:ident, args = [$a:tt, $b:tt, $c:tt, $d:tt]) => {
        concat!(
            "ldr    x16, [x24, {state_pb}]\n",
            "sub    x17, x19, x16\n",
            "str    w17, [x24, {state_pc}]\n",
            "mov    x0, x24\n",
            "mov    w1, w",
            stringify!($a),
            "\n",
            "mov    w2, w",
            stringify!($b),
            "\n",
            "mov    w3, w",
            stringify!($c),
            "\n",
            "mov    w4, w",
            stringify!($d),
            "\n",
            "bl     {",
            stringify!($shim),
            "}\n",
        )
    };
}

/// Post-`call_slow!` dispatch.
///
/// The shim returns a tagged u64:
///
/// - `0x0` (Continue, low 32 = new `pc_offset`) → reload `PC` and
///   tail-jump to the next handler.
/// - `0x1` (Refresh, low 32 = new `pc_offset`) → reload PC, REGS, FV
///   from `state.frame_*` (a frame switch happened) and tail-jump.
/// - `0x2` (Exit) → branch to `{exit}` symbol.
///
/// Uses `AArch64` numeric local labels (`1:` / `1f` / `2:` / `2f`)
/// rather than named `L*` labels. Numeric locals are per-block-of-asm
/// scoped on Apple's assembler, so multiple handlers in the same
/// translation unit don't collide on label names. Named labels
/// (`Lname`) coalesce across `naked_asm!` blocks in the same crate's
/// `.s` output and trip the assembler with "symbol already defined".
///
/// Bindings: `{exit}`, `{state_pb}`, `{state_pc}`, `{state_regs}`,
/// `{state_mt}`.
#[macro_export]
macro_rules! dispatch_after_slow {
    () => {
        concat!(
            // Common case first: tag == Continue (0).
            "cbnz   x0, 1f\n", // → "unusual" handling
            // Continue path: PC = pb_base + new_offset (low 32 of x1).
            // Reload REGS / MT from state: translate_outcome re-derives the
            // per-frame REGS offset on every Continue egress and refreshes
            // state.frame_regs_base / state.frame_metadata_table_base from
            // the live VM; we reload x20 / x21 here so the next handler
            // sees the up-to-date pins.
            "ldr    x16, [x24, {state_pb}]\n",
            "add    x19, x16, x1\n",
            "ldr    x20, [x24, {state_regs}]\n",
            "ldr    x21, [x24, {state_mt}]\n",
            "ldrb   w8, [x19]\n",
            "ldr    x17, [x23, x8, lsl #3]\n",
            "br     x17\n",
            "1:\n", // unusual:
            "cmp    x0, #2\n",
            "b.eq   2f\n", // → exit
            // Refresh path: reload PC / REGS / MT from state.frame_*.
            "ldr    w16, [x24, {state_pc}]\n",
            "ldr    x17, [x24, {state_pb}]\n",
            "add    x19, x17, x16\n",
            "ldr    x20, [x24, {state_regs}]\n",
            "ldr    x21, [x24, {state_mt}]\n",
            "ldrb   w8,  [x19]\n",
            "ldr    x17, [x23, x8, lsl #3]\n",
            "br     x17\n",
            "2:\n", // exit:
            "b      {exit}\n",
        )
    };
}

/// Dispatch after a Rust probe hit returned `tag == 0` with the next-PC
/// offset in `x1`.
///
/// This is a no-refresh dispatch form. It is valid only for probe-hit
/// helpers whose hit contract guarantees no frame switch, no
/// register-stack relocation, and no feedback-vector relocation. Those
/// probes may inspect or mutate the active frame's current registers and
/// feedback data, then advance the PC, but they must not enter guest
/// bytecode, call host code, or take an allocation/GC path that can move
/// the register stack or feedback vector. Misses branch to the normal
/// counted semantic slow path, where `dispatch_after_slow!` handles
/// Continue/Refresh/Exit.
#[macro_export]
macro_rules! dispatch_probe_hit_no_refresh {
    // No-refresh contract: no frame switch, no register-stack relocation,
    // and no feedback-vector relocation while the Rust probe hit helper runs.
    () => {
        concat!(
            "ldr    x16, [x24, {state_pb}]\n",
            "add    x19, x16, x1\n",
            "ldrb   w8, [x19]\n",
            "ldr    x17, [x23, x8, lsl #3]\n",
            "br     x17\n",
        )
    };
}

// ===========================================================================
// Branches.
// ===========================================================================

/// Branch to `$label` if `$reg` is zero (`cbz xR, label`).
#[macro_export]
macro_rules! branch_zero {
    ($reg:tt, $label:tt) => {
        concat!("cbz    x", stringify!($reg), ", ", stringify!($label), "\n",)
    };
}

/// Branch to `$label` if `$reg` is non-zero (`cbnz xR, label`).
#[macro_export]
macro_rules! branch_nonzero {
    ($reg:tt, $label:tt) => {
        concat!("cbnz   x", stringify!($reg), ", ", stringify!($label), "\n",)
    };
}

/// Unconditional branch to a local label.
#[macro_export]
macro_rules! branch {
    ($label:tt) => {
        concat!("b      ", stringify!($label), "\n",)
    };
}

/// Branch to `$label` when `$reg` holds an unsigned byte whose signed
/// i8 interpretation is negative.
#[macro_export]
macro_rules! branch_i8_negative {
    ($reg:tt, $label:tt) => {
        concat!(
            "tbnz   w",
            stringify!($reg),
            ", #7, ",
            stringify!($label),
            "\n",
        )
    };
}

/// Branch to `$label` when `$reg` holds a zero-extended halfword whose
/// signed i16 interpretation is negative.
///
/// `decode_abx!` loads the Abx layout's `bx` operand with `ldrh` (zero-
/// extending the 16-bit field into the low half of the scratch w-reg),
/// so the i16 sign bit is bit 15 of the operand register. Mirrors the
/// `branch_i8_negative!` pattern (tbnz on the sign bit) without
/// committing to a sign-extension, which the matching
/// `jump_relative_i16_and_dispatch!` performs on its own.
///
/// Compiles to 1 instruction (`tbnz`). Uses no scratch registers.
#[macro_export]
macro_rules! branch_i16_negative {
    ($reg:tt, $label:tt) => {
        concat!(
            "tbnz   w",
            stringify!($reg),
            ", #15, ",
            stringify!($label),
            "\n",
        )
    };
}

#[macro_export]
macro_rules! branch_i32_negative {
    ($reg:tt, $label:tt) => {
        concat!(
            "tbnz   w",
            stringify!($reg),
            ", #31, ",
            stringify!($label),
            "\n",
        )
    };
}

/// Apply a sign-extended i8 relative branch delta in `$offset` from the
/// current instruction and dispatch from the resulting PC.
#[macro_export]
macro_rules! jump_relative_i8_and_dispatch {
    ($offset:tt, advance = $n:literal) => {
        concat!(
            "sxtb   x",
            stringify!($offset),
            ", w",
            stringify!($offset),
            "\n",
            "add    x19, x19, #",
            stringify!($n),
            "\n",
            "add    x19, x19, x",
            stringify!($offset),
            "\n",
            "ldrb   w8, [x19]\n",
            "ldr    x16, [x23, x8, lsl #3]\n",
            "br     x16\n",
        )
    };
}

/// Apply a sign-extended i16 relative branch delta in `$offset` from
/// the current instruction and dispatch from the resulting PC.
///
/// `decode_abx!` zero-extends the 16-bit operand into the low half of
/// the scratch w-reg, so `sxth` widens it to i64 before adding to PC.
/// Mirrors `jump_relative_i8_and_dispatch!` (which uses `sxtb`) and
/// `jump_relative_i32_and_dispatch!` (which uses the `sxtw` extended
/// add form). Both forms emit a 6-instruction tail: sign-extend +
/// advance + delta-add + ldrb opcode + indexed table load + br.
///
/// Scratch use: `x8` (next opcode byte) and `x16` (next handler addr);
/// both are AAPCS64 call-clobbered IP slots that the lowerer never
/// assigns to live operands.
#[macro_export]
macro_rules! jump_relative_i16_and_dispatch {
    ($offset:tt, advance = $n:literal) => {
        concat!(
            "sxth   x",
            stringify!($offset),
            ", w",
            stringify!($offset),
            "\n",
            "add    x19, x19, #",
            stringify!($n),
            "\n",
            "add    x19, x19, x",
            stringify!($offset),
            "\n",
            "ldrb   w8, [x19]\n",
            "ldr    x16, [x23, x8, lsl #3]\n",
            "br     x16\n",
        )
    };
}

/// Apply a sign-extended i32 relative branch delta in `$offset` from
/// the current instruction and dispatch from the resulting PC.
#[macro_export]
macro_rules! jump_relative_i32_and_dispatch {
    ($offset:tt, advance = $n:literal) => {
        concat!(
            "add    x19, x19, #",
            stringify!($n),
            "\n",
            "add    x19, x19, w",
            stringify!($offset),
            ", sxtw\n",
            "ldrb   w8, [x19]\n",
            "ldr    x16, [x23, x8, lsl #3]\n",
            "br     x16\n",
        )
    };
}

/// Emit a local label inside the handler body.
#[macro_export]
macro_rules! label {
    ($label:tt) => {
        concat!(stringify!($label), ":\n",)
    };
}

/// Compare two registers and branch to `$label` if equal.
///
/// Two instructions: `cmp x{a}, x{b}; b.eq {label}`. Used to bail to
/// the slow path when `frame_this_value` holds the
/// `Value::uninitialized_lexical()` sentinel.
#[macro_export]
macro_rules! cmp_branch_eq {
    ($a:tt, $b:tt, $label:tt) => {
        concat!(
            "cmp    x",
            stringify!($a),
            ", x",
            stringify!($b),
            "\n",
            "b.eq   ",
            stringify!($label),
            "\n",
        )
    };
}

/// Compare two registers and branch to `$label` if not equal.
#[macro_export]
macro_rules! cmp_branch_ne {
    ($a:tt, $b:tt, $label:tt) => {
        concat!(
            "cmp    x",
            stringify!($a),
            ", x",
            stringify!($b),
            "\n",
            "b.ne   ",
            stringify!($label),
            "\n",
        )
    };
}

/// Compare two registers and materialize the equality result as a 0/1
/// payload in `$dst`.
///
/// Pair with `tag_bool_payload!($dst)` before storing the result as a
/// JavaScript Boolean `Value`.
#[macro_export]
macro_rules! cmp_eq_payload {
    ($a:tt, $b:tt => $dst:tt) => {
        concat!(
            "cmp    x",
            stringify!($a),
            ", x",
            stringify!($b),
            "\n",
            "cset   w",
            stringify!($dst),
            ", eq\n",
        )
    };
}

// ===========================================================================
// Prefix dispatch (op_wide / op_extra_wide).
// ===========================================================================

/// Prefix-opcode handler body. Stashes the prefix kind in
/// `state.prefix`, advances PC by 1, and tail-jumps to the next
/// opcode's handler. Rejects a doubled prefix (e.g. `wide wide`) by
/// dropping into a slow-path shim.
///
/// `$kind` is the prefix discriminator (1 for `wide`, 2 for
/// `extra_wide`) — a literal u8.
///
/// **TODO:** `Ldouble_prefix` currently hits `brk #0` because
/// `op_double_prefix_slow_rs` doesn't exist yet. Replace with
/// `call_slow!(op_double_prefix_slow_rs, args = [])` + a proper exit path.
///
/// Bindings: `{state_prefix}`.
#[macro_export]
macro_rules! dispatch_prefixed {
    (kind = $kind:literal) => {
        concat!(
            // Reject doubled prefix.
            "ldrb   w16, [x24, {state_prefix}]\n",
            "cbnz   w16, 1f\n", // → double-prefix slow path
            // Stash prefix discriminator.
            "mov    w16, #",
            stringify!($kind),
            "\n",
            "strb   w16, [x24, {state_prefix}]\n",
            // Advance PC past the prefix byte and dispatch.
            "add    x19, x19, #1\n",
            "ldrb   w8, [x19]\n",
            "ldr    x17, [x23, x8, lsl #3]\n",
            "br     x17\n",
            "1:\n", // double-prefix:
            // TODO: replace with call_slow!(op_double_prefix_slow_rs, args = []).
            "brk    #0\n",
        )
    };
}
