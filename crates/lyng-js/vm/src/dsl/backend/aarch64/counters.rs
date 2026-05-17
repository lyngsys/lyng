//! Opcode-counter increment, gated by `--features opcode-counters`.
//!
//! When the feature is off, the macro expands to an empty string —
//! zero per-dispatch cost. When on, emits 4 instructions to bump the
//! per-opcode counter slot in the VM's `opcode_counters` array:
//!
//! ```text
//!     ldr  x9, [x22, {vm_counter_base}]
//!     ldr  x10, [x9, #<op*8>]
//!     add  x10, x10, #1
//!     str  x10, [x9, #<op*8>]
//! ```
//!
//! Bindings expected (only when feature is on):
//!
//! - `{vm_counter_base}` — `const VM_OPCODE_COUNTER_OFFSET` (placeholder
//!   until the `Vm` struct gains the field).
//!
//! `$opcode_byte` is the opcode discriminator (`u8`) baked at lower
//! time. We materialize `opcode_byte * 8` as a literal so the offset
//! is encodable as an immediate for narrow indices and the compiler
//! synthesizes the appropriate addressing mode for wider ones.

#[cfg(feature = "opcode-counters")]
#[macro_export]
macro_rules! inc_counter {
    ($opcode_byte:literal) => {
        concat!(
            "ldr    x9, [x22, {vm_counter_base}]\n",
            "ldr    x10, [x9, #", stringify!($opcode_byte), " * 8]\n",
            "add    x10, x10, #1\n",
            "str    x10, [x9, #", stringify!($opcode_byte), " * 8]\n",
        )
    };
}

#[cfg(not(feature = "opcode-counters"))]
#[macro_export]
macro_rules! inc_counter {
    ($opcode_byte:literal) => { "" };
}
