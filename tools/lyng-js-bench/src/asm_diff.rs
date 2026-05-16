//! `lyng-js-bench asm-diff` — capture, normalize, and diff handler asm
//! against committed per-arch baselines.

use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub struct AsmDiffOptions {
    pub opcodes_config: PathBuf,
    pub baseline_dir: PathBuf,
    pub output_dir: PathBuf,
    pub mode: Mode,
    pub capture_mode: CaptureMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Check,
    Update,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureMode {
    Auto,        // try cargo-asm first, fall back to rustc-emit-asm
    CargoAsm,    // force cargo-asm
    RustcEmit,   // force cargo rustc -- --emit=asm
}

/// Run the asm-diff subcommand.
///
/// # Errors
///
/// Returns Err with a user-facing message on parse failure, capture
/// failure, or — in `Check` mode — any per-opcode regression.
pub fn run(args: &[String]) -> Result<(), String> {
    let options = parse_args(args)?;
    // Placeholder: real implementation lands in later tasks.
    // For now, just print what we would do.
    println!("asm-diff: opcodes_config={}", options.opcodes_config.display());
    println!("asm-diff: baseline_dir={}", options.baseline_dir.display());
    println!("asm-diff: output_dir={}", options.output_dir.display());
    println!("asm-diff: mode={:?}", options.mode);
    println!("asm-diff: capture_mode={:?}", options.capture_mode);
    Err("asm-diff: not yet implemented (R-0 Task 9+)".into())
}

fn parse_args(args: &[String]) -> Result<AsmDiffOptions, String> {
    let mut opcodes_config = PathBuf::from("tools/lyng-js-bench/hot-opcodes.toml");
    let mut baseline_dir = PathBuf::from("reports/js/lyng-js/dsl-asm-baseline-aarch64");
    let mut output_dir = PathBuf::from("/tmp/asm-current");
    let mut mode = Mode::Check;
    let mut capture_mode = CaptureMode::Auto;

    let mut iter = args.iter().peekable();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--opcodes-config" => {
                opcodes_config = iter
                    .next()
                    .ok_or("--opcodes-config requires a path")?
                    .into();
            }
            "--baseline" => {
                baseline_dir = iter.next().ok_or("--baseline requires a path")?.into();
            }
            "--output" => {
                output_dir = iter.next().ok_or("--output requires a path")?.into();
            }
            "--mode" => match iter.next().map(String::as_str) {
                Some("check") => mode = Mode::Check,
                Some("update") => mode = Mode::Update,
                Some(other) => return Err(format!("--mode: unknown value {other}")),
                None => return Err("--mode requires check|update".into()),
            },
            "--capture-mode" => match iter.next().map(String::as_str) {
                Some("auto") => capture_mode = CaptureMode::Auto,
                Some("cargo-asm") => capture_mode = CaptureMode::CargoAsm,
                Some("rustc") => capture_mode = CaptureMode::RustcEmit,
                Some(other) => return Err(format!("--capture-mode: unknown value {other}")),
                None => return Err("--capture-mode requires auto|cargo-asm|rustc".into()),
            },
            "--help" | "-h" => {
                return Err(help_text());
            }
            other => return Err(format!("asm-diff: unknown argument {other}\n\n{}", help_text())),
        }
    }

    Ok(AsmDiffOptions {
        opcodes_config,
        baseline_dir,
        output_dir,
        mode,
        capture_mode,
    })
}

fn help_text() -> String {
    [
        "Usage: lyng-js-bench asm-diff [options]",
        "",
        "Options:",
        "  --opcodes-config PATH   Path to hot-opcodes.toml (default: tools/lyng-js-bench/hot-opcodes.toml)",
        "  --baseline DIR          Directory containing committed baselines",
        "  --output DIR            Directory for current-build asm capture",
        "  --mode check|update     check: fail on diff; update: overwrite baselines (default: check)",
        "  --capture-mode auto|cargo-asm|rustc  Capture backend (default: auto)",
    ]
    .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|p| (*p).to_string()).collect()
    }

    #[test]
    fn parses_defaults() {
        let opts = parse_args(&args(&[])).unwrap();
        assert_eq!(opts.mode, Mode::Check);
        assert_eq!(opts.capture_mode, CaptureMode::Auto);
    }

    #[test]
    fn parses_mode_update() {
        let opts = parse_args(&args(&["--mode", "update"])).unwrap();
        assert_eq!(opts.mode, Mode::Update);
    }

    #[test]
    fn rejects_unknown_mode() {
        let err = parse_args(&args(&["--mode", "bogus"])).unwrap_err();
        assert!(err.contains("unknown value"));
    }
}
