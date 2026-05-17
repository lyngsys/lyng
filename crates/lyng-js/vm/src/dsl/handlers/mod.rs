//! DSL handler functions per design §10 (DSL-0b).
//!
//! Each opcode's asm-DSL handler is a `#[unsafe(naked)] extern "C" fn`
//! that the trampoline tail-jumps to via the dispatch table. The
//! actual handler bodies are emitted by the `lyng_js_vm_dsl::dsl_op!`
//! proc-macro and land in Task B29.
//!
//! For DSL-0b Batch 2 this file only carries the table skeleton —
//! every slot points to `unimplemented_dsl_handler`, which is never
//! reached because the trampoline itself is still a stub (see
//! `dsl::entry::run_dsl_trampoline`). Batch 4 fills the table in.

/// Calling convention for a DSL handler. The asm trampoline tail-calls
/// these via the dispatch table indexed by opcode byte; handlers never
/// return — they either tail-jump to the next handler or branch to
/// `_interpreter_exit`.
pub type DslHandler = unsafe extern "C" fn() -> !;

/// Placeholder dispatch table; populated in Task B29.
#[allow(dead_code)]
pub static DSL_DISPATCH_TABLE: [DslHandler; 256] = [unimplemented_dsl_handler; 256];

/// Diverging placeholder used until Task B29 emits real handlers. Hit
/// only if the trampoline is somehow re-entered before DSL-0c flips
/// dispatch — currently unreachable since the trampoline is itself a
/// stub.
#[allow(dead_code)]
unsafe extern "C" fn unimplemented_dsl_handler() -> ! {
    loop {} // SAFETY: never reachable until DSL-0c flips dispatch.
}
