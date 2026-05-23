//! Opcode-counter increments, gated by `--features opcode-counters`.
//!
//! When the feature is off, the macros expand to empty strings — zero
//! per-dispatch cost. When on, each emits 4 instructions to bump the
//! per-opcode counter slot in the relevant `DispatchCounters` bank.
//!
//! ## How the asm path reaches the counter array
//!
//! `OpcodeDispatchCounterStore { counters: Box<DispatchCounters> }` has
//! a single `Box<T>` field. `Box<T>` is layout-equivalent to a raw
//! pointer (8 bytes on 64-bit), so:
//!
//! - `offset_of!(Vm, dispatch_counters)` = offset to the Box (which IS
//!   the `*mut DispatchCounters` pointer).
//! - `ldr xR, [x22, #VM_DISPATCH_COUNTERS_PTR_OFFSET]` directly reads
//!   the pointer to `DispatchCounters`.
//!
//! From there, bank offsets (0, 2048, 4096) index into the three
//! `[u64; 256]` banks. All bank offsets are 8-byte-aligned and within
//! the AArch64 LDR/STR `#imm` range.
//!
//! Emitted shape (4 instructions per increment):
//!
//! ```text
//!     ldr  xS, [x22, {vm_counter_base}]              ; xS = *DispatchCounters
//!     ldr  xT, [xS, #<bank_offset + op*8>]           ; xT = current count
//!     add  xT, xT, #1
//!     str  xT, [xS, #<bank_offset + op*8>]           ; store back
//! ```
//!
//! ## Scratch-register convention per bank (DSL-1 Phase 1.B.0 Task 5)
//!
//! - **Dispatch bank** (`inc_dispatch_counter!`) uses `x9, x10`. Emitted
//!   as the FIRST body fragment, BEFORE the operand-decode prologue —
//!   no live operand values to clobber.
//! - **Slow_semantic / Slow_safepoint banks** use `x16, x17` (AAPCS64
//!   IP0/IP1 scratch). Emitted INSIDE `call_slow!` / `poll_safepoint!`
//!   AFTER the operand-decode prologue, so any live operands in x9..x15
//!   are preserved. The `call_slow!` bridge subsequently reloads x16/x17
//!   for its own pc-offset stash — the counter values are stored before
//!   that point so the clobber is harmless.
//!
//! ## Bindings expected from the proc-macro lowerer (when feature is on)
//!
//! - `{vm_counter_base}` — `const VM_DISPATCH_COUNTERS_PTR_OFFSET`.
//!
//! ## Bank-offset encoding
//!
//! - Dispatch bank: literal `<op>*8`. For op < 256, max offset = 2040 (≤ 32760 scaled-immediate range).
//! - Slow_semantic bank: literal `<op>*8 + 2048`. Max = 4088.
//! - Slow_safepoint bank: literal `<op>*8 + 4096`. Max = 6136.
//!
//! All within AArch64's `LDR Xt, [Xn, #imm]` scaled-u64 immediate range (#0..#32760).

// =============================================================================
// Opcode-counters feature ON: emit real counter increments.
// =============================================================================

#[cfg(feature = "opcode-counters")]
#[macro_export]
macro_rules! inc_dispatch_counter {
    ($opcode_byte:literal) => {
        concat!(
            "ldr    x9, [x22, {vm_counter_base}]\n",
            "ldr    x10, [x9, #",
            stringify!($opcode_byte),
            " * 8]\n",
            "add    x10, x10, #1\n",
            "str    x10, [x9, #",
            stringify!($opcode_byte),
            " * 8]\n",
        )
    };
}

#[cfg(feature = "opcode-counters")]
#[macro_export]
macro_rules! inc_slow_semantic_counter {
    ($opcode_byte:literal) => {
        concat!(
            // Use x16/x17 (AAPCS64 IP0/IP1) — they're free to clobber
            // before the `call_slow!` bridge runs, which reloads them
            // for its own pc-offset stash. Crucially, x9..x15 hold
            // decoded operand values that the bridge moves into w1..w5
            // immediately after this fragment; using x9/x10 here would
            // corrupt them.
            "ldr    x16, [x22, {vm_counter_base}]\n",
            "ldr    x17, [x16, #",
            stringify!($opcode_byte),
            " * 8 + 2048]\n",
            "add    x17, x17, #1\n",
            "str    x17, [x16, #",
            stringify!($opcode_byte),
            " * 8 + 2048]\n",
        )
    };
}

#[cfg(feature = "opcode-counters")]
#[macro_export]
macro_rules! inc_slow_safepoint_counter {
    ($opcode_byte:literal) => {
        concat!(
            // Same x16/x17 convention as `inc_slow_semantic_counter!`:
            // they're free to clobber before the pending-poll path
            // jumps into a `call_slow!` site (which reloads them).
            "ldr    x16, [x22, {vm_counter_base}]\n",
            "ldr    x17, [x16, #",
            stringify!($opcode_byte),
            " * 8 + 4096]\n",
            "add    x17, x17, #1\n",
            "str    x17, [x16, #",
            stringify!($opcode_byte),
            " * 8 + 4096]\n",
        )
    };
}

// =============================================================================
// Opcode-counters feature OFF: empty strings (zero per-dispatch cost).
// =============================================================================

#[cfg(not(feature = "opcode-counters"))]
#[macro_export]
macro_rules! inc_dispatch_counter {
    ($opcode_byte:literal) => {
        ""
    };
}

#[cfg(not(feature = "opcode-counters"))]
#[macro_export]
macro_rules! inc_slow_semantic_counter {
    ($opcode_byte:literal) => {
        ""
    };
}

#[cfg(not(feature = "opcode-counters"))]
#[macro_export]
macro_rules! inc_slow_safepoint_counter {
    ($opcode_byte:literal) => {
        ""
    };
}
