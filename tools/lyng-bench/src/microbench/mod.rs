//! `lyng-bench microbench` — per-opcode ns/dispatch with confidence interval.
//!
//! Loop construction: each opcode has a hand-written JS source snippet that
//! exercises the opcode in a tight inner loop; the harness compiles it,
//! runs it for N iterations, and divides total time by dispatch count.

use std::hint::black_box;
use std::path::PathBuf;

use lyng_builtins::BootstrapMode;
use lyng_common::{AtomTable, SourceId};
use lyng_compiler::compile_script;
use lyng_env::Runtime;
use lyng_host::NoopHostHooks;
use lyng_parser::parse_script;
use lyng_sema::analyze_script;
use lyng_types::Value;
use lyng_vm::{Vm, VmError};

mod snippets;
mod timing;
pub use snippets::{all_snippets, for_opcode, Snippet};

#[derive(Debug, Clone, PartialEq, Eq)]
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

    let config = crate::hot_opcodes::HotOpcodesConfig::load(&options.opcodes_config)?;
    let snippet_table = snippets::all_snippets();

    let mut report_lines: Vec<String> = Vec::new();
    report_lines.push("# Microbench Baseline".to_string());
    report_lines.push(String::new());
    report_lines.push(format!("Samples per opcode: {}", options.samples));
    report_lines.push(format!("Inner iters per sample: {}", options.iters));
    report_lines.push(String::new());
    report_lines.push(
        "| Opcode | Samples | Median ns/dispatch | Min | Max | CI95 half-width | Snippet ratio |"
            .to_string(),
    );
    report_lines.push("|---|---:|---:|---:|---:|---:|---|".to_string());

    for entry in &config.opcodes {
        let Some(snippet) = snippet_table.get(entry.name.as_str()) else {
            report_lines.push(format!(
                "| `{}` | — | no snippet | — | — | — | — |",
                entry.name
            ));
            continue;
        };

        let samples = match run_snippet(snippet, options.iters, options.samples) {
            Ok(s) => s,
            Err(err) => {
                report_lines.push(format!(
                    "| `{}` | — | error: {err} | — | — | — | — |",
                    entry.name
                ));
                continue;
            }
        };
        let stats = timing::SampleStats::from_samples(samples);

        report_lines.push(format!(
            "| `{}` | {} | {:.2} | {:.2} | {:.2} | ±{:.2} | {} ops/iter |",
            entry.name,
            stats.samples.len(),
            stats.median_ns_per_dispatch,
            stats.min_ns_per_dispatch,
            stats.max_ns_per_dispatch,
            stats.ci95_half_width_ns,
            snippet.opcodes_per_iter,
        ));
    }

    let body = report_lines.join("\n") + "\n";
    if let Some(out) = options.output.as_ref() {
        std::fs::write(out, &body).map_err(|err| format!("write {}: {err}", out.display()))?;
        println!("microbench: wrote {}", out.display());
    } else {
        println!("{body}");
    }
    Ok(())
}

/// Compile a snippet and run it `samples` times for `iters` inner-loop
/// iterations each, returning a timing sample per run. One warm-up sample
/// is executed first and discarded so the recorded samples measure steady
/// state, not the first-eval install cost.
///
/// The script source is the snippet's `bench` function declaration followed
/// by a script-level `bench(iters)` call so that re-evaluating the same
/// installed unit drives real opcode dispatches. Each call to this function
/// uses a fresh `Runtime` + `Vm` so feedback caches, shape state, and tier
/// transitions don't bleed across opcodes.
fn run_snippet(
    snippet: &snippets::Snippet,
    iters: u64,
    samples: usize,
) -> Result<Vec<timing::Sample>, String> {
    let src = format!("{}\nbench({});\n", snippet.source, iters);

    let mut atoms = AtomTable::new();
    let source_id = SourceId::new(1);
    let parsed = parse_script(&mut atoms, source_id, &src);
    if parsed.diagnostics.has_errors() {
        return Err(format!(
            "parse errors compiling {}: {:?}",
            snippet.opcode,
            parsed.diagnostics.as_slice()
        ));
    }
    let sema = analyze_script(&parsed, &atoms);
    if sema.diagnostics.has_errors() {
        return Err(format!(
            "sema errors compiling {}: {:?}",
            snippet.opcode,
            sema.diagnostics.as_slice()
        ));
    }
    let unit = compile_script(&parsed, &sema, &mut atoms)
        .map_err(|err| format!("lowering failed for {}: {err:?}", snippet.opcode))?;

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent
        .default_realm()
        .ok_or_else(|| "default realm should exist for microbench".to_string())?;
    let realm_id = realm.id();
    let mut vm = Vm::new();
    let _ = vm
        .bootstrap_realm(agent, realm_id, BootstrapMode::SpecOnly)
        .map_err(|err| format!("spec bootstrap failed: {err:?}"))?;
    let installed = vm
        .install_script(agent, realm_id, &unit)
        .map_err(|err| format!("install_script failed for {}: {err:?}", snippet.opcode))?;
    Vm::instantiate_global_script(agent, &realm, unit.instantiation_plan()).map_err(|err| {
        format!(
            "instantiate_global_script failed for {}: {err:?}",
            snippet.opcode
        )
    })?;

    // Warm-up sample: discard timing so we measure steady-state dispatch
    // cost, not the first-eval install + initial-jit overhead.
    let value = vm
        .installed_eval(agent, installed, realm.global_env(), realm.global_env())
        .run()
        .map_err(|err| format!("warmup eval failed for {}: {err:?}", snippet.opcode))?;
    black_box(value.bits());

    let dispatches_per_sample = iters
        .checked_mul(u64::from(snippet.opcodes_per_iter))
        .ok_or_else(|| format!("dispatch count overflow for {}", snippet.opcode))?;

    let mut out = Vec::with_capacity(samples);
    for sample_index in 0..samples {
        let mut eval_result: Option<Result<Value, VmError>> = None;
        let sample = timing::time_once(dispatches_per_sample, || {
            eval_result = Some(
                vm.installed_eval(agent, installed, realm.global_env(), realm.global_env())
                    .run(),
            );
        });
        let value = eval_result
            .expect("time_once closure always assigns eval_result")
            .map_err(|err| {
                format!(
                    "sample {} eval failed for {}: {err:?}",
                    sample_index + 1,
                    snippet.opcode
                )
            })?;
        black_box(value.bits());
        out.push(sample);
    }
    Ok(out)
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
    let mut opcodes_config = PathBuf::from("tools/lyng-bench/hot-opcodes.toml");
    let mut baseline: Option<PathBuf> = None;
    let mut samples = 7;
    let mut iters = 5_000_000;
    let mut require_isolation = false;
    let mut output: Option<PathBuf> = None;

    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--opcodes-config" => {
                opcodes_config = iter
                    .next()
                    .ok_or("--opcodes-config requires a path")?
                    .into();
            }
            "--baseline" => {
                baseline = Some(iter.next().ok_or("--baseline requires a path")?.into());
            }
            "--samples" => {
                samples = iter
                    .next()
                    .and_then(|s| s.parse().ok())
                    .ok_or("--samples requires a number")?;
            }
            "--iters" => {
                iters = iter
                    .next()
                    .and_then(|s| s.parse().ok())
                    .ok_or("--iters requires a number")?;
            }
            "--require-isolation" => require_isolation = true,
            "--output" => output = Some(iter.next().ok_or("--output requires a path")?.into()),
            "--help" | "-h" => return Err(help_text()),
            other => {
                return Err(format!(
                    "microbench: unknown arg {other}\n\n{}",
                    help_text()
                ))
            }
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
        "Usage: lyng-bench microbench [options]",
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
        let config = crate::hot_opcodes::HotOpcodesConfig::load(std::path::Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/hot-opcodes.toml"
        )))
        .expect("load");
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
