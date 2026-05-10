mod v8_v7;

use serde_json::{json, Value};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

pub const DEFAULT_REPORT_PATH: &str = "reports/js/lyng-js/external-engine-compare.md";
pub const DEFAULT_JSON_PATH: &str = "reports/js/lyng-js/external-engine-compare.json";
const DEFAULT_SAMPLES: usize = 3;
const DEFAULT_WARMUP_SAMPLES: usize = 1;
const DEFAULT_LOOP_TRIPS: usize = 2_048;
const ARITHMETIC_NOTE: &str =
    "Integer arithmetic, branches, and loop backedges without builtin calls.";
const ARRAY_OBJECT_NOTE: &str =
    "Array growth, dense indexed reads, object literals, and named property reads.";
const BUILTIN_NOTE: &str =
    "String case mapping, RegExp replacement, URI decoding, and character access.";

type CompareResult<T> = Result<T, String>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Corpus {
    Synthetic,
    V8V7,
}

impl Corpus {
    const fn label(self) -> &'static str {
        match self {
            Self::Synthetic => "synthetic",
            Self::V8V7 => "v8-v7",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MetricKind {
    WallTime,
    Score,
}

impl MetricKind {
    const fn label(self) -> &'static str {
        match self {
            Self::WallTime => "wall-time",
            Self::Score => "score",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct EngineConfig {
    name: &'static str,
    executable: String,
    pre_args: Vec<String>,
}

impl EngineConfig {
    fn new<const N: usize>(
        name: &'static str,
        executable: impl Into<String>,
        pre_args: [&str; N],
    ) -> Self {
        Self {
            name,
            executable: executable.into(),
            pre_args: pre_args
                .into_iter()
                .map(std::string::ToString::to_string)
                .collect(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Options {
    report_path: String,
    json_path: String,
    samples: usize,
    warmup_samples: usize,
    loop_trip_count: usize,
    scripts_dir: PathBuf,
    engines: Vec<EngineConfig>,
    corpus: Corpus,
    filter: Option<String>,
    full_suite: bool,
}

#[derive(Clone)]
struct Workload {
    name: &'static str,
    category: &'static str,
    file_name: &'static str,
    source: String,
    metric_kind: MetricKind,
    requires_lyng_shell: bool,
}

#[derive(Clone)]
struct EngineWorkloadReport {
    workload_name: String,
    workload_category: String,
    script_path: PathBuf,
    engine_name: String,
    command: Vec<String>,
    samples: Vec<Duration>,
    median: Duration,
    min: Duration,
    max: Duration,
    score_samples: Vec<f64>,
    median_score: Option<f64>,
    quickjs_ratio: Option<f64>,
    metric_kind: MetricKind,
}

/// Runs the external engine comparison suite and writes Markdown plus JSON reports.
///
/// # Errors
///
/// Returns an error when arguments are invalid, scripts cannot be written, an external
/// engine cannot be launched, or a report cannot be written.
pub fn run(args: &[String]) -> CompareResult<()> {
    let options = parse_options(args)?;

    if cfg!(debug_assertions) {
        eprintln!("warning: build with --release for meaningful measurements");
    }

    fs::create_dir_all(&options.scripts_dir).map_err(|error| {
        format!(
            "failed to create compare scripts dir `{}`: {error}",
            options.scripts_dir.display()
        )
    })?;

    let workloads = build_selected_workloads(&options)?;
    let script_paths = write_workload_scripts(&options.scripts_dir, &workloads)?;
    let mut reports = measure_workloads(&options, &workloads, &script_paths)?;
    attach_quickjs_ratios(&mut reports);

    let markdown = render_report(&options, &reports);
    let json = render_json_report(&options, &reports);
    write_report(&options.report_path, &markdown)?;
    write_report(
        &options.json_path,
        &serde_json::to_string_pretty(&json)
            .map_err(|error| format!("failed to render external compare JSON: {error}"))?,
    )?;
    print_summary(&options, &reports);
    Ok(())
}

fn parse_options(args: &[String]) -> CompareResult<Options> {
    let mut options = Options {
        report_path: DEFAULT_REPORT_PATH.to_string(),
        json_path: DEFAULT_JSON_PATH.to_string(),
        samples: DEFAULT_SAMPLES,
        warmup_samples: DEFAULT_WARMUP_SAMPLES,
        loop_trip_count: DEFAULT_LOOP_TRIPS,
        scripts_dir: default_scripts_dir(),
        engines: default_engines(),
        corpus: Corpus::Synthetic,
        filter: None,
        full_suite: false,
    };

    let mut args = args.iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--report" => {
                options.report_path = next_string("--report", args.next())?;
            }
            "--json" => {
                options.json_path = next_string("--json", args.next())?;
            }
            "--preset" => {
                apply_preset(
                    &mut options,
                    args.next()
                        .ok_or_else(|| "--preset requires a name".to_string())?,
                )?;
            }
            "--samples" => {
                options.samples = parse_usize_arg("--samples", args.next())?;
            }
            "--warmup-samples" => {
                options.warmup_samples = parse_usize_arg("--warmup-samples", args.next())?;
            }
            "--loop-trips" => {
                options.loop_trip_count = parse_usize_arg("--loop-trips", args.next())?;
            }
            "--scripts-dir" => {
                options.scripts_dir = PathBuf::from(next_string("--scripts-dir", args.next())?);
            }
            "--corpus" => {
                options.corpus = parse_corpus(
                    args.next()
                        .ok_or_else(|| "--corpus requires a name".to_string())?,
                )?;
            }
            "--filter" => {
                options.filter = Some(next_string("--filter", args.next())?);
            }
            "--full-suite" => {
                options.full_suite = true;
            }
            "--lyng-js" => {
                set_engine_executable(
                    &mut options,
                    "lyng-js",
                    next_string("--lyng-js", args.next())?,
                );
            }
            "--qjs" => {
                set_engine_executable(&mut options, "quickjs", next_string("--qjs", args.next())?);
            }
            "--boa" => {
                set_engine_executable(&mut options, "boa", next_string("--boa", args.next())?);
            }
            "--help" | "-h" => return Err(usage()),
            unknown => return Err(format!("Unknown argument: {unknown}")),
        }
    }

    if options.samples == 0 {
        return Err("--samples must be greater than zero".to_string());
    }
    if options.loop_trip_count == 0 {
        return Err("--loop-trips must be greater than zero".to_string());
    }
    if options.full_suite && options.filter.is_some() {
        return Err("--full-suite cannot be combined with --filter".to_string());
    }

    Ok(options)
}

fn parse_corpus(value: &str) -> CompareResult<Corpus> {
    match value {
        "synthetic" => Ok(Corpus::Synthetic),
        "v8-v7" => Ok(Corpus::V8V7),
        _ => Err(format!(
            "invalid --corpus value `{value}`; expected synthetic or v8-v7"
        )),
    }
}

fn next_string(flag: &str, value: Option<&String>) -> CompareResult<String> {
    value
        .cloned()
        .ok_or_else(|| format!("{flag} requires a value"))
}

fn parse_usize_arg(flag: &str, value: Option<&String>) -> CompareResult<usize> {
    value
        .ok_or_else(|| format!("{flag} requires a value"))?
        .parse()
        .map_err(|_| format!("{flag} expects a positive integer"))
}

fn apply_preset(options: &mut Options, preset: &str) -> CompareResult<()> {
    match preset {
        "smoke" => {
            options.samples = 1;
            options.warmup_samples = 0;
            options.loop_trip_count = 1_024;
        }
        "baseline" => {
            options.samples = DEFAULT_SAMPLES;
            options.warmup_samples = DEFAULT_WARMUP_SAMPLES;
            options.loop_trip_count = DEFAULT_LOOP_TRIPS;
        }
        "profile-target" => {
            options.samples = 1;
            options.warmup_samples = 0;
            options.loop_trip_count = 65_536;
        }
        _ => {
            return Err(format!(
                "invalid --preset value `{preset}`; expected smoke, baseline, or profile-target"
            ));
        }
    }
    Ok(())
}

fn usage() -> String {
    "Usage: lyng-js-bench compare [--corpus <synthetic|v8-v7>] [--filter <name>] [--full-suite] [--preset <smoke|baseline|profile-target>] [--report <path>] [--json <path>] [--samples <n>] [--warmup-samples <n>] [--loop-trips <n>] [--scripts-dir <path>] [--lyng-js <path>] [--qjs <path>] [--boa <path>]"
        .to_string()
}

fn set_engine_executable(options: &mut Options, name: &str, executable: String) {
    if let Some(engine) = options
        .engines
        .iter_mut()
        .find(|engine| engine.name == name)
    {
        engine.executable = executable;
    }
}

fn default_engines() -> Vec<EngineConfig> {
    vec![
        EngineConfig::new("lyng-js", "target/release/lyng-js", []),
        EngineConfig::new("quickjs", default_homebrew_or_path("qjs"), ["--script"]),
        EngineConfig::new("boa", default_homebrew_or_path("boa"), []),
    ]
}

fn default_homebrew_or_path(binary: &str) -> String {
    let homebrew_path = Path::new("/opt/homebrew/bin").join(binary);
    if homebrew_path.exists() {
        homebrew_path.display().to_string()
    } else {
        binary.to_string()
    }
}

fn default_scripts_dir() -> PathBuf {
    PathBuf::from("/tmp/lyng-js-bench-compare-scripts")
}

fn build_workloads(loop_trip_count: usize) -> Vec<Workload> {
    vec![
        Workload {
            name: "arithmetic-loop",
            category: "arithmetic-control-flow",
            file_name: "arithmetic-loop.js",
            source: arithmetic_workload(loop_trip_count),
            metric_kind: MetricKind::WallTime,
            requires_lyng_shell: false,
        },
        Workload {
            name: "array-object-loop",
            category: "array-object",
            file_name: "array-object-loop.js",
            source: array_object_workload(loop_trip_count),
            metric_kind: MetricKind::WallTime,
            requires_lyng_shell: false,
        },
        Workload {
            name: "builtin-string-regexp-loop",
            category: "builtin-heavy",
            file_name: "builtin-string-regexp-loop.js",
            source: builtin_workload(loop_trip_count),
            metric_kind: MetricKind::WallTime,
            requires_lyng_shell: false,
        },
    ]
}

fn build_selected_workloads(options: &Options) -> CompareResult<Vec<Workload>> {
    match options.corpus {
        Corpus::Synthetic => Ok(build_workloads(options.loop_trip_count)),
        Corpus::V8V7 => v8_v7::build_workloads(options.filter.as_deref(), options.full_suite),
    }
}

fn arithmetic_workload(loop_trip_count: usize) -> String {
    format!(
        r#"(function() {{
var __lyngBenchTrips = {loop_trip_count};
var __lyngBenchSink = 0;
for (var i = 0; i < __lyngBenchTrips; i = i + 1) {{
  var value = i * 13 + 7;
  if ((value & 3) === 0) {{
    __lyngBenchSink = __lyngBenchSink + value / 2;
  }} else {{
    __lyngBenchSink = __lyngBenchSink + value - (i % 5);
  }}
}}
if (__lyngBenchSink === -1) {{
  throw new Error("unreachable arithmetic sink");
}}
}})();
"#
    )
}

fn array_object_workload(loop_trip_count: usize) -> String {
    format!(
        r#"(function() {{
var __lyngBenchTrips = {loop_trip_count};
var __lyngBenchSink = 0;
var records = [];
for (var i = 0; i < __lyngBenchTrips; i = i + 1) {{
  records.push({{ index: i, left: i + 1, right: i * 3 }});
}}
for (var j = 0; j < records.length; j = j + 1) {{
  var record = records[j];
  __lyngBenchSink = __lyngBenchSink + record.left + record.right - record.index;
}}
if (__lyngBenchSink === -1) {{
  throw new Error("unreachable array object sink");
}}
}})();
"#
    )
}

fn builtin_workload(loop_trip_count: usize) -> String {
    format!(
        r#"(function() {{
var __lyngBenchTrips = {loop_trip_count};
var __lyngBenchSink = 0;
var source = "Alpha-123-beta-456";
var pattern = /([A-Za-z]+)-([0-9]+)-([a-z]+)-([0-9]+)/;
for (var i = 0; i < __lyngBenchTrips; i = i + 1) {{
  var decoded = decodeURIComponent("%E2%82%AC");
  var replaced = source.replace(pattern, "$3:$1:$2:$4");
  var folded = replaced.toUpperCase();
  __lyngBenchSink = __lyngBenchSink + decoded.charCodeAt(0) + folded.charCodeAt(i % folded.length);
}}
if (__lyngBenchSink === -1) {{
  throw new Error("unreachable builtin sink");
}}
}})();
"#
    )
}

fn write_workload_scripts(
    scripts_dir: &Path,
    workloads: &[Workload],
) -> CompareResult<Vec<PathBuf>> {
    workloads
        .iter()
        .map(|workload| {
            let path = scripts_dir.join(workload.file_name);
            fs::write(&path, &workload.source).map_err(|error| {
                format!(
                    "failed to write compare workload script `{}`: {error}",
                    path.display()
                )
            })?;
            Ok(path)
        })
        .collect()
}

fn measure_workloads(
    options: &Options,
    workloads: &[Workload],
    script_paths: &[PathBuf],
) -> CompareResult<Vec<EngineWorkloadReport>> {
    let mut reports = Vec::new();
    for (workload, script_path) in workloads.iter().zip(script_paths) {
        for engine in &options.engines {
            reports.push(measure_engine_workload(
                options,
                workload,
                script_path,
                engine,
            )?);
        }
    }
    Ok(reports)
}

fn measure_engine_workload(
    options: &Options,
    workload: &Workload,
    script_path: &Path,
    engine: &EngineConfig,
) -> CompareResult<EngineWorkloadReport> {
    for _ in 0..options.warmup_samples {
        let _ = run_engine_once(engine, workload, script_path)?;
    }

    let mut samples = Vec::with_capacity(options.samples);
    let mut score_samples = Vec::with_capacity(options.samples);
    for _ in 0..options.samples {
        let sample = run_engine_once(engine, workload, script_path)?;
        samples.push(sample.elapsed);
        if let Some(score) = sample.score {
            score_samples.push(score);
        }
    }

    let median = median_duration(samples.clone());
    let min = samples
        .iter()
        .copied()
        .min()
        .expect("sample count is validated as non-zero");
    let max = samples
        .iter()
        .copied()
        .max()
        .expect("sample count is validated as non-zero");
    let median_score = if score_samples.is_empty() {
        None
    } else {
        Some(median_f64(score_samples.clone()))
    };

    Ok(EngineWorkloadReport {
        workload_name: workload.name.to_string(),
        workload_category: workload.category.to_string(),
        script_path: script_path.to_path_buf(),
        engine_name: engine.name.to_string(),
        command: command_vector(engine, workload, script_path),
        samples,
        median,
        min,
        max,
        score_samples,
        median_score,
        quickjs_ratio: None,
        metric_kind: workload.metric_kind,
    })
}

struct EngineRunSample {
    elapsed: Duration,
    score: Option<f64>,
}

fn run_engine_once(
    engine: &EngineConfig,
    workload: &Workload,
    script_path: &Path,
) -> CompareResult<EngineRunSample> {
    let mut command = Command::new(&engine.executable);
    for arg in &engine.pre_args {
        command.arg(arg);
    }
    if engine.name == "lyng-js" && workload.requires_lyng_shell {
        command.arg("--shell");
    }
    command.arg(script_path);

    let start = Instant::now();
    let output = command.output().map_err(|error| {
        format!(
            "failed to launch external engine `{engine_name}` for workload `{workload_name}`: {error}",
            engine_name = engine.name,
            workload_name = workload.name
        )
    })?;
    let elapsed = start.elapsed();

    if !output.status.success() {
        return Err(format!(
            "external engine `{engine_name}` failed for workload `{workload_name}` with status {status}\nstdout:\n{stdout}\nstderr:\n{stderr}",
            engine_name = engine.name,
            workload_name = workload.name,
            status = output.status,
            stdout = String::from_utf8_lossy(&output.stdout),
            stderr = String::from_utf8_lossy(&output.stderr)
        ));
    }

    let score = match workload.metric_kind {
        MetricKind::WallTime => None,
        MetricKind::Score => Some(parse_score_output(&output.stdout, workload.name)?),
    };

    Ok(EngineRunSample { elapsed, score })
}

fn parse_score_output(stdout: &[u8], workload_name: &str) -> CompareResult<f64> {
    let text = String::from_utf8_lossy(stdout);
    let prefix = format!("{workload_name}: ");
    for line in text.lines() {
        if let Some(value) = line.strip_prefix(&prefix) {
            return value
                .trim()
                .replace(',', "")
                .parse::<f64>()
                .map_err(|error| {
                    format!(
                        "failed to parse score `{value}` for workload `{workload_name}`: {error}"
                    )
                });
        }
    }
    Err(format!(
        "external benchmark output did not contain a `{workload_name}: <score>` line\nstdout:\n{text}"
    ))
}

fn command_vector(engine: &EngineConfig, workload: &Workload, script_path: &Path) -> Vec<String> {
    let mut command = Vec::with_capacity(engine.pre_args.len() + 3);
    command.push(engine.executable.clone());
    command.extend(engine.pre_args.iter().cloned());
    if engine.name == "lyng-js" && workload.requires_lyng_shell {
        command.push("--shell".to_string());
    }
    command.push(script_path.display().to_string());
    command
}

fn attach_quickjs_ratios(reports: &mut [EngineWorkloadReport]) {
    let quickjs_reports = reports
        .iter()
        .filter(|report| report.engine_name == "quickjs")
        .map(|report| {
            (
                report.workload_name.clone(),
                report.median,
                report.median_score,
                report.metric_kind,
            )
        })
        .collect::<Vec<_>>();

    for report in reports {
        report.quickjs_ratio = quickjs_reports
            .iter()
            .find(|(workload_name, _, _, _)| workload_name == &report.workload_name)
            .and_then(|(_, quickjs_median, quickjs_score, quickjs_metric)| {
                if *quickjs_metric != report.metric_kind {
                    return None;
                }
                match report.metric_kind {
                    MetricKind::WallTime => {
                        let quickjs_ms = duration_ms(*quickjs_median);
                        if quickjs_ms > 0.0 {
                            Some(duration_ms(report.median) / quickjs_ms)
                        } else {
                            None
                        }
                    }
                    MetricKind::Score => {
                        let score = report.median_score?;
                        let quickjs_score = (*quickjs_score)?;
                        if score > 0.0 {
                            Some(quickjs_score / score)
                        } else {
                            None
                        }
                    }
                }
            });
    }
}

fn render_report(options: &Options, reports: &[EngineWorkloadReport]) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "# Lyng JS External Engine Comparison");
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "This report is generated by `cargo run --release -p lyng-js-bench -- compare --report {}`.",
        options.report_path
    );
    let _ = writeln!(
        out,
        "It runs the same standalone JavaScript workload files through Lyng JS, QuickJS, and Boa."
    );
    let _ = writeln!(out);
    let _ = writeln!(out, "## Settings");
    let _ = writeln!(out);
    let _ = writeln!(out, "- Report: `{}`", options.report_path);
    let _ = writeln!(out, "- JSON: `{}`", options.json_path);
    let _ = writeln!(out, "- Scripts dir: `{}`", options.scripts_dir.display());
    let _ = writeln!(out, "- Corpus: `{}`", options.corpus.label());
    if let Some(filter) = options.filter.as_ref() {
        let _ = writeln!(out, "- Filter: `{filter}`");
    }
    let _ = writeln!(out, "- Full suite: `{}`", options.full_suite);
    let _ = writeln!(out, "- Samples: `{}`", options.samples);
    let _ = writeln!(out, "- Warmup samples: `{}`", options.warmup_samples);
    let _ = writeln!(out, "- Loop trips: `{}`", options.loop_trip_count);
    let _ = writeln!(out);
    let _ = writeln!(out, "## Comparison Policy");
    let _ = writeln!(out);
    let _ = writeln!(out, "- QuickJS is the primary interpreter baseline.");
    let _ = writeln!(out, "- Boa is a Rust-engine reference point.");
    let _ = writeln!(
        out,
        "- Treat parity as a workload-family measurement, not exact equality across every script."
    );
    if reports
        .iter()
        .any(|report| report.metric_kind == MetricKind::Score)
    {
        let _ = writeln!(
            out,
            "- QuickJS score ratio is `quickjs score / engine score`; lower is better, and QuickJS is `1.00x`."
        );
    }
    let _ = writeln!(out);
    write_workload_table(&mut out, reports);
    write_results_table(&mut out, reports);
    write_profiler_commands(&mut out, reports);
    out
}

fn write_workload_table(out: &mut String, reports: &[EngineWorkloadReport]) {
    let mut seen = Vec::<&str>::new();
    let _ = writeln!(out, "## Workloads");
    let _ = writeln!(out);
    let _ = writeln!(out, "| Workload | Category | Script | Note |");
    let _ = writeln!(out, "| --- | --- | --- | --- |");
    for report in reports {
        if seen.contains(&report.workload_name.as_str()) {
            continue;
        }
        seen.push(&report.workload_name);
        let _ = writeln!(
            out,
            "| `{}` | `{}` | `{}` | {} |",
            report.workload_name,
            report.workload_category,
            report.script_path.display(),
            workload_note(&report.workload_name)
        );
    }
    let _ = writeln!(out);
}

fn workload_note(workload_name: &str) -> &'static str {
    match workload_name {
        "arithmetic-loop" => ARITHMETIC_NOTE,
        "array-object-loop" => ARRAY_OBJECT_NOTE,
        "builtin-string-regexp-loop" => BUILTIN_NOTE,
        _ => "External engine comparison workload.",
    }
}

fn write_results_table(out: &mut String, reports: &[EngineWorkloadReport]) {
    let _ = writeln!(out, "## Results");
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "| Workload | Category | Engine | Metric | Samples | Score median | Wall-time median | Min wall-time | Max wall-time | QuickJS score ratio | QuickJS wall-time ratio | Command |"
    );
    let _ = writeln!(
        out,
        "| --- | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |"
    );
    for report in reports {
        let _ = writeln!(
            out,
            "| `{}` | `{}` | `{}` | `{}` | `{}` | `{}` | `{}` | `{}` | `{}` | `{}` | `{}` | `{}` |",
            report.workload_name,
            report.workload_category,
            report.engine_name,
            report.metric_kind.label(),
            report.samples.len(),
            format_score_median(report),
            format_duration(report.median),
            format_duration(report.min),
            format_duration(report.max),
            format_quickjs_score_ratio(report),
            format_quickjs_wall_time_ratio(report),
            command_line(&report.command)
        );
    }
    let _ = writeln!(out);
}

fn write_profiler_commands(out: &mut String, reports: &[EngineWorkloadReport]) {
    let _ = writeln!(out, "## Profiler Commands");
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "Build Lyng JS first with `cargo build --release -p lyng-js-cli`."
    );
    let _ = writeln!(
        out,
        "Use `--preset profile-target` to regenerate scripts with longer loops before attaching a profiler."
    );
    let _ = writeln!(out);

    for report in reports {
        let sample_file = format!(
            "/tmp/lyng-js-compare-{}-{}.sample.txt",
            report.engine_name, report.workload_name
        );
        let trace_file = format!(
            "/tmp/lyng-js-compare-{}-{}.trace",
            report.engine_name, report.workload_name
        );
        let command = command_line(&report.command);
        let _ = writeln!(
            out,
            "### `{}` on `{}`",
            report.workload_name, report.engine_name
        );
        let _ = writeln!(out);
        let _ = writeln!(out, "```sh");
        let _ = writeln!(out, "{command} &");
        let _ = writeln!(out, "pid=$!");
        let _ = writeln!(out, "sample \"$pid\" 10 -file {sample_file}");
        let _ = writeln!(out, "wait \"$pid\"");
        let _ = writeln!(out);
        let _ = writeln!(out, "xcrun xctrace record \\");
        let _ = writeln!(out, "  --template 'Time Profiler' \\");
        let _ = writeln!(out, "  --output {trace_file} \\");
        let _ = writeln!(out, "  --launch -- {command}");
        let _ = writeln!(out, "```");
        let _ = writeln!(out);
    }
}

fn render_json_report(options: &Options, reports: &[EngineWorkloadReport]) -> Value {
    json!({
        "schema_version": 1,
        "suite": "external-engine-compare",
        "settings": {
            "report_path": options.report_path.as_str(),
            "json_path": options.json_path.as_str(),
            "corpus": options.corpus.label(),
            "filter": options.filter.as_deref(),
            "full_suite": options.full_suite,
            "samples": options.samples,
            "warmup_samples": options.warmup_samples,
            "loop_trips": options.loop_trip_count,
            "scripts_dir": options.scripts_dir.display().to_string(),
            "policy": {
                "primary_baseline": "quickjs",
                "secondary_reference": "boa",
                "parity_rule": "Evaluate measured gaps per workload family rather than exact equality everywhere."
            }
        },
        "engines": options.engines.iter().map(engine_json).collect::<Vec<_>>(),
        "results": reports.iter().map(report_json).collect::<Vec<_>>(),
    })
}

fn engine_json(engine: &EngineConfig) -> Value {
    json!({
        "name": engine.name,
        "executable": engine.executable.as_str(),
        "pre_args": &engine.pre_args,
    })
}

fn report_json(report: &EngineWorkloadReport) -> Value {
    json!({
        "workload": report.workload_name.as_str(),
        "category": report.workload_category.as_str(),
        "script_path": report.script_path.display().to_string(),
        "engine": report.engine_name.as_str(),
        "command": &report.command,
        "metric_kind": report.metric_kind.label(),
        "samples_ms": report.samples.iter().map(|sample| duration_ms(*sample)).collect::<Vec<_>>(),
        "score_samples": &report.score_samples,
        "median_score": report.median_score,
        "median_ms": duration_ms(report.median),
        "min_ms": duration_ms(report.min),
        "max_ms": duration_ms(report.max),
        "quickjs_ratio": report.quickjs_ratio,
    })
}

fn write_report(path: &str, report: &str) -> CompareResult<()> {
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "failed to create external compare report directory `{}`: {error}",
                parent.display()
            )
        })?;
    }
    fs::write(path, report)
        .map_err(|error| format!("failed to write external compare report `{path}`: {error}"))
}

fn print_summary(options: &Options, reports: &[EngineWorkloadReport]) {
    println!(
        "Wrote external engine comparison report to {} and {}",
        options.report_path, options.json_path
    );
    for report in reports {
        let ratio = report
            .quickjs_ratio
            .map_or_else(|| "n/a".to_string(), |ratio| format!("{ratio:.2}x qjs"));
        println!(
            "{} / {}: median {} ({})",
            report.workload_name,
            report.engine_name,
            format_primary_metric(report),
            ratio
        );
    }
}

fn median_duration(mut durations: Vec<Duration>) -> Duration {
    durations.sort();
    durations[durations.len() / 2]
}

fn median_f64(mut values: Vec<f64>) -> f64 {
    values.sort_by(f64::total_cmp);
    values[values.len() / 2]
}

fn duration_ms(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1_000.0
}

fn format_duration(duration: Duration) -> String {
    if duration.as_secs() > 0 {
        format!("{:.3}s", duration.as_secs_f64())
    } else if duration.as_millis() > 0 {
        format!("{:.3}ms", duration.as_secs_f64() * 1_000.0)
    } else if duration.as_micros() > 0 {
        format!("{:.3}us", duration.as_secs_f64() * 1_000_000.0)
    } else {
        format!("{}ns", duration.as_nanos())
    }
}

fn command_line(command: &[String]) -> String {
    command.join(" ")
}

fn format_primary_metric(report: &EngineWorkloadReport) -> String {
    match report.metric_kind {
        MetricKind::WallTime => format_duration(report.median),
        MetricKind::Score => report
            .median_score
            .map_or_else(|| "n/a".to_string(), |score| format!("{score:.3}")),
    }
}

fn format_score_median(report: &EngineWorkloadReport) -> String {
    report
        .median_score
        .map_or_else(|| "n/a".to_string(), |score| format!("{score:.3}"))
}

fn format_quickjs_score_ratio(report: &EngineWorkloadReport) -> String {
    match report.metric_kind {
        MetricKind::Score => format_ratio(report.quickjs_ratio),
        MetricKind::WallTime => "n/a".to_string(),
    }
}

fn format_quickjs_wall_time_ratio(report: &EngineWorkloadReport) -> String {
    match report.metric_kind {
        MetricKind::WallTime => format_ratio(report.quickjs_ratio),
        MetricKind::Score => "n/a".to_string(),
    }
}

fn format_ratio(ratio: Option<f64>) -> String {
    ratio.map_or_else(|| "n/a".to_string(), |ratio| format!("{ratio:.2}x"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::Duration;

    #[test]
    fn smoke_preset_configures_quick_external_engine_loop() {
        let options = parse_options(&[
            "--preset".to_string(),
            "smoke".to_string(),
            "--lyng-js".to_string(),
            "/tmp/lyng-js".to_string(),
            "--qjs".to_string(),
            "/tmp/qjs".to_string(),
            "--boa".to_string(),
            "/tmp/boa".to_string(),
            "--scripts-dir".to_string(),
            "/tmp/compare-scripts".to_string(),
        ])
        .expect("smoke compare options should parse");

        assert_eq!(options.report_path, DEFAULT_REPORT_PATH);
        assert_eq!(options.json_path, DEFAULT_JSON_PATH);
        assert_eq!(options.samples, 1);
        assert_eq!(options.warmup_samples, 0);
        assert_eq!(options.loop_trip_count, 1_024);
        assert_eq!(options.scripts_dir, PathBuf::from("/tmp/compare-scripts"));
        assert_eq!(options.engines[0].name, "lyng-js");
        assert_eq!(options.engines[0].executable, "/tmp/lyng-js");
        assert!(options.engines[0].pre_args.is_empty());
        assert_eq!(options.engines[1].name, "quickjs");
        assert_eq!(options.engines[1].executable, "/tmp/qjs");
        assert_eq!(options.engines[1].pre_args, ["--script"]);
        assert_eq!(options.engines[2].name, "boa");
        assert_eq!(options.engines[2].executable, "/tmp/boa");
    }

    #[test]
    fn default_scripts_dir_is_stable_for_reported_profiler_commands() {
        let options = parse_options(&[]).expect("default compare options should parse");

        assert_eq!(
            options.scripts_dir,
            PathBuf::from("/tmp/lyng-js-bench-compare-scripts")
        );
    }

    #[test]
    fn baseline_and_profile_presets_stay_practical_for_process_comparison() {
        let baseline = parse_options(&["--preset".to_string(), "baseline".to_string()])
            .expect("baseline compare options should parse");
        let profile = parse_options(&["--preset".to_string(), "profile-target".to_string()])
            .expect("profile compare options should parse");

        assert_eq!(baseline.samples, 3);
        assert_eq!(baseline.warmup_samples, 1);
        assert_eq!(baseline.loop_trip_count, 2_048);
        assert_eq!(profile.samples, 1);
        assert_eq!(profile.warmup_samples, 0);
        assert_eq!(profile.loop_trip_count, 65_536);
    }

    #[test]
    fn v8_v7_options_select_local_corpus_and_filter() {
        let options = parse_options(&[
            "--corpus".to_string(),
            "v8-v7".to_string(),
            "--filter".to_string(),
            "Richards".to_string(),
        ])
        .expect("v8-v7 compare options should parse");

        assert_eq!(options.corpus, Corpus::V8V7);
        assert_eq!(options.filter.as_deref(), Some("Richards"));
        assert_eq!(options.report_path, DEFAULT_REPORT_PATH);
        assert_eq!(options.json_path, DEFAULT_JSON_PATH);
    }

    #[test]
    fn v8_v7_filter_generates_standalone_richards_bundle() {
        let options = parse_options(&[
            "--corpus".to_string(),
            "v8-v7".to_string(),
            "--filter".to_string(),
            "Richards".to_string(),
        ])
        .expect("v8-v7 compare options should parse");
        let workloads = build_selected_workloads(&options).expect("v8-v7 workloads should build");

        assert_eq!(workloads.len(), 1);
        let workload = &workloads[0];
        assert_eq!(workload.name, "Richards");
        assert_eq!(workload.category, "v8-v7");
        assert_eq!(workload.file_name, "v8-v7-richards.js");
        assert_eq!(workload.metric_kind, MetricKind::Score);
        assert!(workload.requires_lyng_shell);
        assert!(workload
            .source
            .contains("function Benchmark(name, run, setup, tearDown)"));
        assert!(workload
            .source
            .contains("var Richards = new BenchmarkSuite('Richards'"));
        assert!(!workload
            .source
            .contains("var DeltaBlue = new BenchmarkSuite"));
        assert!(workload.source.contains("LyngV8V7PrintResult"));
        assert!(workload.source.contains("BenchmarkSuite.RunSuites"));
    }

    #[test]
    fn v8_v7_workload_set_covers_old_octane_benchmarks() {
        let options = parse_options(&["--corpus".to_string(), "v8-v7".to_string()])
            .expect("v8-v7 compare options should parse");
        let workloads = build_selected_workloads(&options).expect("v8-v7 workloads should build");
        let names = workloads
            .iter()
            .map(|workload| workload.name)
            .collect::<Vec<_>>();

        assert_eq!(
            names,
            [
                "Richards",
                "DeltaBlue",
                "Crypto",
                "RayTrace",
                "EarleyBoyer",
                "RegExp",
                "Splay",
                "NavierStokes",
            ]
        );
    }

    #[test]
    fn v8_v7_engine_commands_use_shell_for_lyng_only() {
        let options = parse_options(&[
            "--corpus".to_string(),
            "v8-v7".to_string(),
            "--filter".to_string(),
            "Richards".to_string(),
        ])
        .expect("v8-v7 compare options should parse");
        let workload = build_selected_workloads(&options)
            .expect("v8-v7 workloads should build")
            .pop()
            .expect("filtered workload should exist");
        let script_path = Path::new("/tmp/compare/v8-v7-richards.js");

        assert_eq!(
            command_vector(&options.engines[0], &workload, script_path),
            [
                "target/release/lyng-js".to_string(),
                "--shell".to_string(),
                "/tmp/compare/v8-v7-richards.js".to_string(),
            ]
        );
        assert_eq!(
            command_vector(&options.engines[1], &workload, script_path),
            [
                options.engines[1].executable.clone(),
                "--script".to_string(),
                "/tmp/compare/v8-v7-richards.js".to_string(),
            ]
        );
    }

    #[test]
    fn workload_set_covers_required_baseline_categories() {
        let workloads = build_workloads(16);
        let categories = workloads
            .iter()
            .map(|workload| workload.category)
            .collect::<Vec<_>>();

        assert_eq!(
            categories,
            ["arithmetic-control-flow", "array-object", "builtin-heavy"]
        );
        for workload in workloads {
            assert!(workload.source.contains("var __lyngBenchTrips = 16;"));
            assert!(workload.source.contains("var __lyngBenchSink"));
            assert!(workload.source.contains("throw new Error"));
            assert!(Path::new(workload.file_name)
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("js")));
        }
    }

    #[test]
    fn report_documents_quickjs_policy_ratios_and_profiler_commands() {
        let options = Options {
            report_path: "/tmp/compare.md".to_string(),
            json_path: "/tmp/compare.json".to_string(),
            samples: 1,
            warmup_samples: 0,
            loop_trip_count: 16,
            scripts_dir: PathBuf::from("/tmp/compare-scripts"),
            engines: vec![
                EngineConfig::new("lyng-js", "target/release/lyng-js", []),
                EngineConfig::new("quickjs", "qjs", ["--script"]),
                EngineConfig::new("boa", "boa", []),
            ],
            corpus: Corpus::Synthetic,
            filter: None,
            full_suite: false,
        };
        let reports = synthetic_reports();

        let markdown = render_report(&options, &reports);

        assert!(markdown.contains("QuickJS is the primary interpreter baseline."));
        assert!(markdown.contains("Boa is a Rust-engine reference point."));
        assert!(markdown.contains("| `arithmetic-loop` | `arithmetic-control-flow` | `lyng-js` |"));
        assert!(markdown.contains("2.00x"));
        assert!(markdown.contains("target/release/lyng-js /tmp/compare-scripts/arithmetic-loop.js"));
        assert!(markdown.contains("qjs --script /tmp/compare-scripts/arithmetic-loop.js"));
        assert!(markdown.contains("boa /tmp/compare-scripts/arithmetic-loop.js"));
        assert!(markdown.contains(
            "sample \"$pid\" 10 -file /tmp/lyng-js-compare-lyng-js-arithmetic-loop.sample.txt"
        ));
        assert!(markdown.contains("xcrun xctrace record"));
    }

    #[test]
    fn json_report_records_engine_metadata_samples_and_quickjs_ratios() {
        let options = Options {
            report_path: "/tmp/compare.md".to_string(),
            json_path: "/tmp/compare.json".to_string(),
            samples: 1,
            warmup_samples: 0,
            loop_trip_count: 16,
            scripts_dir: PathBuf::from("/tmp/compare-scripts"),
            engines: vec![
                EngineConfig::new("lyng-js", "target/release/lyng-js", []),
                EngineConfig::new("quickjs", "qjs", ["--script"]),
            ],
            corpus: Corpus::Synthetic,
            filter: None,
            full_suite: false,
        };
        let reports = synthetic_reports()
            .into_iter()
            .filter(|report| report.engine_name != "boa")
            .collect::<Vec<_>>();

        let json = render_json_report(&options, &reports);

        assert_eq!(json["schema_version"], 1);
        assert_eq!(json["suite"], "external-engine-compare");
        assert_eq!(json["settings"]["samples"], 1);
        assert_eq!(json["settings"]["loop_trips"], 16);
        assert_eq!(json["engines"][0]["name"], "lyng-js");
        assert_eq!(json["engines"][1]["pre_args"][0], "--script");
        assert_eq!(json["results"][0]["engine"], "lyng-js");
        assert_eq!(json["results"][0]["quickjs_ratio"], 2.0);
        assert_eq!(json["results"][1]["quickjs_ratio"], 1.0);
        assert_eq!(json["results"][0]["samples_ms"][0], 20.0);
    }

    fn synthetic_reports() -> Vec<EngineWorkloadReport> {
        vec![
            EngineWorkloadReport {
                workload_name: "arithmetic-loop".to_string(),
                workload_category: "arithmetic-control-flow".to_string(),
                script_path: PathBuf::from("/tmp/compare-scripts/arithmetic-loop.js"),
                engine_name: "lyng-js".to_string(),
                command: vec![
                    "target/release/lyng-js".to_string(),
                    "/tmp/compare-scripts/arithmetic-loop.js".to_string(),
                ],
                samples: vec![Duration::from_millis(20)],
                median: Duration::from_millis(20),
                min: Duration::from_millis(20),
                max: Duration::from_millis(20),
                score_samples: Vec::new(),
                median_score: None,
                quickjs_ratio: Some(2.0),
                metric_kind: MetricKind::WallTime,
            },
            EngineWorkloadReport {
                workload_name: "arithmetic-loop".to_string(),
                workload_category: "arithmetic-control-flow".to_string(),
                script_path: PathBuf::from("/tmp/compare-scripts/arithmetic-loop.js"),
                engine_name: "quickjs".to_string(),
                command: vec![
                    "qjs".to_string(),
                    "--script".to_string(),
                    "/tmp/compare-scripts/arithmetic-loop.js".to_string(),
                ],
                samples: vec![Duration::from_millis(10)],
                median: Duration::from_millis(10),
                min: Duration::from_millis(10),
                max: Duration::from_millis(10),
                score_samples: Vec::new(),
                median_score: None,
                quickjs_ratio: Some(1.0),
                metric_kind: MetricKind::WallTime,
            },
            EngineWorkloadReport {
                workload_name: "arithmetic-loop".to_string(),
                workload_category: "arithmetic-control-flow".to_string(),
                script_path: PathBuf::from("/tmp/compare-scripts/arithmetic-loop.js"),
                engine_name: "boa".to_string(),
                command: vec![
                    "boa".to_string(),
                    "/tmp/compare-scripts/arithmetic-loop.js".to_string(),
                ],
                samples: vec![Duration::from_millis(30)],
                median: Duration::from_millis(30),
                min: Duration::from_millis(30),
                max: Duration::from_millis(30),
                score_samples: Vec::new(),
                median_score: None,
                quickjs_ratio: Some(3.0),
                metric_kind: MetricKind::WallTime,
            },
        ]
    }

    #[test]
    fn v8_v7_score_reports_parse_and_render_quickjs_ratios() {
        let options = parse_options(&[
            "--corpus".to_string(),
            "v8-v7".to_string(),
            "--filter".to_string(),
            "Richards".to_string(),
        ])
        .expect("v8-v7 compare options should parse");
        let reports = vec![
            EngineWorkloadReport {
                workload_name: "Richards".to_string(),
                workload_category: "v8-v7".to_string(),
                script_path: PathBuf::from("/tmp/compare/v8-v7-richards.js"),
                engine_name: "lyng-js".to_string(),
                command: vec![
                    "target/release/lyng-js".to_string(),
                    "--shell".to_string(),
                    "/tmp/compare/v8-v7-richards.js".to_string(),
                ],
                samples: vec![Duration::from_secs(1)],
                median: Duration::from_secs(1),
                min: Duration::from_secs(1),
                max: Duration::from_secs(1),
                score_samples: vec![120.0],
                median_score: Some(120.0),
                quickjs_ratio: Some(8.0),
                metric_kind: MetricKind::Score,
            },
            EngineWorkloadReport {
                workload_name: "Richards".to_string(),
                workload_category: "v8-v7".to_string(),
                script_path: PathBuf::from("/tmp/compare/v8-v7-richards.js"),
                engine_name: "quickjs".to_string(),
                command: vec![
                    "qjs".to_string(),
                    "--script".to_string(),
                    "/tmp/compare/v8-v7-richards.js".to_string(),
                ],
                samples: vec![Duration::from_secs(1)],
                median: Duration::from_secs(1),
                min: Duration::from_secs(1),
                max: Duration::from_secs(1),
                score_samples: vec![960.0],
                median_score: Some(960.0),
                quickjs_ratio: Some(1.0),
                metric_kind: MetricKind::Score,
            },
        ];

        let markdown = render_report(&options, &reports);
        assert!(markdown.contains("Score median"));
        assert!(markdown.contains("QuickJS score ratio"));
        assert!(markdown.contains("8.00x"));

        let json = render_json_report(&options, &reports);
        assert_eq!(json["settings"]["corpus"], "v8-v7");
        assert_eq!(json["results"][0]["metric_kind"], "score");
        assert_eq!(json["results"][0]["median_score"], 120.0);
        assert_eq!(json["results"][0]["score_samples"][0], 120.0);
        assert_eq!(json["results"][0]["quickjs_ratio"], 8.0);
    }
}
