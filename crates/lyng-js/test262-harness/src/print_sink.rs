use std::io::{self, Write};
use std::sync::{Arc, Mutex};

/// Pluggable destination for the Test262 harness `print` function.
///
/// Implementors are shared across worker threads via `Arc<dyn Test262PrintSink>`,
/// so they must be `Send + Sync`.
pub trait Test262PrintSink: Send + Sync {
    /// Records one `print` invocation. Each call corresponds to a single
    /// `print(...)` from guest code.
    fn record(&self, message: String);
}

/// Buffered sink that retains every recorded message in memory.
///
/// The batch Test262 runner uses this to detect async completion messages
/// after a test finishes evaluating.
#[derive(Clone, Default)]
pub struct Test262PrintObserver {
    messages: Arc<Mutex<Vec<String>>>,
}

impl Test262PrintObserver {
    /// Returns a snapshot of every message recorded so far.
    #[must_use]
    pub fn messages(&self) -> Vec<String> {
        match self.messages.lock() {
            Ok(messages) => messages.clone(),
            Err(poisoned) => poisoned.into_inner().clone(),
        }
    }
}

impl Test262PrintSink for Test262PrintObserver {
    fn record(&self, message: String) {
        match self.messages.lock() {
            Ok(mut messages) => messages.push(message),
            Err(poisoned) => poisoned.into_inner().push(message),
        }
    }
}

/// Sink that writes each recorded message to process stdout, terminated by a
/// newline.
///
/// The CLI uses this under `--test262` so harness `print` calls match the
/// convention shared by every other Test262-aware engine.
#[derive(Clone, Copy, Default)]
pub struct Test262StdoutPrintSink;

impl Test262PrintSink for Test262StdoutPrintSink {
    fn record(&self, message: String) {
        let _ = writeln!(io::stdout(), "{message}");
    }
}
