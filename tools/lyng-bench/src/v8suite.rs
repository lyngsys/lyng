//! V8 v7 benchmark suite driver (lyng-5xdt).
//!
//! Wires the six V8 v7 workloads — Richards, `DeltaBlue`, Crypto, `RayTrace`,
//! `NavierStokes`, Splay — into `lyng-bench` as a `v8suite` subcommand.
//! Each benchmark is executed inside `target/release/lyng --shell` as
//! an isolated subprocess per sample so warmup state, JIT tier transitions,
//! GC heaps, and feedback caches don't leak between samples. The driver
//! collects `--samples` runs per benchmark (default 5), computes the
//! per-benchmark median score, and emits a markdown report alongside a
//! stable-keyed JSON document for tracking progress toward JSC LLInt parity.
//!
//! The score model is V8's standard reciprocal-time formula:
//! `score = 100 × reference_µs / mean_µs`, where the reference comes
//! from each benchmark's `BenchmarkSuite` declaration in `base.js`-paired
//! files. Higher score is better. Each benchmark's `NotifyResult(name,
//! formatted)` callback prints `SCORE\t<name>\t<value>` lines to stdout
//! that the parent driver parses.

#![allow(
    clippy::cast_precision_loss,
    reason = "benchmark score and counter reports intentionally convert integer timings/counters to f64 ratios"
)]

use std::fmt::Write as _;

use lyng_builtins::BootstrapMode;
use lyng_bytecode::{Opcode, OPCODE_COUNT};
use lyng_common::{AtomTable, SourceId};
use lyng_compiler::compile_script;
use lyng_env::Runtime;
use lyng_host::NoopHostHooks;
use lyng_parser::parse_script;
use lyng_sema::analyze_script;
use lyng_vm::{OpcodeDispatchCounts, SlowPathCounts, Vm};
use serde_json::{json, Value};
use std::fs;
use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

pub const DEFAULT_REPORT_PATH: &str = "reports/lyng/bench-v8.md";
pub const DEFAULT_JSON_PATH: &str = "reports/lyng/bench-v8.json";
const DEFAULT_SAMPLES: usize = 5;
const DEFAULT_PER_SAMPLE_TIMEOUT_SECS: u64 = 120;
const DEFAULT_LYNG_BIN: &str = "target/release/lyng";

/// Static catalog of V8 v7 workloads. The reference µs/iteration must match
/// what each benchmark file declares to `new BenchmarkSuite(name, reference,
/// ...)` — keeping it here mirrors what the JS sees, so the scores we report
/// match what running V8's d8 on the same files would report.
///
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct V8Workload {
    pub name: &'static str,
    pub file: &'static str,
    pub reference_us: u64,
}

pub(crate) const V8_WORKLOADS: &[V8Workload] = &[
    V8Workload {
        name: "Richards",
        file: "richards.js",
        reference_us: 35_302,
    },
    V8Workload {
        name: "DeltaBlue",
        file: "deltablue.js",
        reference_us: 66_118,
    },
    V8Workload {
        name: "Crypto",
        file: "crypto.js",
        reference_us: 266_181,
    },
    V8Workload {
        name: "RayTrace",
        file: "raytrace.js",
        reference_us: 739_989,
    },
    V8Workload {
        name: "NavierStokes",
        file: "navier-stokes.js",
        reference_us: 1_484_000,
    },
    V8Workload {
        name: "Splay",
        file: "splay.js",
        reference_us: 81_491,
    },
];

#[derive(Debug)]
pub(crate) struct Options {
    pub samples: usize,
    pub report_path: String,
    pub json_path: String,
    pub lyng_bin: String,
    pub v8_root: String,
    pub per_sample_timeout: Duration,
    pub filter: Option<String>,
    pub count_opcodes: bool,
    pub count_slow_path_share: bool,
    pub counts_json_path: Option<String>,
}

/// Runs the v8suite benchmark and writes Markdown + JSON reports.
///
/// # Errors
/// Returns an error when CLI parsing fails, the lyng binary is missing,
/// or a benchmark times out / fails on every sample.
pub fn run(args: &[String]) -> Result<(), String> {
    let options = parse_options(args)?;

    if cfg!(debug_assertions) {
        eprintln!("warning: build with --release for meaningful measurements");
    }

    if options.count_opcodes {
        return run_opcode_counts(&options);
    }

    ensure_path_exists(&options.lyng_bin, "lyng binary")?;
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

    let mut reports = Vec::with_capacity(workloads.len());
    for workload in &workloads {
        let benchmark_path = Path::new(&options.v8_root).join(workload.file);
        let benchmark_js = read_file(&benchmark_path)?;
        let harness_source = build_harness(&base_js, &benchmark_js);
        let result = run_workload(workload, &harness_source, &options)?;
        reports.push(result);
    }

    let render = render_markdown(&options, &reports);
    let json = render_json(&options, &reports);
    write_output(&options.report_path, &render)?;
    write_output(
        &options.json_path,
        &serde_json::to_string_pretty(&json)
            .map_err(|error| format!("failed to render v8suite JSON report: {error}"))?,
    )?;
    print_summary(&reports);
    Ok(())
}

/// Per-workload sum of `SlowPathCounts` over every counted sample. Mirrors the
/// shape of `SlowPathCounts` but indexes by raw opcode discriminant so we can
/// accumulate without round-tripping through the per-`Vm` counter store.
#[derive(Clone)]
struct SlowPathCountTotals {
    semantic: Vec<u64>,
    safepoint: Vec<u64>,
}

impl SlowPathCountTotals {
    fn zeroed() -> Self {
        Self {
            semantic: vec![0; OPCODE_COUNT as usize],
            safepoint: vec![0; OPCODE_COUNT as usize],
        }
    }

    fn add_sample(&mut self, counts: &SlowPathCounts) {
        for raw in 0..OPCODE_COUNT {
            let Some(opcode) = Opcode::from_byte(raw) else {
                continue;
            };
            let index = usize::from(raw);
            self.semantic[index] = self.semantic[index].saturating_add(counts.semantic(opcode));
            self.safepoint[index] = self.safepoint[index].saturating_add(counts.safepoint(opcode));
        }
    }

    fn semantic(&self, opcode: Opcode) -> u64 {
        self.semantic
            .get(usize::from(opcode as u8))
            .copied()
            .unwrap_or(0)
    }

    fn safepoint(&self, opcode: Opcode) -> u64 {
        self.safepoint
            .get(usize::from(opcode as u8))
            .copied()
            .unwrap_or(0)
    }
}

/// In-process driver that runs each V8 v7 workload with opcode dispatch
/// counters enabled and emits per-workload per-opcode counts as JSON. Used by
/// the R-0 hot-opcode measurement (lyng-5obc) — the regular score path runs
/// each sample as a subprocess of `target/release/lyng`, which doesn't
/// carry the counter feature.
fn run_opcode_counts(options: &Options) -> Result<(), String> {
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

    let mut entries = Vec::with_capacity(workloads.len());
    for (index, workload) in workloads.iter().enumerate() {
        let benchmark_path = Path::new(&options.v8_root).join(workload.file);
        let benchmark_js = read_file(&benchmark_path)?;
        let harness_source = build_count_harness(&base_js, &benchmark_js);
        let source_id = SourceId::new(
            u32::try_from(index + 1)
                .map_err(|_| "v8suite workload count exceeds SourceId range".to_string())?,
        );
        let (counts, slow_path) = run_workload_opcode_counts(
            workload,
            &harness_source,
            source_id,
            options.samples,
            options.count_slow_path_share,
        )?;
        entries.push((**workload, counts, slow_path));
    }

    let json = render_opcode_counts_json(options, &entries);
    let counts_path = options
        .counts_json_path
        .clone()
        .unwrap_or_else(|| "reports/lyng/bench-v8-opcode-counts.json".to_string());
    write_output(
        &counts_path,
        &serde_json::to_string_pretty(&json)
            .map_err(|error| format!("failed to render v8suite opcode-counts JSON: {error}"))?,
    )?;

    print_opcode_counts_summary(&entries, &counts_path);
    Ok(())
}

fn parse_options(args: &[String]) -> Result<Options, String> {
    let mut options = Options {
        samples: DEFAULT_SAMPLES,
        report_path: DEFAULT_REPORT_PATH.to_string(),
        json_path: DEFAULT_JSON_PATH.to_string(),
        lyng_bin: DEFAULT_LYNG_BIN.to_string(),
        v8_root: default_v8_root(),
        per_sample_timeout: Duration::from_secs(DEFAULT_PER_SAMPLE_TIMEOUT_SECS),
        filter: None,
        count_opcodes: false,
        count_slow_path_share: false,
        counts_json_path: None,
    };

    let mut args = args.iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => {
                println!("{}", help_text());
                std::process::exit(0);
            }
            "--samples" => {
                options.samples = parse_usize_arg(&mut args, "--samples")?;
            }
            "--report" => {
                options.report_path = take_string_arg(&mut args, "--report")?;
            }
            "--json" => {
                options.json_path = take_string_arg(&mut args, "--json")?;
            }
            "--lyng-bin" => {
                options.lyng_bin = take_string_arg(&mut args, "--lyng-bin")?;
            }
            "--v8-root" => {
                options.v8_root = take_string_arg(&mut args, "--v8-root")?;
            }
            "--timeout-secs" => {
                let secs = parse_usize_arg(&mut args, "--timeout-secs")?;
                options.per_sample_timeout = Duration::from_secs(secs as u64);
            }
            "--filter" => {
                options.filter = Some(take_string_arg(&mut args, "--filter")?);
            }
            "--count-opcodes" | "--counter-opcodes" => {
                options.count_opcodes = true;
            }
            "--count-slow-path-share" => {
                options.count_slow_path_share = true;
            }
            "--counts-json" => {
                options.counts_json_path = Some(take_string_arg(&mut args, "--counts-json")?);
            }
            other => {
                return Err(format!(
                    "unknown v8suite argument: {other}\n\n{}",
                    help_text()
                ));
            }
        }
    }

    if options.samples == 0 {
        return Err("--samples must be ≥ 1".to_string());
    }
    Ok(options)
}

fn parse_usize_arg<'a>(
    args: &mut impl Iterator<Item = &'a String>,
    flag: &str,
) -> Result<usize, String> {
    let value = args
        .next()
        .ok_or_else(|| format!("{flag} requires a numeric argument"))?;
    value
        .parse::<usize>()
        .map_err(|error| format!("invalid {flag} value {value:?}: {error}"))
}

fn take_string_arg<'a>(
    args: &mut impl Iterator<Item = &'a String>,
    flag: &str,
) -> Result<String, String> {
    args.next()
        .cloned()
        .ok_or_else(|| format!("{flag} requires a value"))
}

fn default_v8_root() -> String {
    "testdata/js-benchmarks/v8-v7".to_string()
}

#[must_use]
pub fn help_text() -> String {
    [
        "Usage: lyng-bench v8suite [options]",
        "",
        "Runs the V8 v7 benchmark suite (Richards, DeltaBlue, Crypto, RayTrace,",
        "NavierStokes, Splay) inside the lyng shell, one isolated subprocess",
        "per sample. Emits per-benchmark median scores (V8 standard formula:",
        "100 × reference_µs / mean_µs).",
        "",
        "Options:",
        "  --samples N         Samples per benchmark (default: 5).",
        "  --report PATH       Markdown report path",
        "                      (default: reports/lyng/bench-v8.md).",
        "  --json PATH         JSON report path",
        "                      (default: reports/lyng/bench-v8.json).",
        "  --lyng-bin PATH     Path to the lyng executable",
        "                      (default: target/release/lyng).",
        "  --v8-root DIR       Directory containing the V8 v7 .js sources",
        "                      (default: testdata/js-benchmarks/v8-v7).",
        "  --timeout-secs N    Per-sample timeout in seconds (default: 120).",
        "  --filter NAME       Run only the named benchmark.",
        "  --count-opcodes     Run each workload in-process with opcode dispatch",
        "                      counters enabled, emitting per-workload per-opcode",
        "                      counts (skips the normal subprocess score run).",
        "  --count-slow-path-share",
        "                      Also collect per-opcode semantic and safepoint",
        "                      slow-path entries. Requires --count-opcodes; counts",
        "                      join into the same JSON output. All values are zero",
        "                      until DSL-0b adds record_semantic / record_safepoint",
        "                      call sites — the flag wires the infrastructure.",
        "  --counts-json PATH  Path for the opcode-counts JSON output when",
        "                      --count-opcodes is on (default:",
        "                      reports/lyng/bench-v8-opcode-counts.json).",
        "  -h, --help          Show this help.",
    ]
    .join("\n")
}

fn ensure_path_exists(path: &str, what: &str) -> Result<(), String> {
    if Path::new(path).exists() {
        Ok(())
    } else {
        Err(format!("{what} not found at {path}"))
    }
}

fn read_file(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|error| format!("failed to read {}: {error}", path.display()))
}

/// Build the JS source we feed to `lyng --shell`. base.js installs the
/// benchmark framework; the per-workload .js file registers a
/// `BenchmarkSuite`; the trailing script kicks off `RunSuites` and prints
/// `SCORE\t<name>\t<value>` for each suite via the `NotifyResult` callback.
fn build_harness(base_js: &str, benchmark_js: &str) -> String {
    let mut source = String::with_capacity(base_js.len() + benchmark_js.len() + 256);
    source.push_str(base_js);
    source.push('\n');
    source.push_str(benchmark_js);
    source.push_str(
        r#"
BenchmarkSuite.RunSuites({
  NotifyResult: function (name, score) {
    print("SCORE\t" + name + "\t" + score);
  },
  NotifyError: function (name, error) {
    print("ERROR\t" + name + "\t" + (error && error.message ? error.message : error));
  }
});
"#,
    );
    source
}

#[derive(Debug)]
struct WorkloadReport {
    workload: V8Workload,
    samples: Vec<f64>,
    median_score: Option<f64>,
    median_us_per_iter: Option<f64>,
    error: Option<String>,
}

fn run_workload(
    workload: &V8Workload,
    harness_source: &str,
    options: &Options,
) -> Result<WorkloadReport, String> {
    let harness_path = persist_harness(workload, harness_source)?;
    let mut samples = Vec::with_capacity(options.samples);
    let mut last_error: Option<String> = None;
    for index in 0..options.samples {
        match run_single_sample(workload, &harness_path, options) {
            Ok(score) => samples.push(score),
            Err(error) => {
                last_error = Some(format!("sample {idx} failed: {error}", idx = index + 1));
                break;
            }
        }
    }

    let median_score = median(&samples);
    let median_us_per_iter = median_score.map(|score| {
        // score = 100 × reference / mean_µs  →  mean_µs = 100 × reference / score
        (100.0 * workload.reference_us as f64) / score
    });

    Ok(WorkloadReport {
        workload: *workload,
        samples,
        median_score,
        median_us_per_iter,
        error: last_error,
    })
}

fn persist_harness(workload: &V8Workload, source: &str) -> Result<PathBuf, String> {
    let mut path = std::env::temp_dir();
    path.push(format!(
        "lyng-bench-v8-{}-{}.js",
        workload.name.to_ascii_lowercase(),
        std::process::id(),
    ));
    fs::write(&path, source).map_err(|error| {
        format!(
            "failed to write harness for {name} to {}: {error}",
            path.display(),
            name = workload.name
        )
    })?;
    Ok(path)
}

fn run_single_sample(
    workload: &V8Workload,
    harness_path: &Path,
    options: &Options,
) -> Result<f64, String> {
    let mut command = Command::new(&options.lyng_bin);
    command.arg("--shell");
    command.arg(harness_path);
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    let start = Instant::now();
    let mut child = command.spawn().map_err(|error| {
        format!(
            "failed to launch {bin}: {error}",
            bin = options.lyng_bin,
            error = error
        )
    })?;

    loop {
        match child.try_wait() {
            Ok(Some(_)) => {
                let output = child
                    .wait_with_output()
                    .map_err(|error| format!("failed to collect lyng output: {error}"))?;
                return parse_sample_output(workload, &output);
            }
            Ok(None) if start.elapsed() >= options.per_sample_timeout => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!(
                    "sample exceeded timeout of {}s",
                    options.per_sample_timeout.as_secs()
                ));
            }
            Ok(None) => {
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(error) => return Err(format!("lyng wait failed: {error}")),
        }
    }
}

fn parse_sample_output(workload: &V8Workload, output: &Output) -> Result<f64, String> {
    if !output.status.success() {
        return Err(format!(
            "lyng exit status {status}\nstdout:\n{stdout}\nstderr:\n{stderr}",
            status = output.status,
            stdout = String::from_utf8_lossy(&output.stdout),
            stderr = String::from_utf8_lossy(&output.stderr)
        ));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut value: Option<f64> = None;
    for line in stdout.lines() {
        if let Some(rest) = line.strip_prefix("SCORE\t") {
            let mut parts = rest.splitn(2, '\t');
            let name = parts.next().unwrap_or("");
            let score_str = parts.next().unwrap_or("");
            if name == workload.name {
                let parsed: f64 = score_str.trim().parse().map_err(|error| {
                    format!(
                        "could not parse {workload} score {score_str:?}: {error}",
                        workload = workload.name
                    )
                })?;
                value = Some(parsed);
            }
        } else if let Some(rest) = line.strip_prefix("ERROR\t") {
            return Err(format!("benchmark reported error: {rest}"));
        }
    }
    value.ok_or_else(|| {
        format!(
            "no SCORE line for {workload}; stdout was:\n{stdout}",
            workload = workload.name
        )
    })
}

fn median(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mid = sorted.len() / 2;
    Some(if sorted.len().is_multiple_of(2) {
        f64::midpoint(sorted[mid - 1], sorted[mid])
    } else {
        sorted[mid]
    })
}

fn render_markdown(options: &Options, reports: &[WorkloadReport]) -> String {
    let mut out = String::new();
    out.push_str("# Lyng JS V8 v7 Benchmark Report\n\n");
    out.push_str("This report is generated by `cargo run --release -p lyng-bench -- v8suite`.\n\n");
    out.push_str("Each benchmark runs in an isolated `lyng --shell` subprocess per sample so\n");
    out.push_str("warmup, GC, feedback caches, and tier transitions don't leak between samples.\n");
    out.push_str(
        "Score = `100 × reference_µs / mean_µs` (V8 standard formula); higher is better.\n\n",
    );
    out.push_str("## Configuration\n\n");
    writeln!(out, "- Samples per benchmark: `{}`", options.samples)
        .expect("writing to a String cannot fail");
    writeln!(
        out,
        "- Per-sample timeout: `{}s`",
        options.per_sample_timeout.as_secs()
    )
    .expect("writing to a String cannot fail");
    writeln!(out, "- lyng binary: `{}`", options.lyng_bin)
        .expect("writing to a String cannot fail");
    writeln!(out, "- V8 v7 sources: `{}`\n", options.v8_root)
        .expect("writing to a String cannot fail");
    out.push_str("## Scores\n\n");
    out.push_str("| Benchmark | Median score | Median µs/iter | Samples |\n");
    out.push_str("| --- | ---: | ---: | --- |\n");
    for report in reports {
        let score_cell = report
            .median_score
            .map_or_else(|| "—".to_string(), |s| format!("`{s:.0}`"));
        let us_cell = report
            .median_us_per_iter
            .map_or_else(|| "—".to_string(), |u| format!("`{u:.1}`"));
        let samples_cell = if report.samples.is_empty() {
            "—".to_string()
        } else {
            report
                .samples
                .iter()
                .map(|s| format!("{s:.0}"))
                .collect::<Vec<_>>()
                .join(", ")
        };
        writeln!(
            out,
            "| `{name}` | {score} | {us} | {samples} |",
            name = report.workload.name,
            score = score_cell,
            us = us_cell,
            samples = samples_cell,
        )
        .expect("writing to a String cannot fail");
    }
    let any_error = reports.iter().any(|r| r.error.is_some());
    if any_error {
        out.push_str("\n## Errors\n\n");
        for report in reports {
            if let Some(error) = &report.error {
                writeln!(out, "- `{name}`: {error}", name = report.workload.name)
                    .expect("writing to a String cannot fail");
            }
        }
    }
    out
}

fn render_json(options: &Options, reports: &[WorkloadReport]) -> Value {
    let benchmarks: Vec<Value> = reports
        .iter()
        .map(|r| {
            json!({
                "name": r.workload.name,
                "file": r.workload.file,
                "reference_us": r.workload.reference_us,
                "samples": r.samples,
                "median_score": r.median_score,
                "median_us_per_iter": r.median_us_per_iter,
                "error": r.error,
            })
        })
        .collect();
    json!({
        "schema": "lyng-bench/v8suite/v2",
        "samples_per_benchmark": options.samples,
        "per_sample_timeout_secs": options.per_sample_timeout.as_secs(),
        "lyng_bin": options.lyng_bin,
        "v8_root": options.v8_root,
        "benchmarks": benchmarks,
    })
}

fn write_output(path: &str, contents: &str) -> Result<(), String> {
    if let Some(parent) = Path::new(path).parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create directory for {path}: {error}"))?;
    }
    fs::write(path, contents).map_err(|error| format!("failed to write {path}: {error}"))
}

fn print_summary(reports: &[WorkloadReport]) {
    println!("\n========== V8 v7 Suite ==========");
    for report in reports {
        match (report.median_score, &report.error) {
            (Some(score), None) => match report.median_us_per_iter {
                Some(us) => println!(
                    "{name:<14} score={score:>5.0} median_us={us:>8.1}",
                    name = report.workload.name,
                ),
                None => println!("{name:<14} score={score:>5.0}", name = report.workload.name,),
            },
            (_, Some(error)) => {
                println!(
                    "{name:<14} ERROR: {error}",
                    name = report.workload.name,
                    error = error
                );
            }
            (None, None) => {
                println!("{name:<14} (no samples)", name = report.workload.name);
            }
        }
    }
}

/// Build the JS source we feed to an in-process VM run under
/// `--count-opcodes`. Same shape as `build_harness`, except `print` is
/// stubbed out (the lyng binary's CLI `print` isn't installed in-process,
/// and the score is irrelevant for opcode-count measurement). If a benchmark
/// reports a `NotifyError`, the harness re-throws so the driver surfaces it
/// instead of silently swallowing it under the counted suite run.
fn build_count_harness(base_js: &str, benchmark_js: &str) -> String {
    let mut source = String::with_capacity(base_js.len() + benchmark_js.len() + 256);
    source.push_str("var print = function () {};\n");
    source.push_str(base_js);
    source.push('\n');
    source.push_str(benchmark_js);
    source.push_str(
        r#"
var __lyng_v8_count_error = null;
BenchmarkSuite.RunSuites({
  NotifyResult: function (name, score) {},
  NotifyError: function (name, error) {
    if (__lyng_v8_count_error === null) {
      var detail;
      try {
        detail = (error && error.message) ? String(error.message) : String(error);
      } catch (e) {
        detail = "<unknown>";
      }
      __lyng_v8_count_error = name + ": " + detail;
    }
  }
});
if (__lyng_v8_count_error !== null) {
  throw new Error("v8suite count-opcodes benchmark error: " + __lyng_v8_count_error);
}
"#,
    );
    source
}

/// Run a single V8 v7 workload in-process with opcode dispatch counters
/// enabled. Uses a fresh `Runtime` + `Vm` per sample (mirroring the
/// subprocess-isolation discipline of the score path) so feedback caches,
/// shape state, and any tier transitions don't leak across samples.
///
/// When `collect_slow_path` is set the per-sample `SlowPathCounts` snapshots
/// are summed into a `SlowPathCountTotals` and returned alongside the
/// opcode-dispatch totals. The slow-path counters are zero today — DSL-0b is
/// the first task to call `record_semantic` / `record_safepoint`.
fn run_workload_opcode_counts(
    workload: &V8Workload,
    harness_source: &str,
    source_id: SourceId,
    samples: usize,
    collect_slow_path: bool,
) -> Result<(OpcodeDispatchCounts, Option<SlowPathCountTotals>), String> {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(&mut atoms, source_id, harness_source);
    if parsed.diagnostics.has_errors() {
        return Err(format!(
            "parse errors compiling {} for opcode counting: {:?}",
            workload.name,
            parsed.diagnostics.as_slice()
        ));
    }
    let sema = analyze_script(&parsed, &atoms);
    if sema.diagnostics.has_errors() {
        return Err(format!(
            "sema errors compiling {} for opcode counting: {:?}",
            workload.name,
            sema.diagnostics.as_slice()
        ));
    }
    let unit = compile_script(&parsed, &sema, &mut atoms).map_err(|error| {
        format!(
            "lowering failed for {} during opcode counting: {error:?}",
            workload.name
        )
    })?;

    let mut aggregate_pairs: Vec<(lyng_bytecode::Opcode, u64)> = Vec::new();
    let mut slow_path_totals = collect_slow_path.then(SlowPathCountTotals::zeroed);
    for sample_index in 0..samples {
        let (counts, slow_path_sample) =
            run_workload_opcode_counts_once(workload, &unit, collect_slow_path).map_err(
                |error| {
                    format!(
                        "sample {idx} failed for {name}: {error}",
                        idx = sample_index + 1,
                        name = workload.name
                    )
                },
            )?;
        for entry in counts.iter() {
            if entry.count() == 0 {
                continue;
            }
            aggregate_pairs.push((entry.opcode(), entry.count()));
        }
        if let (Some(totals), Some(sample)) = (slow_path_totals.as_mut(), slow_path_sample.as_ref())
        {
            totals.add_sample(sample);
        }
    }
    Ok((
        OpcodeDispatchCounts::from_counts(aggregate_pairs),
        slow_path_totals,
    ))
}

fn run_workload_opcode_counts_once(
    workload: &V8Workload,
    unit: &lyng_bytecode::CompiledScriptUnit,
    collect_slow_path: bool,
) -> Result<(OpcodeDispatchCounts, Option<SlowPathCounts>), String> {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent
        .default_realm()
        .ok_or_else(|| "default realm should exist for v8suite opcode counting".to_string())?;
    let realm_id = realm.id();
    let realm_record = realm;
    let mut vm = Vm::new();
    let _ = vm
        .bootstrap_realm(agent, realm_id, BootstrapMode::SpecOnly)
        .map_err(|error| format!("spec bootstrap failed: {error:?}"))?;
    let installed = vm.install_script(agent, realm_id, unit).map_err(|error| {
        format!(
            "script install failed for {name}: {error:?}",
            name = workload.name
        )
    })?;
    Vm::instantiate_global_script(agent, &realm_record, unit.instantiation_plan()).map_err(
        |error| {
            format!(
                "global declaration instantiation failed for {name}: {error:?}",
                name = workload.name
            )
        },
    )?;

    {
        let counters = vm.opcode_counters_mut();
        if collect_slow_path {
            counters.enable_slow_path();
        }
        counters.reset();
    }

    let value = vm
        .evaluate_installed(
            agent,
            installed,
            realm_record.global_env(),
            realm_record.global_env(),
        )
        .run()
        .map_err(|error| {
            format!(
                "execution failed for {name}: {error:?}",
                name = workload.name
            )
        })?;
    black_box(value.bits());

    let counters = vm.opcode_counters();
    let dispatch = counters.dispatch_counts();
    let slow_path = collect_slow_path.then(|| counters.slow_path_counts().unwrap_or_default());
    Ok((dispatch, slow_path))
}

fn render_opcode_counts_json(
    options: &Options,
    entries: &[(
        V8Workload,
        OpcodeDispatchCounts,
        Option<SlowPathCountTotals>,
    )],
) -> Value {
    let workloads: Vec<Value> = entries
        .iter()
        .map(|(workload, counts, slow_path)| {
            let mut by_opcode: Vec<(&'static str, u64)> = counts
                .iter()
                .filter(|entry| entry.count() != 0)
                .map(|entry| (entry.opcode().name(), entry.count()))
                .collect();
            by_opcode.sort_unstable_by(|left, right| {
                right.1.cmp(&left.1).then_with(|| left.0.cmp(right.0))
            });
            let opcode_counts: serde_json::Map<String, Value> = by_opcode
                .iter()
                .map(|(name, count)| ((*name).to_string(), json!(*count)))
                .collect();
            let slow_path_share = slow_path.as_ref().map_or(Value::Null, |totals| {
                slow_path_workload_json(counts, totals)
            });
            json!({
                "name": workload.name,
                "file": workload.file,
                "total_dispatches": counts.total(),
                "opcode_counts": Value::Object(opcode_counts),
                "slow_path_counts": slow_path_share,
            })
        })
        .collect();

    // Per-opcode totals aggregated across all workloads in the run. This is
    // what Task 5 (lyng-5obc / hot-opcodes.toml) consumes — the hot-list comes
    // from cumulative dispatch share across the whole V8 v7 suite, not from
    // any single workload.
    let mut totals_pairs: Vec<(lyng_bytecode::Opcode, u64)> = Vec::new();
    for (_, counts, _) in entries {
        for entry in counts.iter() {
            if entry.count() == 0 {
                continue;
            }
            totals_pairs.push((entry.opcode(), entry.count()));
        }
    }
    let aggregate = OpcodeDispatchCounts::from_counts(totals_pairs);
    let mut aggregate_pairs: Vec<(&'static str, u64)> = aggregate
        .iter()
        .filter(|entry| entry.count() != 0)
        .map(|entry| (entry.opcode().name(), entry.count()))
        .collect();
    aggregate_pairs
        .sort_unstable_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(right.0)));
    let totals_object: serde_json::Map<String, Value> = aggregate_pairs
        .iter()
        .map(|(name, count)| ((*name).to_string(), json!(*count)))
        .collect();

    // Aggregate slow-path totals across workloads when --count-slow-path-share
    // is on. Mirrors the per-opcode aggregate above so downstream consumers
    // can compute a single suite-wide semantic slow-path share.
    let slow_path_totals_payload = if entries.iter().any(|(_, _, slow)| slow.is_some()) {
        let mut aggregate_totals = SlowPathCountTotals::zeroed();
        for (_, _, slow) in entries {
            if let Some(totals) = slow.as_ref() {
                for raw in 0..OPCODE_COUNT {
                    let index = usize::from(raw);
                    aggregate_totals.semantic[index] =
                        aggregate_totals.semantic[index].saturating_add(totals.semantic[index]);
                    aggregate_totals.safepoint[index] =
                        aggregate_totals.safepoint[index].saturating_add(totals.safepoint[index]);
                }
            }
        }
        slow_path_workload_json(&aggregate, &aggregate_totals)
    } else {
        Value::Null
    };

    json!({
        "schema": "lyng-bench/v8suite/opcode-counts/v1",
        "samples_per_benchmark": options.samples,
        "v8_root": options.v8_root,
        "workloads": workloads,
        "totals": {
            "total_dispatches": aggregate.total(),
            "opcode_counts": Value::Object(totals_object),
            "slow_path_counts": slow_path_totals_payload,
        },
    })
}

fn slow_path_workload_json(
    dispatch: &OpcodeDispatchCounts,
    slow_path: &SlowPathCountTotals,
) -> Value {
    let mut entries: Vec<(Opcode, u64, u64, u64)> = dispatch
        .iter()
        .filter(|entry| entry.count() > 0)
        .map(|entry| {
            let opcode = entry.opcode();
            (
                opcode,
                entry.count(),
                slow_path.semantic(opcode),
                slow_path.safepoint(opcode),
            )
        })
        .collect();
    entries.sort_unstable_by(|left, right| {
        right
            .1
            .cmp(&left.1)
            .then_with(|| left.0.name().cmp(right.0.name()))
    });
    let mut total_semantic = 0_u64;
    let mut total_safepoint = 0_u64;
    let mut total_dispatches = 0_u64;
    let rows: Vec<Value> = entries
        .into_iter()
        .map(|(opcode, dispatches, semantic, safepoint)| {
            let share = if dispatches > 0 {
                (semantic as f64) / (dispatches as f64) * 100.0
            } else {
                0.0
            };
            total_semantic = total_semantic.saturating_add(semantic);
            total_safepoint = total_safepoint.saturating_add(safepoint);
            total_dispatches = total_dispatches.saturating_add(dispatches);
            json!({
                "opcode": opcode.name(),
                "dispatches": dispatches,
                "semantic_slow_path": semantic,
                "safepoint_slow_path": safepoint,
                "semantic_share_percent": share,
            })
        })
        .collect();
    json!({
        "total_dispatches": total_dispatches,
        "total_semantic_slow_path": total_semantic,
        "total_safepoint_slow_path": total_safepoint,
        "per_opcode": rows,
    })
}

fn print_opcode_counts_summary(
    entries: &[(
        V8Workload,
        OpcodeDispatchCounts,
        Option<SlowPathCountTotals>,
    )],
    output_path: &str,
) {
    println!("\n========== V8 v7 Opcode Counts ==========");
    println!("wrote {output_path}");
    for (workload, counts, _) in entries {
        println!(
            "{name:<14} total={total}",
            name = workload.name,
            total = counts.total()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn median_handles_empty() {
        assert_eq!(median(&[]), None);
    }

    #[test]
    fn median_handles_odd_count() {
        assert_eq!(median(&[3.0, 1.0, 2.0]), Some(2.0));
    }

    #[test]
    fn median_handles_even_count() {
        assert_eq!(median(&[1.0, 2.0, 3.0, 4.0]), Some(2.5));
    }

    #[test]
    fn parse_sample_output_extracts_score_line() {
        let output = Output {
            status: std::process::Command::new("true")
                .status()
                .expect("true command should succeed"),
            stdout: b"SCORE\tRichards\t260.5\nSCORE\tOther\t999\n".to_vec(),
            stderr: Vec::new(),
        };
        let result = parse_sample_output(&V8_WORKLOADS[0], &output).expect("should parse");
        assert!((result - 260.5).abs() < 0.0001);
    }

    #[test]
    fn parse_sample_output_propagates_error_line() {
        let output = Output {
            status: std::process::Command::new("true")
                .status()
                .expect("true command should succeed"),
            stdout: b"ERROR\tRichards\tsomething broke\n".to_vec(),
            stderr: Vec::new(),
        };
        let error =
            parse_sample_output(&V8_WORKLOADS[0], &output).expect_err("should propagate error");
        assert!(error.contains("Richards"));
        assert!(error.contains("something broke"));
    }

    #[test]
    fn parse_sample_output_fails_when_score_missing() {
        let output = Output {
            status: std::process::Command::new("true")
                .status()
                .expect("true command should succeed"),
            stdout: b"some other output\n".to_vec(),
            stderr: Vec::new(),
        };
        let error =
            parse_sample_output(&V8_WORKLOADS[0], &output).expect_err("should fail without SCORE");
        assert!(error.contains("no SCORE line"));
    }

    #[test]
    fn build_harness_appends_runsuites_call() {
        let source = build_harness("var BASE = 1;", "var BENCH = 2;");
        assert!(source.contains("var BASE = 1;"));
        assert!(source.contains("var BENCH = 2;"));
        assert!(source.contains("BenchmarkSuite.RunSuites"));
        assert!(source.contains("NotifyResult"));
    }

    #[test]
    fn markdown_report_omits_stale_gate_columns() {
        let options = parse_options(&[]).expect("default v8suite options should parse");
        let reports = vec![WorkloadReport {
            workload: V8_WORKLOADS[0],
            samples: vec![463.0],
            median_score: Some(463.0),
            median_us_per_iter: Some(7624.6),
            error: None,
        }];

        let markdown = render_markdown(&options, &reports);

        assert!(markdown.contains("| Benchmark | Median score | Median µs/iter | Samples |"));
        assert!(!markdown.contains("Baseline"));
        assert!(!markdown.contains("Target"));
        assert!(!markdown.contains("Gate"));
        assert!(!markdown.contains(&format!("phase{}", 1)));
    }

    #[test]
    fn json_report_omits_stale_gate_fields() {
        let options = parse_options(&[]).expect("default v8suite options should parse");
        let reports = vec![WorkloadReport {
            workload: V8_WORKLOADS[0],
            samples: vec![463.0],
            median_score: Some(463.0),
            median_us_per_iter: Some(7624.6),
            error: None,
        }];

        let report = render_json(&options, &reports);
        let benchmark = &report["benchmarks"][0];

        assert_eq!(report["schema"], "lyng-bench/v8suite/v2");
        assert!(benchmark.get("reference_us").is_some());
        assert!(benchmark.get("samples").is_some());
        assert!(benchmark.get("median_score").is_some());
        assert!(benchmark.get("median_us_per_iter").is_some());
        for suffix in ["baseline", "target", "gate_met"] {
            let field = format!("phase{}_{suffix}", 1);
            assert!(benchmark.get(&field).is_none(), "{field} should be absent");
        }
    }

    #[test]
    fn v8_workloads_cover_local_v8_v7_suite() {
        let names: Vec<&str> = V8_WORKLOADS.iter().map(|w| w.name).collect();
        assert_eq!(
            names,
            vec![
                "Richards",
                "DeltaBlue",
                "Crypto",
                "RayTrace",
                "NavierStokes",
                "Splay"
            ]
        );
    }

    #[test]
    fn parse_options_accepts_samples_and_filter() {
        let args = vec![
            "--samples".to_string(),
            "3".to_string(),
            "--filter".to_string(),
            "Richards".to_string(),
        ];
        let options = parse_options(&args).expect("should parse");
        assert_eq!(options.samples, 3);
        assert_eq!(options.filter, Some("Richards".to_string()));
    }

    #[test]
    fn parse_options_rejects_zero_samples() {
        let args = vec!["--samples".to_string(), "0".to_string()];
        assert!(parse_options(&args).is_err());
    }
}
