//! V8 v7 benchmark suite driver (lyng-5xdt).
//!
//! Wires the six V8 v7 workloads — Richards, DeltaBlue, Crypto, RayTrace,
//! NavierStokes, Splay — into `lyng-js-bench` as a `v8suite` subcommand.
//! Each benchmark is executed inside `target/release/lyng-js --shell` as
//! an isolated subprocess per sample so warmup state, JIT tier transitions,
//! GC heaps, and feedback caches don't leak between samples. The driver
//! collects `--samples` runs per benchmark (default 5), computes the
//! per-benchmark median score, and emits a markdown report alongside a
//! stable-keyed JSON document for downstream gate comparisons (Phase 1
//! sub-9 / `lyng-2wji`).
//!
//! The score model is V8's standard reciprocal-time formula:
//! `score = 100 × reference_µs / mean_µs`, where the reference comes
//! from each benchmark's `BenchmarkSuite` declaration in `base.js`-paired
//! files. Higher score is better. Each benchmark's `NotifyResult(name,
//! formatted)` callback prints `SCORE\t<name>\t<value>` lines to stdout
//! that the parent driver parses.

use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

pub const DEFAULT_REPORT_PATH: &str = "reports/js/lyng-js/bench-v8.md";
pub const DEFAULT_JSON_PATH: &str = "reports/js/lyng-js/bench-v8.json";
const DEFAULT_SAMPLES: usize = 5;
const DEFAULT_PER_SAMPLE_TIMEOUT_SECS: u64 = 120;
const DEFAULT_LYNG_BIN: &str = "target/release/lyng-js";

/// Static catalog of V8 v7 workloads. The reference µs/iteration must match
/// what each benchmark file declares to `new BenchmarkSuite(name, reference,
/// ...)` — keeping it here mirrors what the JS sees, so the scores we report
/// match what running V8's d8 on the same files would report.
///
/// `phase1_baseline` / `phase1_target` come from the JSC-aligned engine
/// roadmap (Phase 1 benchmark gates table in
/// `reports/js/lyng-js/jsc-aligned-engine-roadmap.md`). The baseline is the
/// pre-Phase-1 score on the legacy match dispatcher; the target is the
/// Phase 1 exit-gate score that the trampoline + per-handler ABI was
/// expected to achieve.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct V8Workload {
    pub name: &'static str,
    pub file: &'static str,
    pub reference_us: u64,
    pub phase1_baseline: u32,
    pub phase1_target: u32,
}

pub(crate) const V8_WORKLOADS: &[V8Workload] = &[
    V8Workload {
        name: "Richards",
        file: "richards.js",
        reference_us: 35_302,
        phase1_baseline: 234,
        phase1_target: 260,
    },
    V8Workload {
        name: "DeltaBlue",
        file: "deltablue.js",
        reference_us: 66_118,
        phase1_baseline: 277,
        phase1_target: 310,
    },
    V8Workload {
        name: "Crypto",
        file: "crypto.js",
        reference_us: 266_181,
        phase1_baseline: 236,
        phase1_target: 265,
    },
    V8Workload {
        name: "RayTrace",
        file: "raytrace.js",
        reference_us: 739_989,
        phase1_baseline: 387,
        phase1_target: 430,
    },
    V8Workload {
        name: "NavierStokes",
        file: "navier-stokes.js",
        reference_us: 1_484_000,
        phase1_baseline: 424,
        phase1_target: 470,
    },
    V8Workload {
        name: "Splay",
        file: "splay.js",
        reference_us: 81_491,
        phase1_baseline: 1198,
        phase1_target: 1330,
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
}

/// Runs the v8suite benchmark and writes Markdown + JSON reports.
///
/// # Errors
/// Returns an error when CLI parsing fails, the lyng-js binary is missing,
/// or a benchmark times out / fails on every sample.
pub fn run(args: &[String]) -> Result<(), String> {
    let options = parse_options(args)?;

    if cfg!(debug_assertions) {
        eprintln!("warning: build with --release for meaningful measurements");
    }

    ensure_path_exists(&options.lyng_bin, "lyng-js binary")?;
    ensure_path_exists(&options.v8_root, "v8 benchmark root")?;
    let base_js = read_file(&Path::new(&options.v8_root).join("base.js"))?;

    let workloads: Vec<&V8Workload> = V8_WORKLOADS
        .iter()
        .filter(|w| match &options.filter {
            Some(needle) => w.name.eq_ignore_ascii_case(needle),
            None => true,
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

fn parse_options(args: &[String]) -> Result<Options, String> {
    let mut options = Options {
        samples: DEFAULT_SAMPLES,
        report_path: DEFAULT_REPORT_PATH.to_string(),
        json_path: DEFAULT_JSON_PATH.to_string(),
        lyng_bin: DEFAULT_LYNG_BIN.to_string(),
        v8_root: default_v8_root(),
        per_sample_timeout: Duration::from_secs(DEFAULT_PER_SAMPLE_TIMEOUT_SECS),
        filter: None,
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
            other => {
                return Err(format!("unknown v8suite argument: {other}\n\n{}", help_text()));
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
        "Usage: lyng-js-bench v8suite [options]",
        "",
        "Runs the V8 v7 benchmark suite (Richards, DeltaBlue, Crypto, RayTrace,",
        "NavierStokes, Splay) inside the lyng-js shell, one isolated subprocess",
        "per sample. Emits per-benchmark median scores (V8 standard formula:",
        "100 × reference_µs / mean_µs).",
        "",
        "Options:",
        "  --samples N         Samples per benchmark (default: 5).",
        "  --report PATH       Markdown report path",
        "                      (default: reports/js/lyng-js/bench-v8.md).",
        "  --json PATH         JSON report path",
        "                      (default: reports/js/lyng-js/bench-v8.json).",
        "  --lyng-bin PATH     Path to the lyng-js executable",
        "                      (default: target/release/lyng-js).",
        "  --v8-root DIR       Directory containing the V8 v7 .js sources",
        "                      (default: testdata/js-benchmarks/v8-v7).",
        "  --timeout-secs N    Per-sample timeout in seconds (default: 120).",
        "  --filter NAME       Run only the named benchmark.",
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

/// Build the JS source we feed to `lyng-js --shell`. base.js installs the
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
        "lyng-js-bench-v8-{}-{}.js",
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
                    .map_err(|error| format!("failed to collect lyng-js output: {error}"))?;
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
            Err(error) => return Err(format!("lyng-js wait failed: {error}")),
        }
    }
}

fn parse_sample_output(workload: &V8Workload, output: &Output) -> Result<f64, String> {
    if !output.status.success() {
        return Err(format!(
            "lyng-js exit status {status}\nstdout:\n{stdout}\nstderr:\n{stderr}",
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
                    format!("could not parse {workload} score {score_str:?}: {error}",
                        workload = workload.name)
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
    Some(if sorted.len() % 2 == 0 {
        (sorted[mid - 1] + sorted[mid]) / 2.0
    } else {
        sorted[mid]
    })
}

fn render_markdown(options: &Options, reports: &[WorkloadReport]) -> String {
    let mut out = String::new();
    out.push_str("# Lyng JS V8 v7 Benchmark Report\n\n");
    out.push_str(
        "This report is generated by `cargo run --release -p lyng-js-bench -- v8suite`.\n\n",
    );
    out.push_str("Each benchmark runs in an isolated `lyng-js --shell` subprocess per sample so\n");
    out.push_str("warmup, GC, feedback caches, and tier transitions don't leak between samples.\n");
    out.push_str("Score = `100 × reference_µs / mean_µs` (V8 standard formula); higher is better.\n\n");
    out.push_str("## Configuration\n\n");
    out.push_str(&format!("- Samples per benchmark: `{}`\n", options.samples));
    out.push_str(&format!("- Per-sample timeout: `{}s`\n", options.per_sample_timeout.as_secs()));
    out.push_str(&format!("- lyng-js binary: `{}`\n", options.lyng_bin));
    out.push_str(&format!("- V8 v7 sources: `{}`\n\n", options.v8_root));
    out.push_str("## Scores\n\n");
    out.push_str("| Benchmark | Median score | Baseline | Target | Δ vs baseline | Gate | Median µs/iter | Samples |\n");
    out.push_str("| --- | ---: | ---: | ---: | ---: | :---: | ---: | --- |\n");
    for report in reports {
        let score_cell = report
            .median_score
            .map_or_else(|| "—".to_string(), |s| format!("`{s:.0}`"));
        let baseline = report.workload.phase1_baseline;
        let target = report.workload.phase1_target;
        let (delta_cell, gate_cell) = match report.median_score {
            Some(score) => {
                let delta = ((score - f64::from(baseline)) / f64::from(baseline)) * 100.0;
                let delta_str = format!("`{delta:+.1}%`");
                let gate_str = if score >= f64::from(target) {
                    "✓"
                } else {
                    "✗"
                };
                (delta_str, gate_str.to_string())
            }
            None => ("—".to_string(), "—".to_string()),
        };
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
        out.push_str(&format!(
            "| `{name}` | {score} | `{baseline}` | `{target}` | {delta} | {gate} | {us} | {samples} |\n",
            name = report.workload.name,
            score = score_cell,
            baseline = baseline,
            target = target,
            delta = delta_cell,
            gate = gate_cell,
            us = us_cell,
            samples = samples_cell,
        ));
    }
    out.push_str(
        "\nBaseline / target columns come from the Phase 1 exit-gate table in \
         [jsc-aligned-engine-roadmap.md](jsc-aligned-engine-roadmap.md). Baseline = \
         pre-Phase-1 score on the legacy match dispatcher; target = Phase 1 \
         trampoline-cutover score gate (sub-9, `lyng-2wji`). `Δ vs baseline` is \
         `(score − baseline) / baseline × 100%`; negative values are regressions.\n",
    );
    let any_error = reports.iter().any(|r| r.error.is_some());
    if any_error {
        out.push_str("\n## Errors\n\n");
        for report in reports {
            if let Some(error) = &report.error {
                out.push_str(&format!("- `{name}`: {error}\n", name = report.workload.name));
            }
        }
    }
    out
}

fn render_json(options: &Options, reports: &[WorkloadReport]) -> Value {
    let benchmarks: Vec<Value> = reports
        .iter()
        .map(|r| {
            let gate_met = r
                .median_score
                .map(|score| score >= f64::from(r.workload.phase1_target));
            json!({
                "name": r.workload.name,
                "file": r.workload.file,
                "reference_us": r.workload.reference_us,
                "phase1_baseline": r.workload.phase1_baseline,
                "phase1_target": r.workload.phase1_target,
                "samples": r.samples,
                "median_score": r.median_score,
                "median_us_per_iter": r.median_us_per_iter,
                "phase1_gate_met": gate_met,
                "error": r.error,
            })
        })
        .collect();
    json!({
        "schema": "lyng-js-bench/v8suite/v1",
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
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "failed to create directory for {path}: {error}",
                path = path
            )
        })?;
    }
    fs::write(path, contents).map_err(|error| format!("failed to write {path}: {error}"))
}

fn print_summary(reports: &[WorkloadReport]) {
    println!("\n========== V8 v7 Suite ==========");
    for report in reports {
        match (report.median_score, &report.error) {
            (Some(score), None) => {
                let target = report.workload.phase1_target;
                let baseline = report.workload.phase1_baseline;
                let delta = ((score - f64::from(baseline)) / f64::from(baseline)) * 100.0;
                let gate = if score >= f64::from(target) { "✓" } else { "✗" };
                println!(
                    "{name:<14} score={score:>5.0} baseline={baseline:>4} target={target:>4} \
                     Δ={delta:+5.1}% gate={gate}",
                    name = report.workload.name,
                );
            }
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
    fn v8_workloads_cover_phase_1_suite() {
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
