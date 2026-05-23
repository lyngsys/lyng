use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use lyng_js_test262::{prepare_diagnostic_suite, Test262DiagnosticConfig};

static TEMP_DIR_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn make_temp_dir() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    let counter = TEMP_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "lyng-js-test262-diagnostics-{}-{}-{}",
        std::process::id(),
        nonce,
        counter
    ));
    fs::create_dir_all(&path).expect("temp dir should be created");
    path
}

#[test]
fn diagnostic_api_prepares_and_executes_variants() {
    let root = make_temp_dir();
    let test_path = root.join("agent-diagnostic.js");
    fs::write(
        &test_path,
        r"
        function add(left, right) {
            return left + right;
        }
        assert.sameValue(add(1, 2), 3);
        ",
    )
    .expect("fixture should be written");

    let suite = prepare_diagnostic_suite(&Test262DiagnosticConfig {
        filter: Some(test_path.display().to_string()),
        no_skip: true,
        ..Test262DiagnosticConfig::default()
    })
    .expect("suite should prepare");

    assert_eq!(suite.candidate_total(), 1);
    assert_eq!(suite.tests().len(), 2);
    assert_eq!(suite.tests()[0].file, test_path.display().to_string());
    assert_eq!(suite.tests()[0].category, test_path.display().to_string());
    assert_eq!(suite.tests()[0].timeout_ms, 1_000);

    let outcome = suite
        .run_diagnostic(suite.tests()[0].index)
        .expect("diagnostic execution should run");

    assert_eq!(outcome.outcome, "pass");
    assert_eq!(outcome.identity.file, test_path.display().to_string());
    assert!(outcome.timings.total > outcome.timings.evaluation);
    assert!(outcome.timings.parse > std::time::Duration::ZERO);
    assert!(outcome.timings.lowering > std::time::Duration::ZERO);
    assert!(outcome.timings.evaluation > std::time::Duration::ZERO);
    let diagnostics = outcome
        .diagnostics
        .expect("script execution should expose VM diagnostics");
    assert!(diagnostics.function_count >= 2);
    assert!(diagnostics.instruction_words > 0);
    assert!(diagnostics.feedback_slots > 0);

    let _ = fs::remove_dir_all(root);
}
