//! `lyng-js-bench microbench` — per-opcode ns/dispatch with confidence interval.
//!
//! Loop construction: each opcode has a hand-written JS source snippet that
//! exercises the opcode in a tight inner loop; the harness compiles it,
//! runs it for N iterations, and divides total time by dispatch count.

use std::path::PathBuf;

mod snippets;
mod timing;
pub use snippets::{Snippet, all_snippets, for_opcode};

#[derive(Debug, Clone, PartialEq)]
pub struct MicrobenchOptions {
    pub opcodes_config: PathBuf,
    pub baseline: Option<PathBuf>,
    pub samples: usize,
    pub iters: u64,
    pub require_isolation: bool,
    pub output: Option<PathBuf>,
}

/// Run the microbench subcommand.
///
/// # Errors
///
/// Returns Err on CLI parsing failure, isolation-gate failure, or any
/// per-opcode microbench error.
pub fn run(args: &[String]) -> Result<(), String> {
    let options = parse_args(args)?;
    if options.require_isolation {
        gate_on_loadavg()?;
    }
    println!("microbench: options = {options:?}");
    Err("microbench: not yet implemented (R-0 Task 13+)".into())
}

/// Abort if 1-min loadavg > 2.0.
///
/// # Errors
///
/// Returns Err if loadavg cannot be read or exceeds 2.0.
pub fn gate_on_loadavg() -> Result<(), String> {
    let avg = read_loadavg_one_min()?;
    if avg > 2.0 {
        return Err(format!(
            "isolation gate: 1-min load average {avg:.2} > 2.0; \
             run on a quiesced machine or pass without --require-isolation"
        ));
    }
    Ok(())
}

fn read_loadavg_one_min() -> Result<f64, String> {
    #[cfg(target_os = "linux")]
    {
        let text = std::fs::read_to_string("/proc/loadavg")
            .map_err(|err| format!("read /proc/loadavg: {err}"))?;
        let first = text
            .split_whitespace()
            .next()
            .ok_or("malformed /proc/loadavg")?;
        first
            .parse::<f64>()
            .map_err(|err| format!("parse loadavg: {err}"))
    }

    #[cfg(target_os = "macos")]
    {
        let output = std::process::Command::new("uptime")
            .output()
            .map_err(|err| format!("run uptime: {err}"))?;
        let text = String::from_utf8_lossy(&output.stdout);
        let after = text
            .split("load average")
            .nth(1)
            .ok_or("uptime: no load average")?;
        let nums: Vec<&str> = after
            .split(|c: char| !c.is_ascii_digit() && c != '.')
            .filter(|s| !s.is_empty())
            .collect();
        let first = nums.first().ok_or("uptime: no first loadavg number")?;
        first
            .parse::<f64>()
            .map_err(|err| format!("parse loadavg: {err}"))
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        Err("loadavg gate not implemented on this platform".into())
    }
}

fn parse_args(args: &[String]) -> Result<MicrobenchOptions, String> {
    let mut opcodes_config = PathBuf::from("tools/lyng-js-bench/hot-opcodes.toml");
    let mut baseline: Option<PathBuf> = None;
    let mut samples = 7;
    let mut iters = 5_000_000;
    let mut require_isolation = false;
    let mut output: Option<PathBuf> = None;

    let mut iter = args.iter().peekable();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--opcodes-config" => opcodes_config = iter.next().ok_or("--opcodes-config requires a path")?.into(),
            "--baseline" => baseline = Some(iter.next().ok_or("--baseline requires a path")?.into()),
            "--samples" => samples = iter.next().and_then(|s| s.parse().ok()).ok_or("--samples requires a number")?,
            "--iters" => iters = iter.next().and_then(|s| s.parse().ok()).ok_or("--iters requires a number")?,
            "--require-isolation" => require_isolation = true,
            "--output" => output = Some(iter.next().ok_or("--output requires a path")?.into()),
            "--help" | "-h" => return Err(help_text()),
            other => return Err(format!("microbench: unknown arg {other}\n\n{}", help_text())),
        }
    }

    Ok(MicrobenchOptions {
        opcodes_config,
        baseline,
        samples,
        iters,
        require_isolation,
        output,
    })
}

fn help_text() -> String {
    [
        "Usage: lyng-js-bench microbench [options]",
        "",
        "Options:",
        "  --opcodes-config PATH    Path to hot-opcodes.toml",
        "  --baseline PATH          Path to microbench-baseline.md for comparison",
        "  --samples N              Samples per opcode (default 7)",
        "  --iters N                Inner-loop iterations per sample (default 5_000_000)",
        "  --require-isolation      Abort if 1-min loadavg > 2.0",
        "  --output PATH            Write report to PATH (default stdout)",
    ]
    .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_defaults() {
        let opts = parse_args(&[]).unwrap();
        assert_eq!(opts.samples, 7);
        assert_eq!(opts.iters, 5_000_000);
        assert!(!opts.require_isolation);
    }

    #[test]
    fn rejects_missing_samples_value() {
        let err = parse_args(&["--samples".into()]).unwrap_err();
        assert!(err.contains("requires a number"));
    }

    #[test]
    fn snippets_cover_hot_opcodes_or_emit_warning() {
        let config = crate::hot_opcodes::HotOpcodesConfig::load(
            std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/hot-opcodes.toml")),
        ).expect("load");
        let snippets = snippets::all_snippets();
        let mut missing: Vec<&str> = Vec::new();
        for entry in &config.opcodes {
            if !snippets.contains_key(entry.name.as_str()) {
                missing.push(entry.name.as_str());
            }
        }
        // R-0 ships with snippets for the top ~4 by frequency; the rest
        // emit "no snippet" warnings until DSL-0b coverage.
        println!("opcodes without microbench snippet: {missing:?}");
        // No assertion failure: partial coverage is acceptable at R-0.
    }
}
