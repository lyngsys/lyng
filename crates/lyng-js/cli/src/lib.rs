//! Thin `lyng-js` CLI embedding for the shared Phase 5 runtime path.
//!
//! Ownership: `lyng_js_cli` owns command-line parsing, filesystem-backed host
//! hooks, and user-facing process reporting. It is a spec-only embedding over
//! the shared default-realm bootstrap path used by the VM and the Phase 5
//! harness runner; it does not own bootstrap semantics, compilation semantics,
//! or VM semantics.

#![allow(
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::return_self_not_must_use,
    reason = "CLI embedding surface keeps command-domain names and cheap accessors explicit for callers"
)]

mod error;
mod execution;
mod extensions;
mod host;

use lyng_js_builtins::BootstrapMode;
use std::ffi::OsString;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub use error::{CliError, CliErrorKind, CliResult};

const HELP_TEXT: &str = "\
Usage: lyng-js <entry.js|entry.mjs>

Runs one Lyng JS entry file through the shared default-realm bootstrap.
The CLI does not install harness globals, browser globals, or Node-style globals.

Options:
  --shell  Install non-spec shell globals (currently: print).
";

/// High-level CLI command.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CliCommand {
    Help,
    Run(CliInvocation),
}

/// Parsed CLI invocation for one script run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CliInvocation {
    script_path: PathBuf,
    bootstrap_mode: BootstrapMode,
    shell_mode: bool,
}

impl CliInvocation {
    #[inline]
    pub const fn new(
        script_path: PathBuf,
        bootstrap_mode: BootstrapMode,
        shell_mode: bool,
    ) -> Self {
        Self {
            script_path,
            bootstrap_mode,
            shell_mode,
        }
    }

    #[inline]
    pub fn script_path(&self) -> &Path {
        &self.script_path
    }

    #[inline]
    pub fn is_module_entry(&self) -> bool {
        self.script_path
            .extension()
            .and_then(|extension| extension.to_str())
            == Some("mjs")
    }

    #[inline]
    pub const fn bootstrap_mode(&self) -> BootstrapMode {
        self.bootstrap_mode
    }

    #[inline]
    pub const fn shell_mode(&self) -> bool {
        self.shell_mode
    }
}

#[inline]
pub const fn help_text() -> &'static str {
    HELP_TEXT
}

/// Parses one CLI command from process arguments.
///
/// # Errors
/// Returns an error when the argument list is malformed.
pub fn parse_args(args: impl IntoIterator<Item = OsString>) -> CliResult<CliCommand> {
    let mut args = args.into_iter();
    let _program_name = args.next();
    let mut shell_mode = false;
    let mut script_path = None;

    for arg in args.by_ref() {
        if arg == "-h" || arg == "--help" {
            return Ok(CliCommand::Help);
        }
        if arg == "--shell" {
            shell_mode = true;
            continue;
        }
        if arg.to_string_lossy().starts_with('-') {
            return Err(CliError::usage(format!(
                "unknown option `{}`\n\n{}",
                arg.to_string_lossy(),
                help_text()
            )));
        }
        script_path = Some(arg);
        break;
    }
    let Some(script_path) = script_path else {
        return Ok(CliCommand::Help);
    };

    if let Some(extra) = args.next() {
        return Err(CliError::usage(format!(
            "unexpected trailing argument `{}`\n\n{}",
            extra.to_string_lossy(),
            help_text()
        )));
    }

    Ok(CliCommand::Run(CliInvocation::new(
        PathBuf::from(script_path),
        BootstrapMode::SpecOnly,
        shell_mode,
    )))
}

/// Runs one parsed CLI command.
///
/// # Errors
/// Returns an error when parsing selected files or embedding operations fail.
pub fn run(command: CliCommand) -> CliResult<i32> {
    let mut stdout = io::stdout();
    let mut stderr = io::stderr();
    run_with_io(command, &mut stdout, &mut stderr)
}

fn run_with_io(
    command: CliCommand,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> CliResult<i32> {
    match command {
        CliCommand::Help => {
            stdout
                .write_all(help_text().as_bytes())
                .map_err(CliError::io)?;
            Ok(0)
        }
        CliCommand::Run(invocation) => execution::run_script(&invocation, stderr),
    }
}

/// Parses and runs the current process invocation.
///
/// # Errors
/// Returns an error when argument parsing or execution fails.
pub fn run_from_env() -> CliResult<i32> {
    let command = parse_args(std::env::args_os())?;
    run(command)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn parse_args_defaults_to_help_without_script() {
        let command = parse_args([OsString::from("lyng-js")]).expect("no-arg parse should succeed");

        assert_eq!(command, CliCommand::Help);
    }

    #[test]
    fn parse_args_rejects_trailing_arguments() {
        let error = parse_args([
            OsString::from("lyng-js"),
            OsString::from("demo.js"),
            OsString::from("alpha"),
        ])
        .expect_err("unexpected trailing arguments should fail");

        assert_eq!(error.kind(), CliErrorKind::Usage);
        assert!(error.message().contains("unexpected trailing argument"));
    }

    #[test]
    fn parse_args_accepts_shell_flag() {
        let command = parse_args([
            OsString::from("lyng-js"),
            OsString::from("--shell"),
            OsString::from("x.js"),
        ])
        .expect("shell-mode parse should succeed");

        let CliCommand::Run(invocation) = command else {
            panic!("shell flag should produce a run invocation");
        };
        assert_eq!(invocation.script_path(), Path::new("x.js"));
        assert!(invocation.shell_mode());
    }

    #[test]
    fn parse_args_rejects_shell_after_script() {
        let error = parse_args([
            OsString::from("lyng-js"),
            OsString::from("x.js"),
            OsString::from("--shell"),
        ])
        .expect_err("shell flag after the script path should fail");

        assert_eq!(error.kind(), CliErrorKind::Usage);
        assert!(error.message().contains("unexpected trailing argument"));
    }

    #[test]
    fn run_help_returns_success() {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        assert_eq!(
            run_with_io(CliCommand::Help, &mut stdout, &mut stderr),
            Ok(0)
        );
        assert_eq!(String::from_utf8(stdout).unwrap(), help_text());
        assert!(stderr.is_empty());
    }

    #[test]
    fn run_script_executes_through_spec_only_bootstrap() {
        let script = TempScript::new(
            "cli-spec-only.js",
            r#"
            if (typeof $262 !== "undefined") {
                throw new Error("$262 leaked into the CLI realm");
            }
            if (typeof globalThis !== "object") {
                throw new Error("globalThis is missing");
            }
            "#,
        );
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let exit_code = run_with_io(
            CliCommand::Run(CliInvocation::new(
                script.path().to_path_buf(),
                BootstrapMode::SpecOnly,
                false,
            )),
            &mut stdout,
            &mut stderr,
        )
        .expect("script execution should succeed");

        assert_eq!(exit_code, 0);
        assert!(stdout.is_empty());
        assert!(stderr.is_empty());
    }

    #[test]
    fn run_script_in_shell_mode_installs_print() {
        let script = TempScript::new("cli-shell-print.js", r#"print("hello");"#);
        let command = parse_args([
            OsString::from("lyng-js"),
            OsString::from("--shell"),
            script.path().as_os_str().to_owned(),
        ])
        .expect("shell-mode parse should succeed");
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let exit_code =
            run_with_io(command, &mut stdout, &mut stderr).expect("shell print should execute");

        assert_eq!(exit_code, 0);
        assert!(stdout.is_empty());
        assert!(stderr.is_empty());
    }

    #[test]
    fn run_script_reports_parse_errors_to_stderr() {
        let script = TempScript::new("cli-parse-error.js", "let = ;");
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let exit_code = run_with_io(
            CliCommand::Run(CliInvocation::new(
                script.path().to_path_buf(),
                BootstrapMode::SpecOnly,
                false,
            )),
            &mut stdout,
            &mut stderr,
        )
        .expect("parse failures should produce an exit status, not a CLI error");

        let stderr = String::from_utf8(stderr).expect("stderr should stay UTF-8");
        assert_eq!(exit_code, 1);
        assert!(stdout.is_empty());
        assert!(stderr.contains("error:"));
        assert!(stderr.contains("cli-parse-error.js"));
    }

    #[test]
    fn run_script_reports_uncaught_engine_errors_via_host_path() {
        let script = TempScript::new("cli-throw.js", r#"throw new TypeError("boom");"#);
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let exit_code = run_with_io(
            CliCommand::Run(CliInvocation::new(
                script.path().to_path_buf(),
                BootstrapMode::SpecOnly,
                false,
            )),
            &mut stdout,
            &mut stderr,
        )
        .expect("uncaught exceptions should produce an exit status, not a CLI error");

        let stderr = String::from_utf8(stderr).expect("stderr should stay UTF-8");
        assert_eq!(exit_code, 1);
        assert!(stdout.is_empty());
        assert!(stderr.contains("Uncaught exception: TypeError: boom"));
    }

    #[test]
    fn run_module_executes_through_host_backed_module_loading() {
        let workspace = TempWorkspace::new();
        let entry = workspace.write(
            "entry.mjs",
            "import value from './dep.mjs'; export default value; if (typeof import.meta.url !== 'string') { throw new Error('bad import.meta'); }",
        );
        let _ = workspace.write("dep.mjs", "export default import.meta.url;");
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let exit_code = run_with_io(
            CliCommand::Run(CliInvocation::new(entry, BootstrapMode::SpecOnly, false)),
            &mut stdout,
            &mut stderr,
        )
        .expect("module execution should succeed");

        assert_eq!(exit_code, 0);
        assert!(stdout.is_empty());
        assert!(stderr.is_empty());
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
            let path = std::env::temp_dir().join(format!("lyng-js-cli-{unique}-{label}"));
            fs::write(&path, source).expect("temp script should be writable");
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TempScript {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.path);
        }
    }

    struct TempWorkspace {
        root: PathBuf,
    }

    impl TempWorkspace {
        fn new() -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system clock should stay after UNIX_EPOCH")
                .as_nanos();
            let root = std::env::temp_dir().join(format!("lyng-js-cli-workspace-{unique}"));
            fs::create_dir_all(&root).expect("temp workspace should be creatable");
            Self { root }
        }

        fn write(&self, name: &str, source: &str) -> PathBuf {
            let path = self.root.join(name);
            fs::write(&path, source).expect("temp workspace file should be writable");
            path
        }
    }

    impl Drop for TempWorkspace {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}
