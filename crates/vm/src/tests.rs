#![allow(
    clippy::wildcard_imports,
    reason = "VM test submodules share a domain-local support prelude"
)]

mod support;

mod async_and_generators;
mod classes;
mod core;
mod debugger;
mod disposables;
mod dynamic_import;
mod errors;
mod eval_and_with;
mod feedback;
mod generators;
mod inline_caches;
mod metadata_and_tail_calls;
mod modules;
mod promises;
mod text;

mod trampoline_dispatch;
