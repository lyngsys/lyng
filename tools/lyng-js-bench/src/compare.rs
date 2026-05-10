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
}

#[derive(Clone)]
struct Workload {
    name: &'static str,
    category: &'static str,
    file_name: &'static str,
    source: String,
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
    quickjs_ratio: Option<f64>,
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

    let workloads = build_workloads(options.loop_trip_count);
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

    Ok(options)
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
    "Usage: lyng-js-bench compare [--preset <smoke|baseline|profile-target>] [--report <path>] [--json <path>] [--samples <n>] [--warmup-samples <n>] [--loop-trips <n>] [--scripts-dir <path>] [--lyng-js <path>] [--qjs <path>] [--boa <path>]"
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
        },
        Workload {
            name: "array-object-loop",
            category: "array-object",
            file_name: "array-object-loop.js",
            source: array_object_workload(loop_trip_count),
        },
        Workload {
            name: "builtin-string-regexp-loop",
            category: "builtin-heavy",
            file_name: "builtin-string-regexp-loop.js",
            source: builtin_workload(loop_trip_count),
        },
    ]
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
        let _ = run_engine_once(engine, script_path, workload.name)?;
    }

    let mut samples = Vec::with_capacity(options.samples);
    for _ in 0..options.samples {
        samples.push(run_engine_once(engine, script_path, workload.name)?);
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

    Ok(EngineWorkloadReport {
        workload_name: workload.name.to_string(),
        workload_category: workload.category.to_string(),
        script_path: script_path.to_path_buf(),
        engine_name: engine.name.to_string(),
        command: command_vector(engine, script_path),
        samples,
        median,
        min,
        max,
        quickjs_ratio: None,
    })
}

fn run_engine_once(
    engine: &EngineConfig,
    script_path: &Path,
    workload_name: &str,
) -> CompareResult<Duration> {
    let mut command = Command::new(&engine.executable);
    for arg in &engine.pre_args {
        command.arg(arg);
    }
    command.arg(script_path);

    let start = Instant::now();
    let output = command.output().map_err(|error| {
        format!(
            "failed to launch external engine `{}` for workload `{workload_name}`: {error}",
            engine.name
        )
    })?;
    let elapsed = start.elapsed();

    if !output.status.success() {
        return Err(format!(
            "external engine `{}` failed for workload `{workload_name}` with status {}\nstdout:\n{}\nstderr:\n{}",
            engine.name,
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(elapsed)
}

fn command_vector(engine: &EngineConfig, script_path: &Path) -> Vec<String> {
    let mut command = Vec::with_capacity(engine.pre_args.len() + 2);
    command.push(engine.executable.clone());
    command.extend(engine.pre_args.iter().cloned());
    command.push(script_path.display().to_string());
    command
}

fn attach_quickjs_ratios(reports: &mut [EngineWorkloadReport]) {
    let quickjs_medians = reports
        .iter()
        .filter(|report| report.engine_name == "quickjs")
        .map(|report| (report.workload_name.clone(), report.median))
        .collect::<Vec<_>>();

    for report in reports {
        report.quickjs_ratio = quickjs_medians
            .iter()
            .find(|(workload_name, _)| workload_name == &report.workload_name)
            .and_then(|(_, quickjs_median)| {
                let quickjs_ms = duration_ms(*quickjs_median);
                if quickjs_ms > 0.0 {
                    Some(duration_ms(report.median) / quickjs_ms)
                } else {
                    None
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
        "| Workload | Category | Engine | Samples | Median | Min | Max | QuickJS ratio | Command |"
    );
    let _ = writeln!(
        out,
        "| --- | --- | --- | ---: | ---: | ---: | ---: | ---: | --- |"
    );
    for report in reports {
        let ratio = report
            .quickjs_ratio
            .map_or_else(|| "n/a".to_string(), |ratio| format!("{ratio:.2}x"));
        let _ = writeln!(
            out,
            "| `{}` | `{}` | `{}` | `{}` | `{}` | `{}` | `{}` | `{}` | `{}` |",
            report.workload_name,
            report.workload_category,
            report.engine_name,
            report.samples.len(),
            format_duration(report.median),
            format_duration(report.min),
            format_duration(report.max),
            ratio,
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
            "report_path": options.report_path,
            "json_path": options.json_path,
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
        "executable": engine.executable,
        "pre_args": engine.pre_args,
    })
}

fn report_json(report: &EngineWorkloadReport) -> Value {
    json!({
        "workload": report.workload_name,
        "category": report.workload_category,
        "script_path": report.script_path.display().to_string(),
        "engine": report.engine_name,
        "command": report.command,
        "samples_ms": report.samples.iter().map(|sample| duration_ms(*sample)).collect::<Vec<_>>(),
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
            format_duration(report.median),
            ratio
        );
    }
}

fn median_duration(mut durations: Vec<Duration>) -> Duration {
    durations.sort();
    durations[durations.len() / 2]
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
                quickjs_ratio: Some(2.0),
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
                quickjs_ratio: Some(1.0),
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
                quickjs_ratio: Some(3.0),
            },
        ]
    }
}
