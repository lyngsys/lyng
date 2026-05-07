use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static TEMP_DIR_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn make_temp_dir() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    let counter = TEMP_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "lyng-js-test262-harness-{}-{}-{}",
        std::process::id(),
        nonce,
        counter
    ));
    fs::create_dir_all(&path).expect("temp dir should be created");
    path
}

fn run_passing_test(path: &Path, source: &str) -> String {
    let root = path.parent().expect("fixture path should have a parent");
    let report_path = root.join("report.md");
    fs::write(path, source).expect("fixture should be written");

    let output = Command::new(env!("CARGO_BIN_EXE_lyng-js-test262"))
        .args([
            "--filter",
            path.to_str().expect("path should be utf-8"),
            "--report",
            report_path.to_str().expect("path should be utf-8"),
            "--timeout-ms",
            "1000",
            "-j1",
        ])
        .output()
        .expect("runner should execute");

    assert!(
        output.status.success(),
        "runner exited with {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let report = fs::read_to_string(&report_path).expect("report should be written");
    assert_passed(&report, 1, 2);
    report
}

fn assert_passed(report: &str, files: u32, variants: u32) {
    assert!(
        report.contains(&format!("| Passed files | `{files}` |")),
        "unexpected report:\n{report}"
    );
    assert!(
        report.contains(&format!("| Passed variants | `{variants}` |")),
        "unexpected report:\n{report}"
    );
}

fn assert_failed(report: &str, files: u32, variants: u32) {
    assert!(
        report.contains(&format!("| Failed files | `{files}` |")),
        "unexpected report:\n{report}"
    );
    assert!(
        report.contains(&format!("| Failed variants | `{variants}` |")),
        "unexpected report:\n{report}"
    );
}

fn assert_skipped(report: &str, files: u32, variants: u32) {
    assert!(
        report.contains(&format!("| Skipped files | `{files}` |")),
        "unexpected report:\n{report}"
    );
    assert!(
        report.contains(&format!("| Skipped variants | `{variants}` |")),
        "unexpected report:\n{report}"
    );
}

fn run_filtered_test(filter: &str) -> String {
    run_filtered_test_with_timeout(filter, 1000)
}

fn run_filtered_test_with_timeout(filter: &str, timeout_ms: u32) -> String {
    let root = make_temp_dir();
    let report_path = root.join("report.md");

    let output = Command::new(env!("CARGO_BIN_EXE_lyng-js-test262"))
        .args([
            "--filter",
            filter,
            "--report",
            report_path.to_str().expect("path should be utf-8"),
            "--timeout-ms",
            &timeout_ms.to_string(),
            "-j1",
        ])
        .output()
        .expect("runner should execute");

    assert!(
        output.status.success(),
        "runner exited with {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let report = fs::read_to_string(&report_path).expect("report should be written");
    let _ = fs::remove_dir_all(root);
    report
}

fn run_single_test(path: &Path, source: &str) -> String {
    let root = path.parent().expect("fixture path should have a parent");
    let report_path = root.join("report.md");
    fs::write(path, source).expect("fixture should be written");

    let output = Command::new(env!("CARGO_BIN_EXE_lyng-js-test262"))
        .args([
            "--filter",
            path.to_str().expect("path should be utf-8"),
            "--report",
            report_path.to_str().expect("path should be utf-8"),
            "--timeout-ms",
            "1000",
            "-j1",
        ])
        .output()
        .expect("runner should execute");

    assert!(
        output.status.success(),
        "runner exited with {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    fs::read_to_string(&report_path).expect("report should be written")
}

fn run_single_test_with_output(path: &Path, source: &str) -> (String, String) {
    let root = path.parent().expect("fixture path should have a parent");
    let report_path = root.join("report.md");
    fs::write(path, source).expect("fixture should be written");

    let output = Command::new(env!("CARGO_BIN_EXE_lyng-js-test262"))
        .args([
            "--filter",
            path.to_str().expect("path should be utf-8"),
            "--report",
            report_path.to_str().expect("path should be utf-8"),
            "--timeout-ms",
            "1000",
            "-j1",
        ])
        .output()
        .expect("runner should execute");

    assert!(
        output.status.success(),
        "runner exited with {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    (
        String::from_utf8_lossy(&output.stdout).into_owned(),
        fs::read_to_string(&report_path).expect("report should be written"),
    )
}

#[test]
fn runner_exposes_eval_script_and_create_realm_through_external_embedding() {
    let root = make_temp_dir();
    let entry_path = root.join("realm.js");

    let report = run_passing_test(
        &entry_path,
        r#"
        let value = $262.evalScript("globalThis.fromEmbedding = 41; fromEmbedding;");
        let other = $262.createRealm();
        assert.sameValue(value, 41);
        assert.sameValue(fromEmbedding, 41);
        assert.sameValue(typeof other.evalScript, "function");
        assert.sameValue(other.global === globalThis, false);
        assert.sameValue(other.evalScript("typeof $262"), "object");
        assert.sameValue(typeof $262.AbstractModuleSource, "function");
        assert.sameValue(typeof $262.agent, "object");
        assert.sameValue(typeof $262.agent.getReport, "function");
        assert.sameValue(typeof $262.agent.sleep, "function");
        assert.sameValue(typeof $262.agent.monotonicNow, "function");
        assert.sameValue("IsHTMLDDA" in $262, true);
        "#,
    );

    assert_skipped(&report, 0, 0);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn runner_installs_is_htmldda_test262_host_object() {
    let root = make_temp_dir();
    let entry_path = root.join("is-html-dda.js");

    let report = run_passing_test(
        &entry_path,
        r#"
        /*---
        features: [IsHTMLDDA, coalesce-expression]
        ---*/
        const value = $262.IsHTMLDDA;
        assert.sameValue(typeof value, "undefined");
        assert.sameValue(Boolean(value), false);
        assert.sameValue(value == undefined, true);
        assert.sameValue(value == null, true);
        assert.sameValue(value === undefined, false);
        assert.sameValue(value === null, false);
        assert.sameValue(Object.is(value, undefined), false);
        assert.sameValue(value ?? "fallback", value);
        assert.sameValue(value(), null);
        "#,
    );

    assert_skipped(&report, 0, 0);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn runner_supports_single_agent_atomics_helper_surface() {
    let root = make_temp_dir();
    let entry_path = root.join("atomics-agent-helper.js");

    let _report = run_passing_test(
        &entry_path,
        r#"
        /*---
        includes: [atomicsHelper.js]
        features: [Atomics, SharedArrayBuffer, TypedArray]
        ---*/

        let before = $262.agent.monotonicNow();
        assert.sameValue($262.agent.sleep(1), undefined);
        assert($262.agent.monotonicNow() >= before);
        assert.sameValue(typeof setTimeout, "function");
        assert.sameValue(typeof $262.agent.setTimeout, "function");
        "#,
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn runner_supports_single_agent_atomics_helper_set_timeout_callback() {
    let root = make_temp_dir();
    let entry_path = root.join("atomics-agent-helper-set-timeout.js");

    let _report = run_passing_test(
        &entry_path,
        r"
        /*---
        flags: [async]
        includes: [atomicsHelper.js]
        features: [Atomics, SharedArrayBuffer, TypedArray, arrow-function]
        ---*/

        $262.agent.setTimeout(() => $DONE(), 0);
        ",
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn runner_supports_single_agent_start_broadcast_and_reports() {
    let root = make_temp_dir();
    let entry_path = root.join("atomics-agent-helper-start-broadcast.js");

    let _report = run_passing_test(
        &entry_path,
        r#"
        /*---
        flags: [async]
        includes: [atomicsHelper.js]
        features: [Atomics, SharedArrayBuffer, TypedArray, arrow-function]
        ---*/

        const RUNNING = 1;
        $262.agent.start(`
          $262.agent.receiveBroadcast((sab) => {
            const i32a = new Int32Array(sab);
            Atomics.add(i32a, ${RUNNING}, 1);
            $262.agent.report("started:" + Atomics.load(i32a, ${RUNNING}));
            $262.agent.leaving();
          });
        `);

        const i32a = new Int32Array(
          new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT * 2)
        );
        $262.agent.safeBroadcastAsync(i32a, RUNNING, 1).then(async count => {
          assert.sameValue(count, 1);
          assert.sameValue(await $262.agent.getReportAsync(), "started:1");
        }).then($DONE, $DONE);
        "#,
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn runner_supports_single_process_agent_notify_one_waiter() {
    let report = run_filtered_test("built-ins/Atomics/notify/notify-one.js");
    assert_passed(&report, 1, 2);
}

#[test]
fn runner_honors_can_block_false_atomics_wait_metadata() {
    let report = run_filtered_test("built-ins/Atomics/wait/cannot-suspend-throws.js");
    assert_passed(&report, 1, 2);
}

#[test]
fn runner_reports_async_failure_from_single_agent_set_timeout_callback() {
    let root = make_temp_dir();
    let entry_path = root.join("atomics-agent-helper-set-timeout-failure.js");
    let report_path = root.join("report.md");
    fs::write(
        &entry_path,
        r#"
        /*---
        flags: [async]
        includes: [atomicsHelper.js]
        features: [Atomics, SharedArrayBuffer, TypedArray, arrow-function]
        ---*/

        $262.agent.setTimeout(() => $DONE("timer failure"), 0);
        "#,
    )
    .expect("fixture should be written");

    let output = Command::new(env!("CARGO_BIN_EXE_lyng-js-test262"))
        .args([
            "--filter",
            entry_path.to_str().expect("path should be utf-8"),
            "--report",
            report_path.to_str().expect("path should be utf-8"),
            "--timeout-ms",
            "1000",
            "-j1",
        ])
        .output()
        .expect("runner should execute");

    assert!(
        output.status.success(),
        "runner exited with {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let report = fs::read_to_string(&report_path).expect("report should be written");
    assert_failed(&report, 1, 2);
    assert!(
        report.contains("timer failure"),
        "unexpected report:\n{report}"
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn runner_supports_single_agent_wait_async_timeout_promise_all_without_polling() {
    let root = make_temp_dir();
    let entry_path = root.join("atomics-wait-async-timeout-promise-all.js");

    let _report = run_passing_test(
        &entry_path,
        r"
        /*---
        flags: [async]
        includes: [atomicsHelper.js]
        features: [Atomics.waitAsync, SharedArrayBuffer, TypedArray, Atomics, computed-property-names, Symbol, Symbol.toPrimitive, arrow-function]
        ---*/

        const i32a = new Int32Array(
          new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT * 4)
        );
        const valueOf = {
          valueOf() {
            return true;
          }
        };
        const toPrimitive = {
          [Symbol.toPrimitive]() {
            return true;
          }
        };
        let outcomes = [];

        Promise.all([
          Atomics.waitAsync(i32a, 0, 0, true).value,
          Atomics.waitAsync(i32a, 0, 0, valueOf).value,
          Atomics.waitAsync(i32a, 0, 0, toPrimitive).value,
        ]).then(results => {
          outcomes = results;
          assert.sameValue(outcomes[0], 'timed-out');
          assert.sameValue(outcomes[1], 'timed-out');
          assert.sameValue(outcomes[2], 'timed-out');
          $DONE();
        }, $DONE);
        ",
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn runner_supports_single_agent_set_timeout_polling_promise_update() {
    let root = make_temp_dir();
    let entry_path = root.join("atomics-agent-helper-polling-promise.js");

    let _report = run_passing_test(
        &entry_path,
        r#"
        /*---
        flags: [async]
        includes: [atomicsHelper.js]
        features: [Atomics, SharedArrayBuffer, TypedArray, arrow-function]
        ---*/

        let outcomes = [];
        let attempts = 0;

        (function wait() {
          if (outcomes.length) {
            assert.sameValue(outcomes[0], 1);
            $DONE();
            return;
          }
          if (++attempts > 4) {
            $DONE("promise update was not observed before repeated timer callbacks");
            return;
          }
          $262.agent.setTimeout(wait, 0);
        })();

        Promise.resolve([1]).then(results => {
          outcomes = results;
        }, $DONE);
        "#,
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn runner_supports_single_agent_wait_async_timeout_with_atomics_helper() {
    let root = make_temp_dir();
    let entry_path = root.join("atomics-wait-async-timeout.js");

    let _report = run_passing_test(
        &entry_path,
        r#"
        /*---
        flags: [async]
        includes: [atomicsHelper.js]
        features: [Atomics.waitAsync, SharedArrayBuffer, TypedArray, Atomics, computed-property-names, Symbol, Symbol.toPrimitive, arrow-function]
        ---*/

        const i32a = new Int32Array(
          new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT * 4)
        );
        const valueOf = {
          valueOf() {
            return true;
          }
        };
        const toPrimitive = {
          [Symbol.toPrimitive]() {
            return true;
          }
        };
        let outcomes = [];
        let start = $262.agent.monotonicNow();

        (function wait() {
          if ($262.agent.monotonicNow() - start > 1000) {
            $DONE("timed out waiting for Atomics.waitAsync Promise.all");
            return;
          }
          if (outcomes.length) {
            assert.sameValue(outcomes[0], 'timed-out');
            assert.sameValue(outcomes[1], 'timed-out');
            assert.sameValue(outcomes[2], 'timed-out');
            $DONE();
            return;
          }

          $262.agent.setTimeout(wait, 0);
        })();

        Promise.all([
          Atomics.waitAsync(i32a, 0, 0, true).value,
          Atomics.waitAsync(i32a, 0, 0, valueOf).value,
          Atomics.waitAsync(i32a, 0, 0, toPrimitive).value,
        ]).then(results => {
          outcomes = results;
        }, $DONE);
        "#,
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn runner_exposes_detach_array_buffer_and_gc_through_external_embedding() {
    let root = make_temp_dir();
    let entry_path = root.join("detach.js");

    let _report = run_passing_test(
        &entry_path,
        r#"
        let buffer = new ArrayBuffer(4);
        assert.sameValue(buffer.byteLength, 4);
        assert.sameValue($262.gc(), undefined);
        assert.sameValue(typeof $262.gc, "function");
        $262.detachArrayBuffer(buffer);
        assert.sameValue(buffer.byteLength, 0);
        "#,
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn runner_supports_new_pure_helper_includes() {
    let root = make_temp_dir();
    let entry_path = root.join("helpers.js");

    let _report = run_passing_test(
        &entry_path,
        r"
        /*---
        includes: [compareIterator.js]
        ---*/
        let count = 0;
        let iterator = {
          next: function() {
            count += 1;
            if (count === 1) {
              return { value: 3, done: false };
            }
            if (count === 2) {
              return { value: 5, done: false };
            }
            return { value: undefined, done: true };
          }
        };
        assert.compareIterator(iterator, [
          function(value) { assert.sameValue(value, 3); },
          function(value) { assert.sameValue(value, 5); },
        ]);
        ",
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn runner_fails_wrong_runtime_negative_type() {
    let root = make_temp_dir();
    let entry_path = root.join("wrong-runtime-negative.js");

    let report = run_single_test(
        &entry_path,
        r#"
        /*---
        negative:
          phase: runtime
          type: TypeError
        ---*/
        throw new RangeError("wrong type");
        "#,
    );

    assert_passed(&report, 0, 0);
    assert_failed(&report, 1, 2);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn runner_fails_wrong_resolution_negative_type() {
    let root = make_temp_dir();
    let entry_path = root.join("wrong-resolution-negative.js");
    let dependency_path = root.join("dependency.js");
    fs::write(
        &dependency_path,
        r"
        /*---
        flags: [module]
        ---*/
        export const present = 1;
        ",
    )
    .expect("dependency should be written");

    let report = run_single_test(
        &entry_path,
        r#"
        /*---
        flags: [module]
        negative:
          phase: resolution
          type: TypeError
        ---*/
        import { missing } from "./dependency.js";
        "#,
    );

    assert_passed(&report, 0, 0);
    assert_failed(&report, 1, 1);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn runner_reports_skips_separately_from_failures() {
    let root = make_temp_dir();
    let entry_path = root.join("skipped.js");

    let (stdout, report) = run_single_test_with_output(
        &entry_path,
        r"
        /*---
        includes: [unsupportedHelper.js]
        ---*/
        ",
    );

    assert!(
        stdout.contains("Failed files:         0"),
        "unexpected stdout:\n{stdout}"
    );
    assert!(
        stdout.contains("Skipped files:        1"),
        "unexpected stdout:\n{stdout}"
    );
    assert!(
        !stdout.contains("skip included"),
        "unexpected stdout:\n{stdout}"
    );
    assert_failed(&report, 0, 0);
    assert_skipped(&report, 1, 2);
    assert!(
        report.contains("unsupported harness include: unsupportedHelper.js"),
        "unexpected report:\n{report}"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn runner_passes_async_helper_self_test_without_async_flag_done() {
    let report = run_filtered_test("harness/asyncHelpers-asyncTest-without-async-flag.js");
    assert_passed(&report, 1, 2);
}

#[test]
fn runner_passes_assert_tostring_harness_self_test() {
    let report = run_filtered_test("harness/assert-tostring.js");
    assert_passed(&report, 1, 2);
}

#[test]
fn runner_passes_compare_array_arguments_harness_self_test() {
    let report = run_filtered_test("harness/compare-array-arguments.js");
    assert_passed(&report, 1, 2);
}

#[test]
fn runner_passes_decimal_to_hex_string_harness_self_test() {
    let report = run_filtered_test("harness/decimalToHexString.js");
    assert_passed(&report, 1, 2);
}

#[test]
fn runner_passes_assert_relative_date_ms_harness_self_test() {
    let report = run_filtered_test("harness/assertRelativeDateMs.js");
    assert_passed(&report, 1, 2);
}

#[test]
fn runner_passes_compare_array_sparse_harness_self_test() {
    let report = run_filtered_test("harness/compare-array-sparse.js");
    assert_passed(&report, 1, 2);
}

#[test]
fn runner_passes_deep_equal_mapset_harness_self_test() {
    let report = run_filtered_test("harness/deepEqual-mapset.js");
    assert_passed(&report, 1, 2);
}

#[test]
fn runner_passes_deep_equal_primitives_bigint_harness_self_test() {
    let report = run_filtered_test("harness/deepEqual-primitives-bigint.js");
    assert_passed(&report, 1, 2);
}

#[test]
fn runner_passes_well_known_intrinsics_helper_smoke_test() {
    let root = make_temp_dir();
    let entry_path = root.join("well-known-intrinsics.js");

    let _report = run_passing_test(
        &entry_path,
        r"
        /*---
        includes: [wellKnownIntrinsicObjects.js]
        ---*/
        var intrinsicArray = getWellKnownIntrinsicObject('%Array%');
        assert.sameValue(intrinsicArray, Array);
        assert.throws(Test262Error, function () {
          getWellKnownIntrinsicObject('%AsyncFromSyncIteratorPrototype%');
        });
        ",
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn runner_temporal_helper_formats_well_known_symbol_property_names() {
    let root = make_temp_dir();
    let entry_path = root.join("temporal-symbol-formatting.js");

    let _report = run_passing_test(
        &entry_path,
        r#"
        /*---
        includes: [temporalHelpers.js, compareArray.js]
        ---*/
        const calls = [];
        const object = {};
        TemporalHelpers.observeProperty(calls, object, Symbol.asyncIterator, 1, "items");
        object[Symbol.asyncIterator];
        assert.compareArray(calls, ["get items[Symbol.asyncIterator]"]);
        "#,
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn runner_temporal_helper_exposes_plain_date_time_fast_path_check() {
    let root = make_temp_dir();
    let entry_path = root.join("temporal-plain-date-time-fast-path.js");

    let _report = run_passing_test(
        &entry_path,
        r#"
        /*---
        includes: [temporalHelpers.js, compareArray.js]
        ---*/
        let called = false;
        TemporalHelpers.checkToTemporalPlainDateTimeFastPath((date) => {
          called = true;
          assert(date instanceof Temporal.PlainDate);
          assert.sameValue(date.toString(), "2000-05-02");
        });
        assert.sameValue(called, true);
        "#,
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn runner_temporal_helper_plain_date_time_fast_path_check_catches_getter_reads() {
    let root = make_temp_dir();
    let entry_path = root.join("temporal-plain-date-time-fast-path-getter-read.js");

    let report = run_single_test(
        &entry_path,
        r"
        /*---
        includes: [temporalHelpers.js, compareArray.js]
        ---*/
        TemporalHelpers.checkToTemporalPlainDateTimeFastPath((date) => {
          date.year;
        });
        ",
    );

    assert_passed(&report, 0, 0);
    assert_failed(&report, 1, 2);
    assert!(
        report.contains("runtime error: Test262Error"),
        "unexpected report:\n{report}"
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn runner_temporal_helper_exposes_plain_year_month_string_lists() {
    let root = make_temp_dir();
    let entry_path = root.join("temporal-plain-year-month-string-lists.js");

    let _report = run_passing_test(
        &entry_path,
        r#"
        /*---
        includes: [temporalHelpers.js, compareArray.js]
        ---*/
        assert.compareArray(TemporalHelpers.ISO.plainYearMonthStringsInvalid(), [
          "2020-13",
          "1976-11[u-ca=gregory]",
          "1976-11[u-ca=hebrew]",
          "1976-11[U-CA=iso8601]",
          "1976-11[u-CA=iso8601]",
          "1976-11[FOO=bar]",
          "+999999-01",
          "-999999-01",
        ]);
        assert.compareArray(TemporalHelpers.ISO.plainYearMonthStringsValid(), [
          "1976-11",
          "1976-11-10",
          "1976-11-01T09:00:00+00:00",
          "1976-11-01T00:00:00+05:00",
          "197611",
          "+00197611",
          "1976-11-18T15:23:30.1-02:00",
          "1976-11-18T152330.1+00:00",
          "19761118T15:23:30.1+00:00",
          "1976-11-18T15:23:30.1+0000",
          "1976-11-18T152330.1+0000",
          "19761118T15:23:30.1+0000",
          "19761118T152330.1+00:00",
          "19761118T152330.1+0000",
          "+001976-11-18T152330.1+00:00",
          "+0019761118T15:23:30.1+00:00",
          "+001976-11-18T15:23:30.1+0000",
          "+001976-11-18T152330.1+0000",
          "+0019761118T15:23:30.1+0000",
          "+0019761118T152330.1+00:00",
          "+0019761118T152330.1+0000",
          "1976-11-18T15:23",
          "1976-11-18T15",
          "1976-11-18",
        ]);
        assert.compareArray(TemporalHelpers.ISO.plainYearMonthStringsValidNegativeYear(), [
          "-009999-11",
        ]);
        "#,
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn runner_passes_well_known_intrinsics_harness_self_test() {
    let report = run_filtered_test("harness/wellKnownIntrinsicObjects.js");
    assert_passed(&report, 1, 2);
}

#[test]
fn runner_passes_tco_helper_harness_self_test() {
    let report = run_filtered_test("harness/tcoHelper.js");
    assert_passed(&report, 1, 2);
}

#[test]
fn runner_passes_tco_non_eval_global_regression() {
    let report =
        run_filtered_test_with_timeout("language/expressions/call/tco-non-eval-global.js", 5000);
    assert_passed(&report, 1, 1);
}

#[test]
fn runner_passes_tco_non_eval_function_regression() {
    let report =
        run_filtered_test_with_timeout("language/expressions/call/tco-non-eval-function.js", 5000);
    assert_passed(&report, 1, 1);
}

#[test]
fn runner_passes_tco_coalesce_regressions() {
    for test in [
        "language/expressions/coalesce/tco-pos-null.js",
        "language/expressions/coalesce/tco-pos-undefined.js",
    ] {
        let report = run_filtered_test_with_timeout(test, 5000);
        assert_passed(&report, 1, 1);
    }
}
