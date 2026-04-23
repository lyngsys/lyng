use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
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
        "lyng-js-test262-timeout-{}-{}-{}",
        std::process::id(),
        nonce,
        counter
    ));
    fs::create_dir_all(&path).expect("temp dir should be created");
    path
}

#[test]
fn whole_suite_runner_times_out_hanging_tests() {
    let root = make_temp_dir();
    let hang_path = root.join("hang.js");
    let report_path = root.join("report.md");
    fs::write(&hang_path, "while (true) {}\n").expect("fixture should be written");

    let output = Command::new(env!("CARGO_BIN_EXE_lyng-js-test262"))
        .args([
            "--filter",
            hang_path.to_str().expect("path should be utf-8"),
            "--report",
            report_path.to_str().expect("path should be utf-8"),
            "--timeout-ms",
            "50",
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
    assert!(report.contains("timeout after"));
    assert!(report.contains("| Failed | `1` |"));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn worker_mode_handles_multiple_requests_in_one_process() {
    let root = make_temp_dir();
    let first_path = root.join("01-pass.js");
    let second_path = root.join("02-pass.js");
    fs::write(&first_path, "assert.sameValue(1, 1);\n").expect("first fixture should be written");
    fs::write(&second_path, "assert.sameValue(2, 2);\n").expect("second fixture should be written");

    let mut child = Command::new(env!("CARGO_BIN_EXE_lyng-js-test262"))
        .arg("--worker")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("worker should execute");
    let mut stdin = child.stdin.take().expect("stdin should be piped");
    let stdout = child.stdout.take().expect("stdout should be piped");

    writeln!(
        stdin,
        "1\t{}",
        first_path.to_str().expect("path should be utf-8")
    )
    .expect("first request should be written");
    writeln!(
        stdin,
        "2\t{}",
        second_path.to_str().expect("path should be utf-8")
    )
    .expect("second request should be written");
    drop(stdin);

    let mut stdout = BufReader::new(stdout);
    let mut first_response = String::new();
    let mut second_response = String::new();
    stdout
        .read_line(&mut first_response)
        .expect("first response should be readable");
    stdout
        .read_line(&mut second_response)
        .expect("second response should be readable");

    let output = child.wait_with_output().expect("worker should finish");

    assert!(
        output.status.success(),
        "worker exited with {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    assert!(
        first_response.contains("__lyng_js_test262_result__:1:PASS"),
        "unexpected first response:\n{first_response}"
    );
    assert!(
        second_response.contains("__lyng_js_test262_result__:2:PASS"),
        "unexpected second response:\n{second_response}"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn whole_suite_runner_recovers_after_timed_out_test() {
    let root = make_temp_dir();
    let hang_path = root.join("01-hang.js");
    let pass_path = root.join("02-pass.js");
    let report_path = root.join("report.md");
    fs::write(&hang_path, "while (true) {}\n").expect("hang fixture should be written");
    fs::write(&pass_path, "assert.sameValue(1, 1);\n").expect("pass fixture should be written");

    let output = Command::new(env!("CARGO_BIN_EXE_lyng-js-test262"))
        .args([
            "--filter",
            root.to_str().expect("path should be utf-8"),
            "--report",
            report_path.to_str().expect("path should be utf-8"),
            "--timeout-ms",
            "50",
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
    assert!(report.contains("timeout after"));
    assert!(report.contains("| Passed | `1` |"));
    assert!(report.contains("| Failed | `1` |"));

    let _ = fs::remove_dir_all(root);
}
