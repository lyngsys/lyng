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
//! | x21  | FV (`*mut FeedbackEntry`)            |
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
//! - `{state_fv}`   — `const LLINT_STATE_FRAME_FV_BASE`.
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
/// ## Opcode-byte injection (DSL-1 Phase 1.B.0 Task 5)
///
/// The lowerer rewrites every `call_slow!(...)` body invocation to
/// append `, opcode_byte = <N>` — the handler's own opcode discriminant
/// from its `llint_handler!` signature. The macro emits
/// `inc_slow_semantic_counter!(N)` before the rest of the bridge,
/// bumping the slow-semantic bank slot for this opcode whenever the
/// inline hit path falls through (or always, for cold-stub-only opcodes).
///
/// Each arity has two match arms:
/// - With `opcode_byte = N` (preferred — emitted by the lowerer).
/// - Without `opcode_byte` (fallback — only fires if a hand-written
///   call site bypasses the lowerer; emits no counter inc).
///
/// Bindings: `<shim>` (per call site), `{state_pb}`, `{state_pc}`,
/// `{vm_counter_base}` (when `opcode-counters` feature is on).
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
/// `{state_fv}`.
#[macro_export]
macro_rules! dispatch_after_slow {
    () => {
        concat!(
            // Common case first: tag == Continue (0).
            "cbnz   x0, 1f\n", // → "unusual" handling
            // Continue path: PC = pb_base + new_offset (low 32 of x1).
            // Also reload REGS / FV from state — a nested call in the
            // slow path (e.g. ToPrimitive invoking valueOf bytecode)
            // can resize `Vm::register_stack`, leaving the asm-side
            // REGS pin (`x20`) pointing into freed storage even when
            // frame depth is unchanged. The Rust-side
            // `translate_outcome` Continue arm refreshes
            // `state.frame_regs_base` / `state.frame_fv_base` from the
            // live VM on every Continue egress; we reload x20 / x21
            // here so the next handler sees the up-to-date pins.
            "ldr    x16, [x24, {state_pb}]\n",
            "add    x19, x16, x1\n",
            "ldr    x20, [x24, {state_regs}]\n",
            "ldr    x21, [x24, {state_fv}]\n",
            "ldrb   w8, [x19]\n",
            "ldr    x17, [x23, x8, lsl #3]\n",
            "br     x17\n",
            "1:\n", // unusual:
            "cmp    x0, #2\n",
            "b.eq   2f\n", // → exit
            // Refresh path: reload PC / REGS / FV from state.frame_*.
            "ldr    w16, [x24, {state_pc}]\n",
            "ldr    x17, [x24, {state_pb}]\n",
            "add    x19, x17, x16\n",
            "ldr    x20, [x24, {state_regs}]\n",
            "ldr    x21, [x24, {state_fv}]\n",
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

/// Complete a simple nested function return in `LLInt` asm.
///
/// The path is deliberately narrow: it handles non-entry frames whose
/// frame-info entry is marked simple-return-safe. Complex cleanup,
/// constructors, and entry-frame exits branch to the semantic path.
#[macro_export]
macro_rules! return_to_caller_or_branch {
    ($value:tt, $label:tt) => {
        concat!(
            "ldr    w0, [x24, {state_frame_depth}]\n",
            "cmp    w0, #1\n",
            "b.ls   ",
            stringify!($label),
            "\n",
            "ldr    x1, [x24, {state_frame_info_base}]\n",
            "cbz    x1, ",
            stringify!($label),
            "\n",
            "sub    w2, w0, #1\n",
            "add    x2, x1, x2, lsl #{frame_info_stride_shift}\n",
            "ldr    w3, [x2, {frame_info_flags}]\n",
            "tbz    w3, #{frame_info_fast_return_safe_bit}, ",
            stringify!($label),
            "\n",
            "ldr    w3, [x2, {frame_info_return_register}]\n",
            "cmn    w3, #1\n",
            "b.eq   ",
            stringify!($label),
            "\n",
            "sub    w0, w0, #1\n",
            "str    w0, [x24, {state_frame_depth}]\n",
            "ldr    w6, [x2, {frame_info_register_base}]\n",
            "str    w6, [x24, {state_register_stack_top}]\n",
            "sub    w4, w0, #1\n",
            "add    x4, x1, x4, lsl #{frame_info_stride_shift}\n",
            "ldr    x20, [x4, {frame_info_regs_base}]\n",
            "str    x",
            stringify!($value),
            ", [x20, x3, lsl #3]\n",
            "ldr    x2, [x4, {frame_info_pb_base}]\n",
            "ldr    w5, [x4, {frame_info_pc_offset}]\n",
            "add    x19, x2, x5\n",
            "ldr    x21, [x4, {frame_info_fv_base}]\n",
            "ldr    x6, [x4, {frame_info_const_base}]\n",
            "ldr    x7, [x4, {frame_info_this_value}]\n",
            "str    w5, [x24, {state_pc}]\n",
            "str    x2, [x24, {state_pb}]\n",
            "str    x20, [x24, {state_regs}]\n",
            "str    x21, [x24, {state_fv}]\n",
            "str    x6, [x24, {vm_const_base}]\n",
            "str    x7, [x24, {state_this_value}]\n",
            "ldrb   w8, [x19]\n",
            "ldr    x16, [x23, x8, lsl #3]\n",
            "br     x16\n",
        )
    };
}

/// Enter an eligible zero-argument bytecode callee entirely in `LLInt` asm.
///
/// The callee must have an asm-visible call-target table entry. The
/// path bails for unsupported `this` coercions, missing metadata, and
/// insufficient frame/register-stack headroom.
#[macro_export]
macro_rules! call0_bytecode_or_branch {
    ($result:tt, $callee:tt, $this_reg:tt, $label:tt) => {
        concat!(
            "mov    w13, w",
            stringify!($result),
            "\n",
            "ldr    x0, [x20, x",
            stringify!($callee),
            ", lsl #3]\n",
            "movz   x16, #0xffff, lsl #32\n",
            "movk   x16, #0x7ff8, lsl #48\n",
            "and    x16, x0, x16\n",
            "movz   x17, #0x5, lsl #32\n",
            "movk   x17, #0x7ff8, lsl #48\n",
            "cmp    x16, x17\n",
            "b.ne   ",
            stringify!($label),
            "\n",
            "uxtw   x1, w0\n",
            "ldr    x2, [x24, {state_call_targets_base}]\n",
            "cbz    x2, ",
            stringify!($label),
            "\n",
            "ldr    w3, [x24, {state_call_targets_len}]\n",
            "cmp    w1, w3\n",
            "b.hs   ",
            stringify!($label),
            "\n",
            "add    x4, x2, x1, lsl #{call_target_stride_shift}\n",
            "ldr    w5, [x4, {call_target_flags}]\n",
            "tbz    w5, #{call_target_enabled_bit}, ",
            stringify!($label),
            "\n",
            "ldr    x6, [x4, {call_target_callee_bits}]\n",
            "cmp    x0, x6\n",
            "b.ne   ",
            stringify!($label),
            "\n",
            "ldr    x7, [x20, x",
            stringify!($this_reg),
            ", lsl #3]\n",
            "tbz    w5, #{call_target_this_global_bit}, 1f\n",
            "movz   x8, #0x1, lsl #32\n",
            "movk   x8, #0x7ff8, lsl #48\n",
            "cmp    x7, x8\n",
            "b.eq   2f\n",
            "movz   x8, #0x2, lsl #32\n",
            "movk   x8, #0x7ff8, lsl #48\n",
            "cmp    x7, x8\n",
            "b.eq   2f\n",
            "movz   x16, #0xffff, lsl #32\n",
            "movk   x16, #0x7ff8, lsl #48\n",
            "and    x16, x7, x16\n",
            "movz   x17, #0x5, lsl #32\n",
            "movk   x17, #0x7ff8, lsl #48\n",
            "cmp    x16, x17\n",
            "b.ne   ",
            stringify!($label),
            "\n",
            "b      1f\n",
            "2:\n",
            "ldr    x7, [x4, {call_target_global_this}]\n",
            "1:\n",
            "ldr    w8, [x24, {state_frame_depth}]\n",
            "cbz    w8, ",
            stringify!($label),
            "\n",
            "ldr    w9, [x24, {state_frame_info_len}]\n",
            "cmp    w8, w9\n",
            "b.hs   ",
            stringify!($label),
            "\n",
            "ldr    w10, [x4, {call_target_register_len}]\n",
            "ldr    w11, [x24, {state_register_stack_top}]\n",
            "adds   w12, w11, w10\n",
            "b.cs   ",
            stringify!($label),
            "\n",
            "ldr    w1, [x24, {state_register_stack_len}]\n",
            "cmp    w12, w1\n",
            "b.hi   ",
            stringify!($label),
            "\n",
            "ldr    x14, [x24, {state_register_stack_base}]\n",
            "cbz    x14, ",
            stringify!($label),
            "\n",
            "add    x15, x14, x11, lsl #3\n",
            "movz   x6, #0x1, lsl #32\n",
            "movk   x6, #0x7ff8, lsl #48\n",
            "mov    w16, #0\n",
            "3:\n",
            "cmp    w16, w10\n",
            "b.hs   4f\n",
            "str    x6, [x15, x16, lsl #3]\n",
            "add    w16, w16, #1\n",
            "b      3b\n",
            "4:\n",
            "ldr    x2, [x24, {state_frame_info_base}]\n",
            "cbz    x2, ",
            stringify!($label),
            "\n",
            "sub    w3, w8, #1\n",
            "add    x3, x2, x3, lsl #{frame_info_stride_shift}\n",
            "ldr    x6, [x24, {state_pb}]\n",
            "sub    x16, x19, x6\n",
            "add    w16, w16, #{length}\n",
            "str    w16, [x3, {frame_info_pc_offset}]\n",
            "add    x3, x2, x8, lsl #{frame_info_stride_shift}\n",
            "ldr    x6, [x4, {call_target_pb_base}]\n",
            "str    x6, [x3, {frame_info_pb_base}]\n",
            "str    x15, [x3, {frame_info_regs_base}]\n",
            "ldr    x16, [x4, {call_target_fv_base}]\n",
            "str    x16, [x3, {frame_info_fv_base}]\n",
            "ldr    x17, [x4, {call_target_const_base}]\n",
            "str    x17, [x3, {frame_info_const_base}]\n",
            "str    x7, [x3, {frame_info_this_value}]\n",
            "str    wzr, [x3, {frame_info_pc_offset}]\n",
            "str    w13, [x3, {frame_info_return_register}]\n",
            "ubfx   w16, w5, #{call_target_fast_return_safe_bit}, #1\n",
            "ubfx   w17, w5, #{call_target_strict_bit}, #1\n",
            "orr    w16, w16, w17, lsl #{frame_info_strict_bit}\n",
            "ubfx   w17, w5, #{call_target_tail_call_recycle_safe_bit}, #1\n",
            "orr    w16, w16, w17, lsl #{frame_info_tail_call_recycle_safe_bit}\n",
            "str    w16, [x3, {frame_info_flags}]\n",
            "str    w11, [x3, {frame_info_register_base}]\n",
            "str    w10, [x3, {frame_info_register_len}]\n",
            "ldr    w16, [x4, {call_target_code_raw}]\n",
            "str    w16, [x3, {frame_info_code_raw}]\n",
            "ldr    w16, [x4, {call_target_realm_raw}]\n",
            "str    w16, [x3, {frame_info_realm_raw}]\n",
            "ldr    w16, [x4, {call_target_lexical_env_raw}]\n",
            "str    w16, [x3, {frame_info_lexical_env_raw}]\n",
            "ldr    w16, [x4, {call_target_variable_env_raw}]\n",
            "str    w16, [x3, {frame_info_variable_env_raw}]\n",
            "ldr    w16, [x4, {call_target_private_env_raw}]\n",
            "str    w16, [x3, {frame_info_private_env_raw}]\n",
            "ldr    w16, [x4, {call_target_callee_raw}]\n",
            "str    w16, [x3, {frame_info_callee_raw}]\n",
            "ldr    w16, [x4, {call_target_parameter_initializer_end_offset}]\n",
            "str    w16, [x3, {frame_info_parameter_initializer_end_offset}]\n",
            "mov    w16, #3\n",
            "str    w16, [x3, {frame_info_frame_flags_raw}]\n",
            "str    wzr, [x3, {frame_info_tail_caller_raw}]\n",
            "str    wzr, [x3, {frame_info_tail_caller_strict}]\n",
            "add    w8, w8, #1\n",
            "str    w8, [x24, {state_frame_depth}]\n",
            "str    w12, [x24, {state_register_stack_top}]\n",
            "str    wzr, [x24, {state_pc}]\n",
            "ldr    x19, [x4, {call_target_pb_base}]\n",
            "str    x19, [x24, {state_pb}]\n",
            "mov    x20, x15\n",
            "str    x20, [x24, {state_regs}]\n",
            "ldr    x21, [x4, {call_target_fv_base}]\n",
            "str    x21, [x24, {state_fv}]\n",
            "ldr    x16, [x4, {call_target_const_base}]\n",
            "str    x16, [x24, {vm_const_base}]\n",
            "str    x7, [x24, {state_this_value}]\n",
            "ldrb   w8, [x19]\n",
            "ldr    x16, [x23, x8, lsl #3]\n",
            "br     x16\n",
        )
    };
}

/// Replace the active frame with an eligible no-argument bytecode tail callee.
#[macro_export]
macro_rules! tail_call0_bytecode_or_branch {
    ($callee:tt, $this_reg:tt, $label:tt) => {
        concat!(
            "ldrh   w13, [x19, #4]\n",
            "cbnz   w13, ",
            stringify!($label),
            "\n",
            "ldr    x0, [x20, x",
            stringify!($callee),
            ", lsl #3]\n",
            "movz   x16, #0xffff, lsl #32\n",
            "movk   x16, #0x7ff8, lsl #48\n",
            "and    x16, x0, x16\n",
            "movz   x17, #0x5, lsl #32\n",
            "movk   x17, #0x7ff8, lsl #48\n",
            "cmp    x16, x17\n",
            "b.ne   ",
            stringify!($label),
            "\n",
            "uxtw   x1, w0\n",
            "ldr    x2, [x24, {state_call_targets_base}]\n",
            "cbz    x2, ",
            stringify!($label),
            "\n",
            "ldr    w3, [x24, {state_call_targets_len}]\n",
            "cmp    w1, w3\n",
            "b.hs   ",
            stringify!($label),
            "\n",
            "add    x4, x2, x1, lsl #{call_target_stride_shift}\n",
            "ldr    w5, [x4, {call_target_flags}]\n",
            "tbz    w5, #{call_target_enabled_bit}, ",
            stringify!($label),
            "\n",
            "ldr    x6, [x4, {call_target_callee_bits}]\n",
            "cmp    x0, x6\n",
            "b.ne   ",
            stringify!($label),
            "\n",
            "ldr    x7, [x20, x",
            stringify!($this_reg),
            ", lsl #3]\n",
            "tbz    w5, #{call_target_this_global_bit}, 1f\n",
            "movz   x8, #0x1, lsl #32\n",
            "movk   x8, #0x7ff8, lsl #48\n",
            "cmp    x7, x8\n",
            "b.eq   2f\n",
            "movz   x8, #0x2, lsl #32\n",
            "movk   x8, #0x7ff8, lsl #48\n",
            "cmp    x7, x8\n",
            "b.eq   2f\n",
            "movz   x16, #0xffff, lsl #32\n",
            "movk   x16, #0x7ff8, lsl #48\n",
            "and    x16, x7, x16\n",
            "movz   x17, #0x5, lsl #32\n",
            "movk   x17, #0x7ff8, lsl #48\n",
            "cmp    x16, x17\n",
            "b.ne   ",
            stringify!($label),
            "\n",
            "b      1f\n",
            "2:\n",
            "ldr    x7, [x4, {call_target_global_this}]\n",
            "1:\n",
            "ldr    w8, [x24, {state_frame_depth}]\n",
            "cbz    w8, ",
            stringify!($label),
            "\n",
            "ldr    w9, [x24, {state_frame_info_len}]\n",
            "cmp    w8, w9\n",
            "b.hi   ",
            stringify!($label),
            "\n",
            "ldr    x2, [x24, {state_frame_info_base}]\n",
            "cbz    x2, ",
            stringify!($label),
            "\n",
            "sub    w3, w8, #1\n",
            "add    x3, x2, x3, lsl #{frame_info_stride_shift}\n",
            "ldr    w16, [x3, {frame_info_flags}]\n",
            "tbz    w16, #{frame_info_tail_call_recycle_safe_bit}, ",
            stringify!($label),
            "\n",
            "ldr    w11, [x3, {frame_info_register_base}]\n",
            "ldr    w10, [x4, {call_target_register_len}]\n",
            "adds   w12, w11, w10\n",
            "b.cs   ",
            stringify!($label),
            "\n",
            "ldr    w1, [x24, {state_register_stack_len}]\n",
            "cmp    w12, w1\n",
            "b.hi   ",
            stringify!($label),
            "\n",
            "ldr    x14, [x24, {state_register_stack_base}]\n",
            "cbz    x14, ",
            stringify!($label),
            "\n",
            "add    x15, x14, x11, lsl #3\n",
            "ldr    w9, [x3, {frame_info_return_register}]\n",
            "ldr    w13, [x3, {frame_info_callee_raw}]\n",
            "ubfx   w14, w16, #{frame_info_strict_bit}, #1\n",
            "movz   x6, #0x1, lsl #32\n",
            "movk   x6, #0x7ff8, lsl #48\n",
            "mov    w16, #0\n",
            "3:\n",
            "cmp    w16, w10\n",
            "b.hs   4f\n",
            "str    x6, [x15, x16, lsl #3]\n",
            "add    w16, w16, #1\n",
            "b      3b\n",
            "4:\n",
            "ldr    x6, [x4, {call_target_pb_base}]\n",
            "str    x6, [x3, {frame_info_pb_base}]\n",
            "str    x15, [x3, {frame_info_regs_base}]\n",
            "ldr    x16, [x4, {call_target_fv_base}]\n",
            "str    x16, [x3, {frame_info_fv_base}]\n",
            "ldr    x17, [x4, {call_target_const_base}]\n",
            "str    x17, [x3, {frame_info_const_base}]\n",
            "str    x7, [x3, {frame_info_this_value}]\n",
            "str    wzr, [x3, {frame_info_pc_offset}]\n",
            "str    w9, [x3, {frame_info_return_register}]\n",
            "ubfx   w16, w5, #{call_target_fast_return_safe_bit}, #1\n",
            "ubfx   w17, w5, #{call_target_strict_bit}, #1\n",
            "orr    w16, w16, w17, lsl #{frame_info_strict_bit}\n",
            "ubfx   w17, w5, #{call_target_tail_call_recycle_safe_bit}, #1\n",
            "orr    w16, w16, w17, lsl #{frame_info_tail_call_recycle_safe_bit}\n",
            "str    w16, [x3, {frame_info_flags}]\n",
            "str    w11, [x3, {frame_info_register_base}]\n",
            "str    w10, [x3, {frame_info_register_len}]\n",
            "ldr    w16, [x4, {call_target_code_raw}]\n",
            "str    w16, [x3, {frame_info_code_raw}]\n",
            "ldr    w16, [x4, {call_target_realm_raw}]\n",
            "str    w16, [x3, {frame_info_realm_raw}]\n",
            "ldr    w16, [x4, {call_target_lexical_env_raw}]\n",
            "str    w16, [x3, {frame_info_lexical_env_raw}]\n",
            "ldr    w16, [x4, {call_target_variable_env_raw}]\n",
            "str    w16, [x3, {frame_info_variable_env_raw}]\n",
            "ldr    w16, [x4, {call_target_private_env_raw}]\n",
            "str    w16, [x3, {frame_info_private_env_raw}]\n",
            "ldr    w16, [x4, {call_target_callee_raw}]\n",
            "str    w16, [x3, {frame_info_callee_raw}]\n",
            "ldr    w16, [x4, {call_target_parameter_initializer_end_offset}]\n",
            "str    w16, [x3, {frame_info_parameter_initializer_end_offset}]\n",
            "mov    w16, #3\n",
            "str    w16, [x3, {frame_info_frame_flags_raw}]\n",
            "str    w13, [x3, {frame_info_tail_caller_raw}]\n",
            "str    w14, [x3, {frame_info_tail_caller_strict}]\n",
            "str    w12, [x24, {state_register_stack_top}]\n",
            "str    wzr, [x24, {state_pc}]\n",
            "ldr    x19, [x4, {call_target_pb_base}]\n",
            "str    x19, [x24, {state_pb}]\n",
            "mov    x20, x15\n",
            "str    x20, [x24, {state_regs}]\n",
            "ldr    x21, [x4, {call_target_fv_base}]\n",
            "str    x21, [x24, {state_fv}]\n",
            "ldr    x16, [x4, {call_target_const_base}]\n",
            "str    x16, [x24, {vm_const_base}]\n",
            "str    x7, [x24, {state_this_value}]\n",
            "ldrb   w8, [x19]\n",
            "ldr    x16, [x23, x8, lsl #3]\n",
            "br     x16\n",
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
/// Two instructions: `cmp x{a}, x{b}; b.eq {label}`. The label is
/// substituted by the lowerer (DSL `.slow` → `<handler_name>slow`),
/// matching the existing `.slow:` body-label convention used by hot
/// handlers like `op_add` (see `crates/vm/src/dsl/handlers/hot.rs`).
///
/// Used by Phase 1.B.2's `op_load_this` inline port to bail to the
/// slow path when `frame_this_value` holds the
/// `Value::uninitialized_lexical()` sentinel (i.e., `ThisState` was
/// `Uninitialized` or `Lexical` at trampoline entry).
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
/// **TODO (Batch 7+):** `Ldouble_prefix` currently hits `brk #0` because
/// `op_double_prefix_slow_rs` doesn't exist yet. When the `op_wide` /
/// `op_extra_wide` ports land, replace the `brk #0` with a
/// `call_slow!(op_double_prefix_slow_rs, args = [])` + a proper exit
/// path.
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
            // TODO: call op_double_prefix_slow_rs once Batch 7 lands.
            "brk    #0\n",
        )
    };
}
