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
#[macro_export]
macro_rules! poll_safepoint {
    ($label_pending:tt) => {
        concat!(
            "ldrb   w9, [x22, {vm_poll}]\n",
            "cbnz   w9, ", stringify!($label_pending), "\n",
        )
    };
}
