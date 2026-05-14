//! Arithmetic family handlers for the trampoline dispatch path.
//!
//! Placeholder until sub-4 (lyng-54em) lands real SMI fast paths + cold
//! delegations to the existing `execute_*_opcode` helpers. The spike's
//! prototype `op_add` (with a fictional encoding) was removed in sub-3
//! because mixing it into the production dispatch table would produce
//! incorrect results on any program that hits the real `Opcode::Add`
//! encoding; the slot now falls back through `op_unimplemented<Add>` until
//! sub-4 replaces it.
