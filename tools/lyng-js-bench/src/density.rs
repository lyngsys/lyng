#![allow(clippy::too_many_lines)]

use lyng_js_bytecode::CompiledScriptUnit;
use lyng_js_common::{AtomTable, SourceId};
use lyng_js_compiler::compile_script;
use lyng_js_env::Runtime;
use lyng_js_host::NoopHostHooks;
use lyng_js_parser::parse_script;
use lyng_js_sema::analyze_script;
use lyng_js_vm::Vm;
use std::env;
use std::fmt::Write;
use std::fs;
use std::hint::black_box;
use std::path::Path;
use std::time::{Duration, Instant};

pub const DEFAULT_SAMPLES: usize = 7;
pub const DEFAULT_EVALS: usize = 15;
pub const DEFAULT_LOOP_TRIPS: usize = 2_048;

struct Options {
    report_path: String,
    samples: usize,
    evals_per_sample: usize,
    loop_trip_count: usize,
}

#[derive(Clone)]
struct Workload {
    name: &'static str,
    note: &'static str,
    source: String,
}

#[derive(Clone)]
struct DensityMetrics {
    functions: usize,
    entry_words: usize,
    entry_bytes: usize,
    unit_words: usize,
    unit_bytes: usize,
    base_words: usize,
    wide_words: usize,
    wide_share_percent: f64,
    metadata_records: usize,
    max_registers: u16,
}

#[derive(Clone)]
struct ThroughputResult {
    name: &'static str,
    note: &'static str,
    samples: usize,
    evals_per_sample: usize,
    median_total: Duration,
    median_us_per_eval: f64,
    checksum: u64,
}

#[derive(Clone, Copy)]
struct SampleResult {
    elapsed: Duration,
    checksum: u64,
}

#[derive(Clone)]
struct WorkloadReport {
    name: &'static str,
    note: &'static str,
    density: DensityMetrics,
    throughput: ThroughputResult,
}

pub fn run(args: &[String]) -> Result<(), String> {
    let options = parse_options(args)?;

    if cfg!(debug_assertions) {
        eprintln!("warning: build with --release for meaningful measurements");
    }

    let workloads = build_workloads(options.loop_trip_count);
    let reports = workloads
        .iter()
        .map(|workload| measure_workload(workload, &options))
        .collect::<Vec<_>>();
    let report = render_report(&options, &reports);
    write_report(&options.report_path, &report);
    print_summary(&options.report_path, &reports);
    Ok(())
}

fn parse_options(args: &[String]) -> Result<Options, String> {
    let mut options = Options {
        report_path: default_report_path(env::consts::ARCH),
        samples: DEFAULT_SAMPLES,
        evals_per_sample: DEFAULT_EVALS,
        loop_trip_count: DEFAULT_LOOP_TRIPS,
    };

    let mut args = args.iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--report" => {
                options.report_path = args.next().map_or_else(
                    || Err("--report requires a path".to_string()),
                    |value| Ok(value.to_string()),
                )?;
            }
            "--samples" => {
                options.samples = parse_usize_arg("--samples", args.next())?;
            }
            "--evals" => {
                options.evals_per_sample = parse_usize_arg("--evals", args.next())?;
            }
            "--loop-trips" => {
                options.loop_trip_count = parse_usize_arg("--loop-trips", args.next())?;
            }
            "--help" | "-h" => {
                return Err(usage());
            }
            unknown => return Err(format!("Unknown argument: {unknown}")),
        }
    }

    if options.samples == 0 {
        return Err("--samples must be greater than zero".to_string());
    }
    if options.evals_per_sample == 0 {
        return Err("--evals must be greater than zero".to_string());
    }
    if options.loop_trip_count == 0 {
        return Err("--loop-trips must be greater than zero".to_string());
    }

    Ok(options)
}

fn usage() -> String {
    "Usage: lyng-js-bench density [--report <path>] [--samples <n>] [--evals <n>] [--loop-trips <n>]"
        .to_string()
}

pub fn default_report_path(arch: &str) -> String {
    format!("reports/js/lyng-js/bytecode-density-{arch}.md")
}

fn parse_usize_arg(flag: &str, value: Option<&String>) -> Result<usize, String> {
    value
        .ok_or_else(|| format!("{flag} requires a value"))?
        .parse()
        .map_err(|_| format!("{flag} expects a positive integer"))
}

fn build_workloads(loop_trip_count: usize) -> Vec<Workload> {
    vec![
        Workload {
            name: "script.core.objects-and-arrays",
            note: "Representative script-core locals, arrays, object literals, named properties, and while-loop control flow.",
            source: script_core_workload(loop_trip_count),
        },
        Workload {
            name: "functions.closure-calls",
            note: "Closure capture and repeated bytecode call dispatch inside one loop-heavy script evaluation.",
            source: closure_call_workload(loop_trip_count),
        },
        Workload {
            name: "activation.arguments-rest-for-in",
            note: "Mapped arguments, rest arrays, and ordinary-object for-in enumeration inside one activation-heavy path.",
            source: activation_workload(loop_trip_count / 4),
        },
        Workload {
            name: "exceptions.try-catch-finally",
            note: "Catch/finally edges and abrupt-completion bookkeeping on a looped control-flow-heavy workload.",
            source: exception_workload(loop_trip_count / 4),
        },
        Workload {
            name: "wide.large-register-function",
            note: "Large-register stress case that forces wide operands without changing the default 4-byte base layout.",
            source: wide_register_workload(loop_trip_count / 8),
        },
    ]
}

fn measure_workload(workload: &Workload, options: &Options) -> WorkloadReport {
    let mut atoms = AtomTable::new();
    let unit = compile_unit(&workload.source, &mut atoms);
    let density = collect_density_metrics(&unit);
    let throughput = measure_throughput(workload, &unit, options);
    WorkloadReport {
        name: workload.name,
        note: workload.note,
        density,
        throughput,
    }
}

fn compile_unit(source: &str, atoms: &mut AtomTable) -> CompiledScriptUnit {
    let parsed = parse_script(atoms, SourceId::new(0), source);
    assert!(
        !parsed.diagnostics.has_errors(),
        "unexpected parse errors in benchmark workload: {:?}",
        parsed.diagnostics.as_slice()
    );
    let sema = analyze_script(&parsed, atoms);
    assert!(
        !sema.diagnostics.has_errors(),
        "unexpected sema errors in benchmark workload: {:?}",
        sema.diagnostics.as_slice()
    );
    compile_script(&parsed, &sema, atoms).expect("benchmark workload should lower")
}

fn collect_density_metrics(unit: &CompiledScriptUnit) -> DensityMetrics {
    let entry = unit
        .function(unit.entry())
        .expect("compiled unit should contain its entry function");
    let entry_words = entry.instructions().len() + entry.wide_operands().len();
    let unit_words = unit
        .functions()
        .iter()
        .map(|function| function.instructions().len() + function.wide_operands().len())
        .sum::<usize>();
    let base_words = unit
        .functions()
        .iter()
        .map(|function| function.instructions().len())
        .sum::<usize>();
    let wide_words = unit
        .functions()
        .iter()
        .map(|function| function.wide_operands().len())
        .sum::<usize>();
    let metadata_records = unit
        .functions()
        .iter()
        .map(|function| {
            function.constants().len()
                + function.child_functions().len()
                + function.captures().len()
                + function.exception_handlers().len()
                + function.feedback_sites().len()
                + function.source_map().len()
                + function.safepoints().len()
                + function.deopt_snapshots().len()
        })
        .sum::<usize>();
    let max_registers = unit
        .functions()
        .iter()
        .map(|function| {
            function
                .register_count()
                .saturating_add(function.hidden_register_count())
        })
        .max()
        .unwrap_or(0);
    let encoded_words = base_words + wide_words;

    DensityMetrics {
        functions: unit.functions().len(),
        entry_words,
        entry_bytes: entry_words * 4,
        unit_words,
        unit_bytes: unit_words * 4,
        base_words,
        wide_words,
        wide_share_percent: if encoded_words == 0 {
            0.0
        } else {
            (wide_words as f64 / encoded_words as f64) * 100.0
        },
        metadata_records,
        max_registers,
    }
}

fn measure_throughput(
    workload: &Workload,
    unit: &CompiledScriptUnit,
    options: &Options,
) -> ThroughputResult {
    let expected_bits = single_eval_bits(unit);
    let mut samples = Vec::with_capacity(options.samples);
    for _ in 0..options.samples {
        let start = Instant::now();
        let mut checksum = 0_u64;
        for _ in 0..options.evals_per_sample {
            let bits = black_box(single_eval_bits(unit));
            assert_eq!(
                bits, expected_bits,
                "benchmark workload {} returned an unstable result",
                workload.name
            );
            checksum = checksum.wrapping_add(bits);
        }
        samples.push(SampleResult {
            elapsed: start.elapsed(),
            checksum,
        });
    }

    samples.sort_by(|left, right| left.elapsed.cmp(&right.elapsed));
    let median = samples[samples.len() / 2];
    ThroughputResult {
        name: workload.name,
        note: workload.note,
        samples: options.samples,
        evals_per_sample: options.evals_per_sample,
        median_total: median.elapsed,
        median_us_per_eval: median.elapsed.as_secs_f64() * 1_000_000.0
            / options.evals_per_sample as f64,
        checksum: median.checksum,
    }
}

fn single_eval_bits(unit: &CompiledScriptUnit) -> u64 {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    vm.evaluate_script(agent, realm, unit)
        .expect("benchmark workload should execute")
        .bits()
}

fn render_report(options: &Options, reports: &[WorkloadReport]) -> String {
    let mut out = String::new();
    let profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    let aggregate = aggregate_density(reports);
    let invocation = invocation_hint(&options.report_path);

    let _ = writeln!(
        out,
        "# Lyng JS Bytecode Density and Instruction-Cache Proxy ({})",
        env::consts::ARCH
    );
    let _ = writeln!(out);
    let _ = writeln!(out, "This report is generated by `{invocation}`.");
    let _ = writeln!(
        out,
        "It covers the current Lyng JS bytecode-density workload corpus: script-core execution, closures and calls, activation-heavy `arguments` or rest or `for-in` paths, exception-heavy control flow, and one large-register stress case that forces wide operands."
    );
    let _ = writeln!(out);
    let _ = writeln!(out, "## Settings");
    let _ = writeln!(out);
    let _ = writeln!(out, "- Profile: `{profile}`");
    let _ = writeln!(out, "- Target OS: `{}`", env::consts::OS);
    let _ = writeln!(out, "- Target architecture: `{}`", env::consts::ARCH);
    let _ = writeln!(out, "- Samples per workload: `{}`", options.samples);
    let _ = writeln!(
        out,
        "- Fresh install-plus-execute evaluations per sample: `{}`",
        options.evals_per_sample
    );
    let _ = writeln!(
        out,
        "- Primary loop trip count seed: `{}`",
        options.loop_trip_count
    );
    let _ = writeln!(out);
    let _ = writeln!(out, "## Static Density");
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "| Workload | Functions | Entry words | Entry bytes | Unit words | Unit bytes | Base words | Wide payload words | Wide share | Metadata records | Max registers | Note |"
    );
    let _ = writeln!(
        out,
        "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |"
    );
    for report in reports {
        let _ = writeln!(
            out,
            "| `{}` | `{}` | `{}` | `{}` | `{}` | `{}` | `{}` | `{}` | `{:.2}%` | `{}` | `{}` | {} |",
            report.name,
            report.density.functions,
            report.density.entry_words,
            report.density.entry_bytes,
            report.density.unit_words,
            report.density.unit_bytes,
            report.density.base_words,
            report.density.wide_words,
            report.density.wide_share_percent,
            report.density.metadata_records,
            report.density.max_registers,
            report.note,
        );
    }
    let _ = writeln!(out);
    let _ = writeln!(out, "## Runtime Throughput Proxy");
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "| Workload | Samples | Evals/sample | Median total | Median us/eval | Entry bytes | Unit bytes | Note |"
    );
    let _ = writeln!(
        out,
        "| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |"
    );
    for report in reports {
        let _ = writeln!(
            out,
            "| `{}` | `{}` | `{}` | `{}` | `{:.2}` | `{}` | `{}` | {} |",
            report.throughput.name,
            report.throughput.samples,
            report.throughput.evals_per_sample,
            format_duration(report.throughput.median_total),
            report.throughput.median_us_per_eval,
            report.density.entry_bytes,
            report.density.unit_bytes,
            report.throughput.note,
        );
    }
    let _ = writeln!(out);
    let _ = writeln!(out, "## Aggregate Density");
    let _ = writeln!(out);
    let _ = writeln!(out, "- Workloads measured: `{}`", reports.len());
    let _ = writeln!(out, "- Aggregate unit bytes: `{}`", aggregate.unit_bytes);
    let _ = writeln!(out, "- Aggregate base words: `{}`", aggregate.base_words);
    let _ = writeln!(
        out,
        "- Aggregate wide payload words: `{}`",
        aggregate.wide_words
    );
    let _ = writeln!(
        out,
        "- Aggregate wide share: `{:.2}%`",
        aggregate.wide_share_percent
    );
    let _ = writeln!(
        out,
        "- Largest entry bytecode body: `{}`",
        aggregate.max_entry_bytes
    );
    let _ = writeln!(
        out,
        "- Largest unit bytecode body: `{}`",
        aggregate.max_unit_bytes
    );
    let _ = writeln!(out);
    let _ = writeln!(out, "## Notes");
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "- Bytecode density is architecture-independent because the encoded stream is compiler-owned data, not native machine code. The static-density section should therefore match across `aarch64` and `x86_64` runs for the same source corpus."
    );
    let _ = writeln!(
        out,
        "- The runtime section is an instruction-cache proxy, not a hardware-counter study. It measures fresh install plus execute on the same workload corpus so the VM sees real bytecode shapes without cross-iteration global-state bleed."
    );
    let _ = writeln!(
        out,
        "- The large-register row exists specifically to exercise wide operands and test whether the base 4-byte word plus one extra payload word remains acceptable without redesigning the common-case instruction layout."
    );

    out
}

fn invocation_hint(report_path: &str) -> String {
    if env::consts::OS == "macos" && env::consts::ARCH == "x86_64" {
        return format!(
            "cargo run --release --target x86_64-apple-darwin -p lyng-js-bench -- density --report {report_path}"
        );
    }
    format!("cargo run --release -p lyng-js-bench -- density --report {report_path}")
}

#[derive(Default)]
struct AggregateDensity {
    unit_bytes: usize,
    base_words: usize,
    wide_words: usize,
    wide_share_percent: f64,
    max_entry_bytes: usize,
    max_unit_bytes: usize,
}

fn aggregate_density(reports: &[WorkloadReport]) -> AggregateDensity {
    let unit_bytes = reports.iter().map(|report| report.density.unit_bytes).sum();
    let base_words = reports.iter().map(|report| report.density.base_words).sum();
    let wide_words = reports.iter().map(|report| report.density.wide_words).sum();
    let encoded_words = base_words + wide_words;

    AggregateDensity {
        unit_bytes,
        base_words,
        wide_words,
        wide_share_percent: if encoded_words == 0 {
            0.0
        } else {
            (wide_words as f64 / encoded_words as f64) * 100.0
        },
        max_entry_bytes: reports
            .iter()
            .map(|report| report.density.entry_bytes)
            .max()
            .unwrap_or(0),
        max_unit_bytes: reports
            .iter()
            .map(|report| report.density.unit_bytes)
            .max()
            .unwrap_or(0),
    }
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

fn write_report(path: &str, report: &str) {
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent).expect("failed to create report directory");
    }
    fs::write(path, report).expect("failed to write report");
}

fn print_summary(path: &str, reports: &[WorkloadReport]) {
    println!("Report written to {path}");
    for report in reports {
        println!(
            "{}: {} entry bytes, {} unit bytes, {:.2}% wide, {:.2} us/eval, checksum {}",
            report.name,
            report.density.entry_bytes,
            report.density.unit_bytes,
            report.density.wide_share_percent,
            report.throughput.median_us_per_eval,
            report.throughput.checksum,
        );
    }
}

fn script_core_workload(loop_trip_count: usize) -> String {
    format!(
        r#"
        let obj = {{ base: 3 }};
        let arr = [1, 2, 3, 4];
        let total = 0;
        let index = 0;
        let i = 0;
        while (i < {loop_trip_count}) {{
            obj.base = obj.base + arr[index];
            total = total + obj.base;
            index = index + 1;
            if (index == 4) {{
                index = 0;
            }}
            i = i + 1;
        }}
        total;
        "#
    )
}

fn closure_call_workload(loop_trip_count: usize) -> String {
    format!(
        r#"
        function outer(base) {{
            return function(step) {{
                return base + step;
            }};
        }}
        let add = outer(7);
        let total = 0;
        let i = 0;
        while (i < {loop_trip_count}) {{
            total = total + add(i);
            i = i + 1;
        }}
        total;
        "#
    )
}

fn activation_workload(loop_trip_count: usize) -> String {
    format!(
        r#"
        function alias(a, b) {{
            a = 7;
            arguments[1] = 9;
            return arguments[0] + b;
        }}
        function collect(head, ...rest) {{
            rest[1] = rest[1] + 1;
            return rest[0] + rest[1] + rest.length;
        }}
        function readObject(obj) {{
            let total = 0;
            for (var key in obj) {{
                total = total + obj[key];
            }}
            return total;
        }}
        let obj = {{ alpha: 1, beta: 2, gamma: 3 }};
        let total = 0;
        let i = 0;
        while (i < {loop_trip_count}) {{
            total = total + alias(1, 2);
            total = total + collect(1, 2, 3);
            total = total + readObject(obj);
            i = i + 1;
        }}
        total;
        "#
    )
}

fn exception_workload(loop_trip_count: usize) -> String {
    format!(
        r#"
        function step(value) {{
            try {{
                if (value < 2) {{
                    throw 6;
                }}
                return value + 3;
            }} catch (error) {{
                return error;
            }} finally {{
                value = value + 1;
            }}
        }}
        let total = 0;
        let i = 0;
        while (i < {loop_trip_count}) {{
            total = total + step(i < 2 ? i : 3);
            i = i + 1;
        }}
        total;
        "#
    )
}

fn wide_register_workload(loop_trip_count: usize) -> String {
    let mut out = String::from(
        r#"
        let fnRef = function(value) {
            return value;
        };
"#,
    );
    for index in 0..320 {
        let _ = writeln!(out, "            let value{index} = {index};");
    }
    let _ = writeln!(
        out,
        r#"
        let total = 0;
        let i = 0;
        while (i < {loop_trip_count}) {{
            total = total + fnRef(value319) + value318;
            i = i + 1;
        }}
        total;
"#
    );
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_workloads_compile() {
        let mut atoms = AtomTable::new();
        for workload in build_workloads(64) {
            let unit = compile_unit(&workload.source, &mut atoms);
            assert!(
                !unit.functions().is_empty(),
                "{} should compile",
                workload.name
            );
        }
    }

    #[test]
    fn density_metrics_count_wide_words() {
        let mut atoms = AtomTable::new();
        let unit = compile_unit(&wide_register_workload(8), &mut atoms);
        let density = collect_density_metrics(&unit);
        assert!(density.wide_words > 0);
        assert!(density.unit_bytes >= density.entry_bytes);
        assert!(density.max_registers > 255);
    }

    #[test]
    fn density_report_path_drops_phase_naming() {
        assert_eq!(
            default_report_path("aarch64"),
            "reports/js/lyng-js/bytecode-density-aarch64.md"
        );
    }
}
