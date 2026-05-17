//! Hot DSL handlers. Populated by tasks B39–B42.
//!
//! Per the design (§10), hot handlers are the highest-frequency opcodes
//! and ship with inline fast paths (SMI arithmetic, register moves, fast
//! object access). The `llint_handler!` proc-macro lowers each handler
//! body into a single `naked_asm!` block; the backend `macro_rules!`
//! macros (under `crates/lyng-js/vm/src/dsl/backend/aarch64/`) supply the
//! asm fragments for individual DSL ops (`decode_ab!`, `load_reg!`,
//! `dispatch!`, etc.).
//!
//! For DSL-0b the handler symbols exist (so the link-check passes) but
//! they are not yet wired into `DSL_DISPATCH_TABLE` — the alpha path
//! continues to dispatch through the legacy handlers. Phase C of the
//! plan flips the table over.

// Bring the AArch64 backend macros into scope so the proc-macro-emitted
// `decode_ab!`, `load_reg!`, `store_reg!`, `dispatch!`, ... calls
// resolve. They are `#[macro_export]`-ed at the crate root.
#[cfg(target_arch = "aarch64")]
use crate::{decode_ab, dispatch, load_reg, store_reg};

#[cfg(target_arch = "aarch64")]
use lyng_js_vm_dsl::llint_handler;

#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_move, layout = Ab, length = 3, |dst, src| {
        load_reg!(src => t0);
        store_reg!(dst, t0);
        dispatch!();
    }
}

/// Non-aarch64 stub. The DSL handler family is aarch64-only in DSL-0b
/// per design §3; on other hosts we emit a placeholder so the dispatch
/// table can still be assembled.
#[cfg(not(target_arch = "aarch64"))]
pub unsafe extern "C" fn op_move() -> ! {
    loop {}
}
