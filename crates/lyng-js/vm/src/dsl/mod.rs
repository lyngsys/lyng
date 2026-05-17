//! asm-DSL substrate per docs/lyng-js/2026-05-16-asm-dsl-llint-interpreter-design.md.
//!
//! This module hosts the DSL runtime support — opcode manifest, slow-path
//! bridge types, register-pin convention, `LlIntState` ABI, entry/exit
//! shims, and the per-arch DSL operation backend. The proc-macro that
//! consumes these lives in the separate `lyng-js-vm-dsl` crate.
//!
//! During DSL-0a the only module populated is `opcode_manifest` plus the
//! transitional `LlIntDispatchState` wrapper in `slow_path`. DSL-0b adds
//! every other module.

pub mod entry;
pub mod feedback_flat;
pub mod handlers;
pub mod llint_state;
pub mod opcode_manifest;
pub mod reg_convention;
pub mod slow_path;
