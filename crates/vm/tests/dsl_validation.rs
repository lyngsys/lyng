//! Consolidated DSL-0b/0c validation test binary.
//!
//! Each submodule below corresponds to one of the original
//! `dsl_validation_<case>.rs` integration test files. They are grouped
//! into a single binary to amortize cargo's per-binary link cost across
//! the 10 cases (the suite as a whole runs in milliseconds; the wall-time
//! cost was almost entirely cargo overhead).

#[path = "dsl_validation/empty.rs"]
mod empty;
#[path = "dsl_validation/frame_context.rs"]
mod frame_context;
#[path = "dsl_validation/pc_sync.rs"]
mod pc_sync;
#[path = "dsl_validation/prefix_double.rs"]
mod prefix_double;
#[path = "dsl_validation/prefix_extra_wide.rs"]
mod prefix_extra_wide;
#[path = "dsl_validation/prefix_wide.rs"]
mod prefix_wide;
#[path = "dsl_validation/safepoint_backward_cond_jump.rs"]
mod safepoint_backward_cond_jump;
#[path = "dsl_validation/safepoint_backward_jump.rs"]
mod safepoint_backward_jump;
#[path = "dsl_validation/safepoint_loop_header.rs"]
mod safepoint_loop_header;
#[path = "dsl_validation/slow_roundtrip.rs"]
mod slow_roundtrip;
