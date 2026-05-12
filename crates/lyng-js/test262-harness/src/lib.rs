//! Shared Test262 host harness realm extension.
//!
//! Owns the `$262` host object, the `print` global, the `setTimeout` global, and the
//! agent.* helpers that Test262 requires. The harness is exposed as a
//! [`RealmExtensionProvider`] so that any embedding — the batch
//! `lyng-js-test262` tool, the `lyng-js` CLI under `--test262`, or other future
//! embeddings — can install the same realm-scoped surface without duplicating
//! the embedding-function bookkeeping.

#![allow(
    clippy::module_name_repetitions,
    reason = "Test262 type prefixes intentionally namespace the harness surface"
)]

mod print_sink;
mod realm_extension;

pub use print_sink::{Test262PrintObserver, Test262PrintSink, Test262StdoutPrintSink};
pub use realm_extension::Test262RealmExtension;
