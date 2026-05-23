//! Safepoint / interrupt-poll macros.
//!
//! Cooperative safepoints check a VM-local flag every back-branch (and
//! before allocation-heavy ops). When the host sets the flag, the
//! macro branches to a slow path that lets host hooks run.
//!
//! Bindings expected from the proc-macro lowerer:
//!
//! - `{vm_poll}` — `const VM_POLL_PENDING_OFFSET`. Currently a
//!   placeholder (offset 0) until Task B41 lands the real `Vm` field.
//!   The asm bridge never reads through this in DSL-0b — it's
//!   declared so the macro doesn't need a separate "later" form.

/// Poll the VM safepoint flag. If set, branch to `$label_pending`
/// (typically a `call_slow!(op_poll_slow_rs, ...)` site).
///
/// Compiles to 2 instructions on the fast (no-pending) path.
///
/// ## Opcode-byte injection (DSL-1 Phase 1.B.0 Task 5)
///
/// The lowerer rewrites every `poll_safepoint!(.label)` invocation to
/// append `, opcode_byte = N` — the handler's own opcode discriminant.
/// On the pending-poll branch (the slow path), the macro emits
/// `inc_slow_safepoint_counter!(N)`. The fast (no-pending) path keeps
/// its original 2-instruction shape regardless of feature flag state.
///
/// Implementation note: the counter increment must not happen on the
/// fast path. The two feature-gated arms reshape the asm:
///
/// - **Feature OFF**: identical to the legacy form (2 insns on fast
///   path; direct `cbnz` to the pending label). Zero overhead.
/// - **Feature ON**: 3 insns on fast path (`ldrb` + `cbz 9f` + jump
///   past a 5-insn slow block). The slow block bumps the
///   `slow_safepoint` bank slot for this opcode and falls through to
///   the pending-poll label via an unconditional branch.
///
/// Bindings: `{vm_poll}`, `{vm_counter_base}` (only referenced when
/// `opcode-counters` feature is on).
#[cfg(feature = "opcode-counters")]
#[macro_export]
macro_rules! poll_safepoint {
    ($label_pending:tt, opcode_byte = $op:literal) => {
        concat!(
            "ldrb   w16, [x22, {vm_poll}]\n",
            // Fast path: clear flag → skip the counter-bump block.
            // `9f` is a numeric local label; Apple's assembler scopes
            // numeric locals per asm block, so it can't collide with
            // `1:`/`2:` used elsewhere (e.g. `dispatch_after_slow!`).
            "cbz    w16, 9f\n",
            // Slow path: bump slow_safepoint bank slot for this opcode,
            // then branch to the pending-poll target.
            $crate::inc_slow_safepoint_counter!($op),
            "b      ",
            stringify!($label_pending),
            "\n",
            "9:\n",
        )
    };
    // Legacy form (no opcode_byte) — same expansion as feature-off path
    // for hand-written callers that bypass the lowerer.
    ($label_pending:tt) => {
        concat!(
            "ldrb   w16, [x22, {vm_poll}]\n",
            "cbnz   w16, ",
            stringify!($label_pending),
            "\n",
        )
    };
}

#[cfg(not(feature = "opcode-counters"))]
#[macro_export]
macro_rules! poll_safepoint {
    // Feature OFF: opcode_byte is silently consumed, no counter emission.
    // Identical asm shape to the legacy 2-insn form.
    ($label_pending:tt, opcode_byte = $op:literal) => {
        concat!(
            "ldrb   w16, [x22, {vm_poll}]\n",
            "cbnz   w16, ",
            stringify!($label_pending),
            "\n",
        )
    };
    ($label_pending:tt) => {
        concat!(
            "ldrb   w16, [x22, {vm_poll}]\n",
            "cbnz   w16, ",
            stringify!($label_pending),
            "\n",
        )
    };
}
