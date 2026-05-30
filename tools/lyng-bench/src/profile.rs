//! `lyng-bench profile` — VM-internal time-attribution profiler.
//!
//! Runs V8 v7 workloads in-process with a statistical sampler that bins
//! samples by (opcode x fast/slow path), then emits a ranked time-attribution
//! report (Markdown + JSON, schema `lyng-bench/profile/v1`). Optionally also
//! captures a samply profile for function-level drill-down (`--samply`).

#![allow(
    clippy::cast_precision_loss,
    reason = "profile time-share and rate reports intentionally convert integer sample/dispatch counts to f64 ratios"
)]

use std::path::Path;
use std::time::Duration;

use lyng_builtins::BootstrapMode;
use lyng_bytecode::Opcode;
use lyng_common::{AtomTable, SourceId};
use lyng_compiler::compile_script;
use lyng_env::Runtime;
use lyng_host::NoopHostHooks;
use lyng_parser::parse_script;
use lyng_sema::analyze_script;
use lyng_vm::{OpcodeDispatchCounts, SampleHistogram, SamplingProfiler, Vm};
use serde_json::{json, Value};
use std::hint::black_box;

use crate::v8suite::{
    build_count_harness, default_v8_root, ensure_path_exists, read_file, write_output, V8Workload,
    V8_WORKLOADS,
};

const DEFAULT_INTERVAL_US: u64 = 200;
const DEFAULT_SAMPLES: usize = 1;

pub(crate) struct Options {
    pub report_path: String,
    pub json_path: String,
    pub v8_root: String,
    pub samples: usize,
    pub interval_us: u64,
    pub filter: Option<String>,
    pub samply: bool,
    pub lyng_bin: String,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            report_path: "reports/lyng/profile.md".to_string(),
            json_path: "reports/lyng/profile.json".to_string(),
            v8_root: default_v8_root(),
            samples: DEFAULT_SAMPLES,
            interval_us: DEFAULT_INTERVAL_US,
            filter: None,
            samply: false,
            lyng_bin: "target/release/lyng".to_string(),
        }
    }
}

pub(crate) fn parse_options(args: &[String]) -> Result<Options, String> {
    let mut options = Options::default();
    let mut iter = args.iter().cloned();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--report" => {
                options.report_path = iter.next().ok_or("--report needs a path")?;
            }
            "--json" => {
                options.json_path = iter.next().ok_or("--json needs a path")?;
            }
            "--v8-root" => {
                options.v8_root = iter.next().ok_or("--v8-root needs a path")?;
            }
            "--filter" => {
                options.filter = Some(iter.next().ok_or("--filter needs a value")?);
            }
            "--samples" => {
                options.samples = iter
                    .next()
                    .ok_or("--samples needs a value")?
                    .parse()
                    .map_err(|_| "--samples must be a positive integer".to_string())?;
            }
            "--interval-us" => {
                options.interval_us = iter
                    .next()
                    .ok_or("--interval-us needs a value")?
                    .parse()
                    .map_err(|_| "--interval-us must be a positive integer".to_string())?;
            }
            "--lyng-bin" => {
                options.lyng_bin = iter.next().ok_or("--lyng-bin needs a path")?;
            }
            "--samply" => options.samply = true,
            "--help" | "-h" => return Err(help_text()),
            other => {
                return Err(format!(
                    "unknown profile option: {other}\n\n{}",
                    help_text()
                ))
            }
        }
    }
    if options.samples == 0 {
        return Err("--samples must be >= 1".to_string());
    }
    if options.interval_us == 0 {
        return Err("--interval-us must be >= 1".to_string());
    }
    Ok(options)
}

pub(crate) fn help_text() -> String {
    [
        "Usage: lyng-bench profile [options]",
        "",
        "Options:",
        "  --filter <name>      Only profile the named workload (e.g. RayTrace)",
        "  --samples <n>        Sampled runs to sum per workload (default: 1)",
        "  --interval-us <n>    Sampler tick interval in microseconds (default: 200)",
        "  --samply             Also capture a samply profile per workload",
        "  --lyng-bin <path>    lyng binary for --samply (default: target/release/lyng)",
        "  --v8-root <path>     V8 v7 sources dir",
        "  --report <path>      Markdown report path (default: reports/lyng/profile.md)",
        "  --json <path>        JSON report path (default: reports/lyng/profile.json)",
    ]
    .join("\n")
}

/// Per-workload accumulated result: summed dispatch counts + summed histogram.
struct WorkloadProfile {
    workload: V8Workload,
    dispatch: OpcodeDispatchCounts,
    histogram: SampleHistogram,
}

/// Runs the profile suite and writes Markdown + JSON reports.
///
/// # Errors
/// Returns an error when CLI parsing fails, the V8 root is missing, a
/// workload fails to compile, or a workload errors on every sample.
pub fn run(args: &[String]) -> Result<(), String> {
    let options = parse_options(args)?;
    if cfg!(debug_assertions) {
        eprintln!("warning: build with --release for meaningful measurements");
    }
    ensure_path_exists(&options.v8_root, "v8 benchmark root")?;
    let base_js = read_file(&Path::new(&options.v8_root).join("base.js"))?;

    let workloads: Vec<&V8Workload> = V8_WORKLOADS
        .iter()
        .filter(|w| {
            options
                .filter
                .as_ref()
                .is_none_or(|needle| w.name.eq_ignore_ascii_case(needle))
        })
        .collect();
    if workloads.is_empty() {
        return Err(format!(
            "no benchmarks matched filter `{}`. known: {}",
            options.filter.as_deref().unwrap_or("<none>"),
            V8_WORKLOADS
                .iter()
                .map(|w| w.name)
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    let mut profiles = Vec::with_capacity(workloads.len());
    for (index, workload) in workloads.iter().enumerate() {
        let benchmark_js = read_file(&Path::new(&options.v8_root).join(workload.file))?;
        let harness = build_count_harness(&base_js, &benchmark_js);
        let source_id = SourceId::new(
            u32::try_from(index + 1)
                .map_err(|_| "workload count exceeds SourceId range".to_string())?,
        );
        let profile = profile_workload(workload, &harness, source_id, &options)?;
        if options.samply {
            samply::capture(workload, &harness, &options)?;
        }
        profiles.push(profile);
    }

    write_output(&options.report_path, &render_markdown(&options, &profiles))?;
    write_output(
        &options.json_path,
        &serde_json::to_string_pretty(&render_json(&options, &profiles))
            .map_err(|e| format!("failed to render profile JSON: {e}"))?,
    )?;
    print_summary(&profiles, &options);
    Ok(())
}

fn profile_workload(
    workload: &V8Workload,
    harness: &str,
    source_id: SourceId,
    options: &Options,
) -> Result<WorkloadProfile, String> {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(&mut atoms, source_id, harness);
    if parsed.diagnostics.has_errors() {
        return Err(format!(
            "parse errors for {}: {:?}",
            workload.name,
            parsed.diagnostics.as_slice()
        ));
    }
    let sema = analyze_script(&parsed, &atoms);
    if sema.diagnostics.has_errors() {
        return Err(format!(
            "sema errors for {}: {:?}",
            workload.name,
            sema.diagnostics.as_slice()
        ));
    }
    let unit = compile_script(&parsed, &sema, &mut atoms)
        .map_err(|e| format!("lowering failed for {}: {e:?}", workload.name))?;

    let mut dispatch = OpcodeDispatchCounts::default();
    let mut histogram = None::<SampleHistogram>;
    let interval = Duration::from_micros(options.interval_us);
    for sample in 0..options.samples {
        let (sample_dispatch, sample_hist) = profile_once(workload, &unit, interval)
            .map_err(|e| format!("sample {} failed for {}: {e}", sample + 1, workload.name))?;
        // Sum dispatch counts across samples.
        let merged_pairs: Vec<(Opcode, u64)> = sample_dispatch
            .iter()
            .filter(|entry| entry.count() != 0)
            .map(|entry| (entry.opcode(), entry.count()))
            .chain(
                dispatch
                    .iter()
                    .filter(|e| e.count() != 0)
                    .map(|e| (e.opcode(), e.count())),
            )
            .collect();
        dispatch = OpcodeDispatchCounts::from_counts(merged_pairs);
        match histogram.as_mut() {
            Some(acc) => acc.merge(&sample_hist),
            None => histogram = Some(sample_hist),
        }
    }
    Ok(WorkloadProfile {
        // V8Workload is Copy; `workload` is &V8Workload, so deref once.
        workload: *workload,
        dispatch,
        // parse_options enforces samples >= 1, so the loop ran at least once.
        histogram: histogram.expect("samples >= 1 guarantees one histogram"),
    })
}

/// Function-level drill-down via the external `samply` profiler. Writes the
/// generated harness to a temp script and records `lyng --shell <script>`.
/// This is the microscope for splitting a single slow handler's internals;
/// the in-process sampler above is the default opcode-level signal.
pub(crate) mod samply {
    use super::{Options, V8Workload};
    use std::process::Command;

    pub(crate) fn capture(
        workload: &V8Workload,
        harness: &str,
        options: &Options,
    ) -> Result<(), String> {
        let script_path = std::env::temp_dir().join(format!(
            "lyng-profile-{}-{}.js",
            workload.name,
            std::process::id()
        ));
        std::fs::write(&script_path, harness)
            .map_err(|e| format!("failed to write samply script: {e}"))?;
        let out_path = format!("reports/lyng/samply-{}.json.gz", workload.name);
        if let Some(parent) = std::path::Path::new(&out_path).parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("failed to create samply output dir: {e}"))?;
        }

        let result = Command::new("samply")
            .arg("record")
            .arg("--save-only")
            .arg("-o")
            .arg(&out_path)
            .arg("--")
            .arg(&options.lyng_bin)
            .arg("--shell")
            .arg(&script_path)
            .status();

        match result {
            Ok(status) if status.success() => {
                println!(
                    "  samply: {} profile saved to {out_path} (open with `samply load {out_path}`)",
                    workload.name
                );
                Ok(())
            }
            Ok(status) => Err(format!("samply exited with {status} for {}", workload.name)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Err(
                "samply not found on PATH. Install with `cargo install samply`, or omit --samply."
                    .to_string(),
            ),
            Err(error) => Err(format!("failed to launch samply: {error}")),
        }
    }
}

/// Run one sampled execution of a precompiled `unit`, returning summed
/// dispatch counts + the sampler histogram for that run.
///
/// Note: `workload` is used only for its `name` in error messages —
/// `profile_once` operates entirely on the precompiled `unit` and never reads
/// `workload.file`, so callers may pass a sentinel `file` (e.g. `"n/a"` in
/// tests) without triggering a file read.
fn profile_once(
    workload: &V8Workload,
    unit: &lyng_bytecode::CompiledScriptUnit,
    interval: Duration,
) -> Result<(OpcodeDispatchCounts, SampleHistogram), String> {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent
        .default_realm()
        .ok_or_else(|| "default realm should exist".to_string())?;
    let realm_id = realm.id();
    let realm_record = realm;
    let mut vm = Vm::new();
    let _ = vm
        .bootstrap_realm(agent, realm_id, BootstrapMode::SpecOnly)
        .map_err(|e| format!("spec bootstrap failed: {e:?}"))?;
    let installed = vm
        .install_script(agent, realm_id, unit)
        .map_err(|e| format!("script install failed for {}: {e:?}", workload.name))?;
    vm.instantiate_global_script(agent, &realm_record, unit.instantiation_plan())
        .map_err(|e| format!("global instantiation failed for {}: {e:?}", workload.name))?;

    // Reset counters (also resets current_opcode to the idle sentinel) BEFORE
    // starting the sampler so no stale opcode is attributed.
    vm.opcode_counters_mut().reset();

    let histogram;
    let value;
    {
        // Borrow the cell through the counters; the boxed DispatchCounters
        // address is stable for the VM's lifetime. Take a raw `*const` so no
        // live `&` to `vm` is held across `evaluate_installed` below.
        let cell_ptr: *const std::sync::atomic::AtomicU64 =
            vm.opcode_counters().dispatch_banks().current_opcode_cell();
        // SAFETY: the cell lives as long as `vm`, and the profiler is stopped
        // (its thread joined via `stop()`) before `vm` is dropped at the end of
        // this scope, so the sampler's raw pointer never outlives the cell. The
        // `&*cell_ptr` deref is valid because `cell_ptr` came from a live `&`
        // to the boxed, stable-address counters allocation that `vm` owns.
        let profiler = unsafe {
            let cell: &std::sync::atomic::AtomicU64 = &*cell_ptr;
            SamplingProfiler::start(cell, interval)
        };

        value = vm
            .evaluate_installed(
                agent,
                installed,
                realm_record.global_env(),
                realm_record.global_env(),
            )
            .run()
            .map_err(|e| format!("execution failed for {}: {e:?}", workload.name))?;

        histogram = profiler.stop();
    }
    black_box(value.bits());

    let dispatch = vm.opcode_counters().dispatch_counts();
    Ok((dispatch, histogram))
}

fn pct(part: u64, whole: u64) -> f64 {
    if whole == 0 {
        0.0
    } else {
        (part as f64 / whole as f64) * 100.0
    }
}

fn samples_per_mdispatch(samples: u64, dispatches: u64) -> f64 {
    if dispatches == 0 {
        0.0
    } else {
        samples as f64 / (dispatches as f64 / 1_000_000.0)
    }
}

/// Rows sorted by descending total samples, opcodes with >=1 sample only.
fn sorted_rows(profile: &WorkloadProfile) -> Vec<(Opcode, u64, u64, u64)> {
    // (opcode, samples, slow_samples, dispatches)
    let mut rows: Vec<(Opcode, u64, u64, u64)> = profile
        .histogram
        .iter()
        .map(|(op, fast, slow)| (op, fast + slow, slow, profile.dispatch.count(op)))
        .collect();
    rows.sort_unstable_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.name().cmp(b.0.name())));
    rows
}

fn render_markdown(options: &Options, profiles: &[WorkloadProfile]) -> String {
    use std::fmt::Write as _;
    let mut out = String::new();
    let _ = writeln!(out, "# Lyng JS Time-Attribution Profile\n");
    let _ = writeln!(
        out,
        "Generated by `cargo run --release -p lyng-bench -- profile`. A background \
         sampler reads the live opcode every `{}us` and bins samples by \
         (opcode x fast/slow path). This is a STATISTICAL view: small-share rows \
         are noise-dominated; judge confidence against total sample count.\n",
        options.interval_us
    );
    let _ = writeln!(out, "- Samples summed per workload: `{}`", options.samples);
    let _ = writeln!(out, "- Sampler interval: `{}us`\n", options.interval_us);

    for profile in profiles {
        let total = profile.histogram.total();
        let total_dispatch = profile.dispatch.total();
        let _ = writeln!(out, "## {}\n", profile.workload.name);
        let _ = writeln!(
            out,
            "Total samples: `{total}` | Total dispatches: `{total_dispatch}` | \
             Non-opcode samples: `{}` ({:.2}%)\n",
            profile.histogram.non_opcode(),
            pct(profile.histogram.non_opcode(), total)
        );
        let _ = writeln!(
            out,
            "| Opcode | Time share | Slow share (of its time) | Dispatches | Samples / Mdispatch |"
        );
        let _ = writeln!(out, "| --- | ---: | ---: | ---: | ---: |");
        for (op, samples, slow, dispatches) in sorted_rows(profile) {
            let _ = writeln!(
                out,
                "| `{}` | {:.2}% | {:.2}% | {} | {:.2} |",
                op.name(),
                pct(samples, total),
                pct(slow, samples),
                dispatches,
                samples_per_mdispatch(samples, dispatches),
            );
        }
        let _ = writeln!(out);
    }
    out
}

fn render_json(options: &Options, profiles: &[WorkloadProfile]) -> Value {
    let workloads: Vec<Value> = profiles
        .iter()
        .map(|profile| {
            let total = profile.histogram.total();
            let rows: Vec<Value> = sorted_rows(profile)
                .into_iter()
                .map(|(op, samples, slow, dispatches)| {
                    json!({
                        "opcode": op.name(),
                        "samples": samples,
                        "slow_samples": slow,
                        "time_share_pct": pct(samples, total),
                        "slow_share_pct": pct(slow, samples),
                        "dispatches": dispatches,
                        "samples_per_mdispatch": samples_per_mdispatch(samples, dispatches),
                    })
                })
                .collect();
            json!({
                "name": profile.workload.name,
                "total_samples": total,
                "total_dispatches": profile.dispatch.total(),
                "non_opcode_samples": profile.histogram.non_opcode(),
                "rows": rows,
            })
        })
        .collect();
    json!({
        "schema": "lyng-bench/profile/v1",
        "interval_us": options.interval_us,
        "samples": options.samples,
        "workloads": workloads,
    })
}

fn print_summary(profiles: &[WorkloadProfile], options: &Options) {
    println!(
        "profile: {} workload(s), {} sample-run(s) each @ {}us interval -> {}",
        profiles.len(),
        options.samples,
        options.interval_us,
        options.report_path
    );
    for profile in profiles {
        if let Some((op, samples, _slow, _d)) = sorted_rows(profile).into_iter().next() {
            println!(
                "  {}: top opcode `{}` at {:.1}% of time",
                profile.workload.name,
                op.name(),
                pct(samples, profile.histogram.total())
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|p| (*p).to_string()).collect()
    }

    #[test]
    fn defaults_are_applied_when_no_args() {
        let options = parse_options(&[]).unwrap();
        assert_eq!(options.samples, 1);
        assert_eq!(options.interval_us, 200);
        assert!(!options.samply);
    }

    #[test]
    fn parses_filter_interval_and_samply() {
        let options = parse_options(&args(&[
            "--filter",
            "RayTrace",
            "--interval-us",
            "100",
            "--samply",
        ]))
        .unwrap();
        assert_eq!(options.filter.as_deref(), Some("RayTrace"));
        assert_eq!(options.interval_us, 100);
        assert!(options.samply);
    }

    #[test]
    fn zero_samples_is_rejected() {
        assert!(parse_options(&args(&["--samples", "0"])).is_err());
    }

    #[test]
    fn samply_capture_reports_error_gracefully() {
        // Force a deterministic failure regardless of whether `samply` is
        // installed: a bogus lyng-bin makes samply exit nonzero, and if samply
        // is absent we get a NotFound error. Either way: Err, never a panic.
        let options = Options {
            samply: true,
            lyng_bin: "/nonexistent/lyng-binary-xyz".to_string(),
            ..Options::default()
        };
        let workload = V8Workload::new("Probe", "n/a");
        let result = super::samply::capture(&workload, "1;", &options);
        assert!(result.is_err());
    }

    #[test]
    #[ignore = "slow: 2M-iter loop; run with: cargo test --release -p lyng-bench -- --ignored"]
    fn hot_opcode_dominates_the_profile() {
        // A tight arithmetic loop is dominated by a small set of opcodes; the
        // top profiled opcode should also be among the top dispatched opcodes.
        // Statistical, so assert a weak invariant: the #1 time-share opcode has
        // a nonzero dispatch count and the histogram captured real samples.
        let mut atoms = lyng_common::AtomTable::new();
        let source_id = lyng_common::SourceId::new(1);
        let src = "var s = 0; for (var i = 0; i < 2000000; i++) { s = s + i; } s;";
        let parsed = lyng_parser::parse_script(&mut atoms, source_id, src);
        assert!(!parsed.diagnostics.has_errors());
        let sema = lyng_sema::analyze_script(&parsed, &atoms);
        assert!(!sema.diagnostics.has_errors());
        let unit = lyng_compiler::compile_script(&parsed, &sema, &mut atoms).unwrap();
        let workload = V8Workload::new("LoopMicro", "n/a");
        let (dispatch, hist) = profile_once(&workload, &unit, Duration::from_micros(50)).unwrap();
        assert!(
            hist.total() > 0,
            "sampler should capture samples on a 2M-iter loop"
        );
        let profile = WorkloadProfile {
            workload,
            dispatch,
            histogram: hist,
        };
        let top = sorted_rows(&profile)
            .into_iter()
            .next()
            .expect("at least one opcode");
        assert!(
            top.3 > 0,
            "top time-share opcode `{}` should have dispatches",
            top.0.name()
        );
    }
}
