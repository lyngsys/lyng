#![allow(
    clippy::wildcard_imports,
    reason = "script-core test submodules share a domain-local support prelude"
)]

mod arrays_and_buffers;
mod builtins_reflect_proxy;
mod control_flow;
mod destructuring_and_iteration;
mod globals_dates_templates;
mod misc_regressions;
mod numeric_and_collections;
mod regexp_and_annex_b;
mod string_and_regexp;
