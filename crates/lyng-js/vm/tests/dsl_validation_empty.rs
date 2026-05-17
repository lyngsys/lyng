//! DSL-0b validation case 1 (design §10): an empty naked handler
//! compiles and is callable.
//!
//! This is the load-bearing test of the asm-DSL design — see B30 in the
//! plan. It exercises the full proc-macro + backend-macro +
//! `naked_asm!` integration:
//!
//! 1. `llint_handler!` parses the syntax in `lyng-js-vm-dsl::parse`.
//! 2. The lowerer (`lyng-js-vm-dsl::lower`) emits an
//!    `#[unsafe(naked)] pub extern "C" fn` whose body is a single
//!    `core::arch::naked_asm!` call. Each body statement becomes a
//!    comma-separated template argument; the trailing
//!    `length = const N as u32` is the only named binding needed for
//!    this minimal case.
//! 3. `dispatch!(advance = 0)` is a `#[macro_export]`-ed backend macro
//!    living at `lyng_js_vm::dispatch`. It expands to a `concat!(...)`
//!    yielding a four-instruction tail-jump asm fragment.
//! 4. rustc composes everything into a single asm template and produces
//!    a real `extern "C" fn` whose symbol can be taken at runtime.

#[cfg(target_arch = "aarch64")]
use lyng_js_vm::dispatch;
use lyng_js_vm_dsl::llint_handler;

#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_validation_empty, layout = None, length = 1, || {
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
// is aarch64-only in DSL-0b per design §3.
#[cfg(not(target_arch = "aarch64"))]
#[test]
fn empty_handler_symbol_exists() {
    // No-op on non-aarch64; the asm-DSL backend isn't compiled here.
}
