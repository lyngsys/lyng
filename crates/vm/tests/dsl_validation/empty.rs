//! DSL validation case: an empty naked handler compiles and is callable.
//!
//! Exercises the full proc-macro + backend-macro + `naked_asm!` pipeline:
//! `llint_handler!` → lowerer → `naked_asm!` → addressable `extern "C" fn`.

#[cfg(target_arch = "aarch64")]
use lyng_vm::dispatch;
use lyng_vm_dsl::llint_handler;

#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_validation_empty, opcode_byte = 200, layout = None, length = 1, || {
        dispatch!(advance = 0);
    }
}

#[cfg(target_arch = "aarch64")]
#[test]
fn empty_handler_symbol_exists() {
    // Take the function's address to force the linker to keep it.
    let ptr = op_validation_empty as *const ();
    assert!(!ptr.is_null());
}

// On non-aarch64 hosts the backend macros aren't compiled in, so we
// skip the validation case entirely. The trampoline / handler family
// is aarch64-only per design.
#[cfg(not(target_arch = "aarch64"))]
#[test]
fn empty_handler_symbol_exists() {
    // No-op on non-aarch64; the asm-DSL backend isn't compiled here.
}
