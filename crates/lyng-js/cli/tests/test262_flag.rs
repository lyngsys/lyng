//! Process-level integration tests for the `lyng-js --test262` flag.
//!
//! The harness extension installs the `print` global on the global object, so
//! `print("ok")` is observable on the child process's stdout. The library-level
//! tests in `lib.rs` verify exit codes through `run_with_io`; these tests
//! additionally cover the captured-stdout contract that external Test262
//! runners (test262.fyi, qjs-like adapters) rely on.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn test262_flag_prints_to_stdout() {
    let script = TempScript::new(
        "cli-test262-stdout.js",
        r#"
        if (typeof $262 !== "object") throw new Error("no $262");
        for (const k of ["evalScript", "createRealm", "detachArrayBuffer", "gc"]) {
            if (typeof $262[k] !== "function") throw new Error("missing " + k);
        }
        if (typeof print !== "function") throw new Error("no print");
        print("ok");
        "#,
    );
    let output = Command::new(env!("CARGO_BIN_EXE_lyng-js"))
        .arg("--test262")
        .arg(script.path())
        .output()
        .expect("lyng-js binary should run");

    assert!(
        output.status.success(),
        "lyng-js --test262 exited with {:?}, stderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout should stay UTF-8");
    assert!(
        stdout.contains("ok"),
        "expected stdout to contain `ok`, got: {stdout:?}"
    );
}

#[test]
fn default_cli_leaves_test262_undefined() {
    let script = TempScript::new(
        "cli-default-no-test262.js",
        r#"if (typeof $262 !== "undefined") throw new Error("$262 leaked");"#,
    );
    let output = Command::new(env!("CARGO_BIN_EXE_lyng-js"))
        .arg(script.path())
        .output()
        .expect("lyng-js binary should run");

    assert!(
        output.status.success(),
        "default lyng-js exited with {:?}, stderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
}

struct TempScript {
    path: PathBuf,
}

impl TempScript {
    fn new(label: &str, source: &str) -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should stay after UNIX_EPOCH")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("lyng-js-test262-{unique}-{label}"));
        fs::write(&path, source).expect("temp script should be writable");
        Self { path }
    }

    fn path(&self) -> &std::path::Path {
        &self.path
    }
}

impl Drop for TempScript {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}
