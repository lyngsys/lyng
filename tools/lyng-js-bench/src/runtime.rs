use lyng_js_builtins::BootstrapMode;
use lyng_js_bytecode::{BytecodeFunction, CompiledAtom, CompiledScriptUnit};
use lyng_js_common::{AtomTable, SourceId};
use lyng_js_compiler::{compile_module, compile_script, CompiledModuleUnit};
use lyng_js_env::{ExecutableId, Runtime, RuntimePhase6Accounting as RuntimeAccounting};
use lyng_js_host::{HostJobKind, HostSharedBufferId, NoopHostHooks};
use lyng_js_parser::{parse_module, parse_script};
use lyng_js_sema::{analyze_module, analyze_script};
use lyng_js_types::CodeRef;
use lyng_js_vm::Vm;
use std::cmp::Ordering;
use std::env;
use std::fmt::Write;
use std::fs;
use std::hint::black_box;
use std::mem::{size_of, size_of_val};
use std::path::Path;
use std::time::{Duration, Instant};

pub const DEFAULT_REPORT_PATH: &str = "reports/js/lyng-js/bench.md";
const DEFAULT_SAMPLES: usize = 7;
const DEFAULT_RUNS: usize = 9;
const DEFAULT_WARMUP_RUNS: usize = 2;
const DEFAULT_LOOP_TRIPS: usize = 2_048;
const DEFAULT_FRONTEND_REPETITIONS: usize = 24;

type BenchResult<T> = Result<T, String>;

struct Options {
    report_path: String,
    samples: usize,
    runs_per_sample: usize,
    warmup_runs: usize,
    loop_trip_count: usize,
    frontend_repetitions: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WorkloadPipeline {
    ScriptRuntime,
    ScriptFrontend,
    ModuleCompile,
}

impl WorkloadPipeline {
    const fn label(self) -> &'static str {
        match self {
            Self::ScriptRuntime => "script.runtime",
            Self::ScriptFrontend => "script.frontend",
            Self::ModuleCompile => "module.compile",
        }
    }
}

#[derive(Clone)]
struct Workload {
    name: &'static str,
    pipeline: WorkloadPipeline,
    note: &'static str,
    source: String,
    operations_per_run: usize,
}

#[derive(Clone)]
struct ThroughputResult {
    samples: usize,
    runs_per_sample: usize,
    operations_per_run: usize,
    median_total: Duration,
    median_us_per_run: f64,
    median_ns_per_operation: f64,
}

#[derive(Clone, Default)]
struct MemoryResult {
    functions: Option<usize>,
    encoded_bytes: Option<usize>,
    metadata_records: Option<usize>,
    template_bytes: Option<usize>,
    atom_payload_bytes: usize,
    feedback_slots: Option<usize>,
    live_feedback_sites: Option<usize>,
    allocated_feedback_code_count: Option<usize>,
    allocated_feedback_bytes: Option<usize>,
    note: &'static str,
}

#[derive(Clone)]
struct WorkloadReport {
    workload: Workload,
    throughput: ThroughputResult,
    memory: MemoryResult,
}

#[derive(Clone, Copy)]
struct SampleResult {
    elapsed: Duration,
}

#[derive(Clone, Copy, Default)]
struct FeedbackTotals {
    slot_count: usize,
    live_site_count: usize,
    allocated_code_count: usize,
    allocated_bytes: usize,
}

#[derive(Clone)]
struct RuntimeSnapshot {
    label: &'static str,
    accounting: RuntimeAccounting,
    note: &'static str,
}

/// Runs the runtime benchmark suite and writes a Markdown report.
///
/// # Errors
///
/// Returns an error if the command-line arguments are invalid.
///
pub fn run(args: &[String]) -> Result<(), String> {
    let options = parse_options(args)?;

    if cfg!(debug_assertions) {
        eprintln!("warning: build with --release for meaningful measurements");
    }

    let mut reports = build_workloads(options.loop_trip_count, options.frontend_repetitions)
        .into_iter()
        .enumerate()
        .map(|(index, workload)| {
            let source_id = SourceId::new(
                u32::try_from(index + 1)
                    .map_err(|_| "runtime workload count exceeds SourceId range".to_string())?,
            );
            measure_workload(source_id, workload, &options)
        })
        .collect::<BenchResult<Vec<_>>>()?;
    reports.sort_by(|left, right| left.workload.name.cmp(right.workload.name));

    let snapshots = capture_runtime_snapshots()?;
    let report = render_report(&options, &reports, &snapshots)?;
    write_report(&options.report_path, &report)?;
    print_summary(&options.report_path, &reports, &snapshots)?;
    Ok(())
}

fn parse_options(args: &[String]) -> Result<Options, String> {
    let mut options = Options {
        report_path: DEFAULT_REPORT_PATH.to_string(),
        samples: DEFAULT_SAMPLES,
        runs_per_sample: DEFAULT_RUNS,
        warmup_runs: DEFAULT_WARMUP_RUNS,
        loop_trip_count: DEFAULT_LOOP_TRIPS,
        frontend_repetitions: DEFAULT_FRONTEND_REPETITIONS,
    };

    let mut args = args.iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--report" => {
                options.report_path = args.next().map_or_else(
                    || Err("--report requires a path".to_string()),
                    |value| Ok(value.clone()),
                )?;
            }
            "--samples" => {
                options.samples = parse_usize_arg("--samples", args.next())?;
            }
            "--runs" => {
                options.runs_per_sample = parse_usize_arg("--runs", args.next())?;
            }
            "--warmup-runs" => {
                options.warmup_runs = parse_usize_arg("--warmup-runs", args.next())?;
            }
            "--loop-trips" => {
                options.loop_trip_count = parse_usize_arg("--loop-trips", args.next())?;
            }
            "--frontend-repetitions" => {
                options.frontend_repetitions =
                    parse_usize_arg("--frontend-repetitions", args.next())?;
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
    if options.runs_per_sample == 0 {
        return Err("--runs must be greater than zero".to_string());
    }
    if options.warmup_runs == 0 {
        return Err("--warmup-runs must be greater than zero".to_string());
    }
    if options.loop_trip_count == 0 {
        return Err("--loop-trips must be greater than zero".to_string());
    }
    if options.frontend_repetitions == 0 {
        return Err("--frontend-repetitions must be greater than zero".to_string());
    }

    Ok(options)
}

fn usage() -> String {
    "Usage: lyng-js-bench runtime [--report <path>] [--samples <n>] [--runs <n>] [--warmup-runs <n>] [--loop-trips <n>] [--frontend-repetitions <n>]".to_string()
}

fn parse_usize_arg(flag: &str, value: Option<&String>) -> Result<usize, String> {
    value
        .ok_or_else(|| format!("{flag} requires a value"))?
        .parse()
        .map_err(|_| format!("{flag} expects a positive integer"))
}

fn build_workloads(loop_trip_count: usize, frontend_repetitions: usize) -> Vec<Workload> {
    vec![
        Workload {
            name: "array-heavy.literal-indexed-runtime",
            pipeline: WorkloadPipeline::ScriptRuntime,
            note: "Array-literal creation plus repeated dense indexed reads and writes on the current runtime path.",
            source: array_heavy_runtime_workload(loop_trip_count),
            operations_per_run: loop_trip_count,
        },
        Workload {
            name: "array-heavy.iterator-runtime",
            pipeline: WorkloadPipeline::ScriptRuntime,
            note: "Array for-of traversal plus iterator-driven array-literal and call spread on the current iterator runtime path.",
            source: array_iterator_runtime_workload(loop_trip_count),
            operations_per_run: loop_trip_count,
        },
        Workload {
            name: "async-heavy.frontend",
            pipeline: WorkloadPipeline::ScriptFrontend,
            note: "Async functions, async generators, and await-heavy syntax through the current frontend-only async benchmark surface.",
            source: async_heavy_frontend_workload(frontend_repetitions),
            operations_per_run: frontend_repetitions,
        },
        Workload {
            name: "class-heavy.runtime",
            pipeline: WorkloadPipeline::ScriptRuntime,
            note: "Class construction, instance and static private state, static blocks, and super dispatch on the current class runtime path.",
            source: class_heavy_runtime_workload(loop_trip_count),
            operations_per_run: loop_trip_count,
        },
        Workload {
            name: "module-heavy.placeholder-compile",
            pipeline: WorkloadPipeline::ModuleCompile,
            note: "Static import/export-heavy module sources through the current placeholder compile_module path.",
            source: module_heavy_compile_workload(frontend_repetitions),
            operations_per_run: frontend_repetitions,
        },
        Workload {
            name: "string-heavy.concat-runtime",
            pipeline: WorkloadPipeline::ScriptRuntime,
            note: "Repeated string concatenation and equality checks on the current runtime path.",
            source: string_heavy_runtime_workload(loop_trip_count),
            operations_per_run: loop_trip_count,
        },
        Workload {
            name: "regexp-heavy.runtime",
            pipeline: WorkloadPipeline::ScriptRuntime,
            note: "Repeated global exec, sticky matches, and named-capture replacement on the current RegExp runtime path.",
            source: regexp_heavy_runtime_workload(loop_trip_count),
            operations_per_run: loop_trip_count,
        },
        Workload {
            name: "regexp-constructor-compile.runtime",
            pipeline: WorkloadPipeline::ScriptRuntime,
            note: "Repeated RegExp constructor compilation over a small rotating pattern set.",
            source: regexp_constructor_compile_runtime_workload(loop_trip_count),
            operations_per_run: loop_trip_count,
        },
        Workload {
            name: "regexp-named-replace.runtime",
            pipeline: WorkloadPipeline::ScriptRuntime,
            note: "Global named-capture replacement without the broader mixed RegExp-heavy workload.",
            source: regexp_named_replace_runtime_workload(loop_trip_count),
            operations_per_run: loop_trip_count,
        },
        Workload {
            name: "regexp-legacy-statics.runtime",
            pipeline: WorkloadPipeline::ScriptRuntime,
            note: "Successful matches followed by Annex B RegExp legacy static accessor reads.",
            source: regexp_legacy_static_runtime_workload(loop_trip_count),
            operations_per_run: loop_trip_count,
        },
        Workload {
            name: "regexp-stable-exec.runtime",
            pipeline: WorkloadPipeline::ScriptRuntime,
            note: "Repeated default exec/test over stable Latin-1, UTF-16, astral, and lone-surrogate input strings.",
            source: regexp_stable_exec_runtime_workload(loop_trip_count),
            operations_per_run: loop_trip_count,
        },
        Workload {
            name: "typed-array-heavy.runtime",
            pipeline: WorkloadPipeline::ScriptRuntime,
            note: "ArrayBuffer-backed typed-array views plus DataView read/write traffic on the current binary-data runtime path.",
            source: typed_array_heavy_runtime_workload(loop_trip_count),
            operations_per_run: loop_trip_count,
        },
    ]
}

fn measure_workload(
    source_id: SourceId,
    workload: Workload,
    options: &Options,
) -> BenchResult<WorkloadReport> {
    let throughput = measure_throughput(source_id, &workload, options)?;
    let memory = capture_memory(source_id, &workload, options)?;
    Ok(WorkloadReport {
        workload,
        throughput,
        memory,
    })
}

fn measure_throughput(
    source_id: SourceId,
    workload: &Workload,
    options: &Options,
) -> BenchResult<ThroughputResult> {
    let samples = match workload.pipeline {
        WorkloadPipeline::ScriptRuntime => {
            measure_script_runtime_samples(source_id, workload, options)?
        }
        WorkloadPipeline::ScriptFrontend => {
            measure_script_frontend_samples(source_id, workload, options)?
        }
        WorkloadPipeline::ModuleCompile => {
            measure_module_compile_samples(source_id, workload, options)?
        }
    };

    throughput_result(options, workload, &samples)
}

fn measure_script_runtime_samples(
    source_id: SourceId,
    workload: &Workload,
    options: &Options,
) -> BenchResult<Vec<SampleResult>> {
    let mut atoms = AtomTable::new();
    let unit = compile_script_unit(source_id, &workload.source, &mut atoms)?;
    let mut samples = Vec::with_capacity(options.samples);
    for _ in 0..options.samples {
        samples.push(measure_script_runtime_sample(workload, options, &unit)?);
    }
    Ok(samples)
}

fn measure_script_runtime_sample(
    workload: &Workload,
    options: &Options,
    unit: &CompiledScriptUnit,
) -> BenchResult<SampleResult> {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent
        .default_realm()
        .ok_or_else(|| "default realm should exist for runtime benchmark".to_string())?;
    let mut vm = Vm::new();
    let _ = vm
        .bootstrap_realm(agent, realm.id(), BootstrapMode::SpecOnly)
        .map_err(|error| format!("spec bootstrap failed for runtime benchmark: {error:?}"))?;
    let installed = vm
        .install_script(agent, realm.id(), unit)
        .map_err(|error| format!("script install failed for {}: {error:?}", workload.name))?;

    for _ in 0..options.warmup_runs {
        let value = vm
            .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .map_err(|error| format!("warmup execution failed for {}: {error:?}", workload.name))?;
        black_box(value.bits());
    }

    let start = Instant::now();
    let mut checksum = 0_u64;
    for _ in 0..options.runs_per_sample {
        let value = black_box(
            vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
                .map_err(|error| {
                    format!("timed execution failed for {}: {error:?}", workload.name)
                })?,
        );
        checksum = checksum.wrapping_add(black_box(value.bits()));
    }
    black_box(checksum);
    Ok(SampleResult {
        elapsed: start.elapsed(),
    })
}

fn measure_script_frontend_samples(
    source_id: SourceId,
    workload: &Workload,
    options: &Options,
) -> BenchResult<Vec<SampleResult>> {
    let mut samples = Vec::with_capacity(options.samples);
    for _ in 0..options.samples {
        for _ in 0..options.warmup_runs {
            check_frontend_workload(source_id, workload)?;
        }

        let start = Instant::now();
        for _ in 0..options.runs_per_sample {
            check_frontend_workload(source_id, workload)?;
        }
        samples.push(SampleResult {
            elapsed: start.elapsed(),
        });
    }
    Ok(samples)
}

fn check_frontend_workload(source_id: SourceId, workload: &Workload) -> BenchResult<()> {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(&mut atoms, source_id, &workload.source);
    if parsed.diagnostics.has_errors() {
        return Err(format!(
            "parse errors in {}: {:?}",
            workload.name,
            parsed.diagnostics.as_slice()
        ));
    }
    let sema = analyze_script(&parsed, &atoms);
    if sema.diagnostics.has_errors() {
        return Err(format!(
            "sema errors in {}: {:?}",
            workload.name,
            sema.diagnostics.as_slice()
        ));
    }
    black_box(parsed.root.raw());
    Ok(())
}

fn measure_module_compile_samples(
    source_id: SourceId,
    workload: &Workload,
    options: &Options,
) -> BenchResult<Vec<SampleResult>> {
    let mut samples = Vec::with_capacity(options.samples);
    for _ in 0..options.samples {
        for _ in 0..options.warmup_runs {
            let mut atoms = AtomTable::new();
            let _ = compile_module_unit(source_id, &workload.source, &mut atoms)?;
        }

        let start = Instant::now();
        for _ in 0..options.runs_per_sample {
            let mut atoms = AtomTable::new();
            let unit = compile_module_unit(source_id, &workload.source, &mut atoms)?;
            black_box(unit.entry());
        }
        samples.push(SampleResult {
            elapsed: start.elapsed(),
        });
    }
    Ok(samples)
}

fn throughput_result(
    options: &Options,
    workload: &Workload,
    samples: &[SampleResult],
) -> BenchResult<ThroughputResult> {
    let median_total = median_duration(samples.iter().map(|sample| sample.elapsed).collect());
    let runs_per_sample = u32::try_from(options.runs_per_sample)
        .map_err(|_| "--runs exceeds runtime benchmark reporting range".to_string())?;
    let median_us_per_run =
        duration_seconds(median_total) * 1_000_000.0 / f64::from(runs_per_sample);
    let total_operations = options
        .runs_per_sample
        .checked_mul(workload.operations_per_run)
        .ok_or_else(|| "runtime benchmark operation count overflowed".to_string())?;
    let total_operations_for_report = u32::try_from(total_operations)
        .map_err(|_| "runtime benchmark operation count exceeds reporting range".to_string())?;
    let median_ns_per_operation =
        duration_seconds(median_total) * 1_000_000_000.0 / f64::from(total_operations_for_report);

    Ok(ThroughputResult {
        samples: options.samples,
        runs_per_sample: options.runs_per_sample,
        operations_per_run: workload.operations_per_run,
        median_total,
        median_us_per_run,
        median_ns_per_operation,
    })
}

fn capture_memory(
    source_id: SourceId,
    workload: &Workload,
    options: &Options,
) -> BenchResult<MemoryResult> {
    match workload.pipeline {
        WorkloadPipeline::ScriptRuntime => {
            let mut atoms = AtomTable::new();
            let unit = compile_script_unit(source_id, &workload.source, &mut atoms)?;
            let atom_payload_bytes = compiled_unit_atom_payload_bytes(&unit);

            let mut runtime = Runtime::new(NoopHostHooks);
            let agent = runtime.root_agent_mut();
            let realm = agent
                .default_realm()
                .ok_or_else(|| "default realm should exist for memory capture".to_string())?;
            let mut vm = Vm::new();
            let _ = vm
                .bootstrap_realm(agent, realm.id(), BootstrapMode::SpecOnly)
                .map_err(|error| format!("spec bootstrap failed for memory capture: {error:?}"))?;
            let installed = vm
                .install_script(agent, realm.id(), &unit)
                .map_err(|error| {
                    format!("script install failed for {}: {error:?}", workload.name)
                })?;
            for _ in 0..options.warmup_runs {
                let value = vm
                    .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
                    .map_err(|error| {
                        format!(
                            "memory warmup execution failed for {}: {error:?}",
                            workload.name
                        )
                    })?;
                black_box(value.bits());
            }
            let feedback = collect_feedback_totals(&vm, installed.code())?;

            Ok(MemoryResult {
                functions: Some(unit.functions().len()),
                encoded_bytes: Some(compiled_unit_encoded_bytes(&unit)),
                metadata_records: Some(compiled_unit_metadata_records(&unit)),
                template_bytes: Some(compiled_unit_template_bytes(&unit)),
                atom_payload_bytes,
                feedback_slots: Some(feedback.slot_count),
                live_feedback_sites: Some(feedback.live_site_count),
                allocated_feedback_code_count: Some(feedback.allocated_code_count),
                allocated_feedback_bytes: Some(feedback.allocated_bytes),
                note: "Warmed script-template and feedback-vector footprint.",
            })
        }
        WorkloadPipeline::ScriptFrontend => {
            let mut atoms = AtomTable::new();
            let parsed = parse_script(&mut atoms, source_id, &workload.source);
            if parsed.diagnostics.has_errors() {
                return Err(format!(
                    "parse errors in {}: {:?}",
                    workload.name,
                    parsed.diagnostics.as_slice()
                ));
            }
            let sema = analyze_script(&parsed, &atoms);
            if sema.diagnostics.has_errors() {
                return Err(format!(
                    "sema errors in {}: {:?}",
                    workload.name,
                    sema.diagnostics.as_slice()
                ));
            }
            black_box(parsed.root.raw());
            Ok(MemoryResult {
                atom_payload_bytes: atom_payload_bytes(&atoms),
                note: "Frontend-only source and atom surface for the current async benchmark row.",
                ..MemoryResult::default()
            })
        }
        WorkloadPipeline::ModuleCompile => {
            let mut atoms = AtomTable::new();
            let unit = compile_module_unit(source_id, &workload.source, &mut atoms)?;
            black_box(unit.entry());
            Ok(MemoryResult {
                encoded_bytes: Some(0),
                metadata_records: Some(0),
                template_bytes: Some(size_of::<CompiledModuleUnit>()),
                atom_payload_bytes: atom_payload_bytes(&atoms),
                note: "Placeholder module compile unit plus atom payload bytes.",
                ..MemoryResult::default()
            })
        }
    }
}

fn capture_runtime_snapshots() -> BenchResult<Vec<RuntimeSnapshot>> {
    let empty_runtime = Runtime::new(NoopHostHooks);
    let empty = RuntimeSnapshot {
        label: "runtime.empty",
        accounting: empty_runtime.phase6_accounting(),
        note: "Fresh runtime with the default root-agent shell only.",
    };

    let spec_bootstrapped = {
        let mut runtime = Runtime::new(NoopHostHooks);
        let agent = runtime.root_agent_mut();
        let realm = agent
            .default_realm()
            .ok_or_else(|| "default realm should exist for spec-bootstrap snapshot".to_string())?;
        let mut vm = Vm::new();
        let _ = vm
            .bootstrap_realm(agent, realm.id(), BootstrapMode::SpecOnly)
            .map_err(|error| format!("spec bootstrap snapshot failed: {error:?}"))?;
        RuntimeSnapshot {
            label: "runtime.spec-bootstrap",
            accounting: runtime.phase6_accounting(),
            note: "Spec-only default-realm bootstrap baseline.",
        }
    };

    let regexp_literal_cache = {
        let mut atoms = AtomTable::new();
        let unit = compile_script_unit(
            SourceId::new(90),
            r"
            function make() {
                return /cache/g;
            }
            make();
            make();
            make();
            ",
            &mut atoms,
        )?;
        let mut runtime = Runtime::new(NoopHostHooks);
        {
            let agent = runtime.root_agent_mut();
            let realm = agent.default_realm().ok_or_else(|| {
                "default realm should exist for RegExp literal-cache snapshot".to_string()
            })?;
            let mut vm = Vm::new();
            let value = vm.evaluate_script(agent, realm, &unit).map_err(|error| {
                format!("RegExp literal-cache snapshot execution failed: {error:?}")
            })?;
            black_box(value.bits());
        }
        RuntimeSnapshot {
            label: "runtime.regexp-literal-cache",
            accounting: runtime.phase6_accounting(),
            note: "Executed repeated RegExp literal evaluations so retained compiled literal payload cache accounting is visible. RegExp payload bytes are a lower-bound estimate because backend-owned regex tables are opaque.",
        }
    };

    let promise_and_backing_store = {
        let mut runtime = Runtime::new(NoopHostHooks);
        let root = runtime.root_agent_id();
        let worker = runtime
            .root_cluster_mut()
            .add_agent(None, Some("bench-worker".into()));
        let shared_buffer = HostSharedBufferId::from_raw(19)
            .ok_or_else(|| "shared buffer fixture id must be non-zero".to_string())?;
        let backing_store = runtime
            .root_cluster_mut()
            .register_shared_backing_store(root, 4096)
            .ok_or_else(|| "shared backing-store fixture failed to register".to_string())?;

        if !runtime
            .root_cluster_mut()
            .cache_shared_backing_store_handle(backing_store, shared_buffer)
        {
            return Err("shared backing-store fixture failed to cache host handle".to_string());
        }
        if !runtime
            .root_cluster_mut()
            .share_shared_backing_store(backing_store, worker)
        {
            return Err("shared backing-store fixture failed to share with worker".to_string());
        }
        runtime
            .enqueue_job(
                root,
                HostJobKind::Promise,
                ExecutableId::Builtin,
                None,
                Some("bench-promise-job".into()),
            )
            .map_err(|error| format!("promise job fixture failed to enqueue: {error:?}"))?;

        RuntimeSnapshot {
            label: "runtime.promise-and-backing-store",
            accounting: runtime.phase6_accounting(),
            note: "Seeded promise-job queue entry plus one shared backing-store fixture. Iterator state remains transient VM execution state, so the post-run iterator and module-cache domains stay at zero in this retained-runtime snapshot.",
        }
    };

    Ok(vec![
        empty,
        spec_bootstrapped,
        regexp_literal_cache,
        promise_and_backing_store,
    ])
}

fn compile_script_unit(
    source_id: SourceId,
    source: &str,
    atoms: &mut AtomTable,
) -> BenchResult<CompiledScriptUnit> {
    let parsed = parse_script(atoms, source_id, source);
    if parsed.diagnostics.has_errors() {
        return Err(format!(
            "parse errors in benchmark workload: {:?}",
            parsed.diagnostics.as_slice()
        ));
    }
    let sema = analyze_script(&parsed, atoms);
    if sema.diagnostics.has_errors() {
        return Err(format!(
            "sema errors in benchmark workload: {:?}",
            sema.diagnostics.as_slice()
        ));
    }
    compile_script(&parsed, &sema, atoms)
        .map_err(|error| format!("benchmark workload failed to lower: {error:?}"))
}

fn compile_module_unit(
    source_id: SourceId,
    source: &str,
    atoms: &mut AtomTable,
) -> BenchResult<CompiledModuleUnit> {
    let parsed = parse_module(atoms, source_id, source);
    if parsed.diagnostics.has_errors() {
        return Err(format!(
            "parse errors in module benchmark workload: {:?}",
            parsed.diagnostics.as_slice()
        ));
    }
    let sema = analyze_module(&parsed, atoms);
    if sema.diagnostics.has_errors() {
        return Err(format!(
            "sema errors in module benchmark workload: {:?}",
            sema.diagnostics.as_slice()
        ));
    }
    compile_module(&parsed, &sema, atoms)
        .map_err(|error| format!("module benchmark workload failed to compile: {error:?}"))
}

fn collect_feedback_totals(vm: &Vm, root: CodeRef) -> BenchResult<FeedbackTotals> {
    let mut totals = FeedbackTotals::default();
    let mut stack = vec![root];

    while let Some(code) = stack.pop() {
        let Some(function) = vm.installed_function(code) else {
            continue;
        };
        if let Some(footprint) = vm.feedback_vector_footprint(code) {
            totals.slot_count = totals.slot_count.saturating_add(footprint.slot_count());
            totals.live_site_count = totals
                .live_site_count
                .saturating_add(footprint.live_site_count());
            totals.allocated_bytes = totals
                .allocated_bytes
                .saturating_add(footprint.allocated_bytes());
            if footprint.allocated() {
                totals.allocated_code_count = totals.allocated_code_count.saturating_add(1);
            }
        }
        for child_index in 0..function.child_functions().len() {
            let child_index = u32::try_from(child_index)
                .map_err(|_| "child function count exceeds installed-code range".to_string())?;
            if let Some(child_code) = vm.installed_child_code(code, child_index) {
                stack.push(child_code);
            }
        }
    }

    Ok(totals)
}

fn compiled_unit_encoded_bytes(unit: &CompiledScriptUnit) -> usize {
    unit.functions()
        .iter()
        .map(|function| function.instructions().len() + function.wide_operands().len())
        .sum::<usize>()
        .saturating_mul(4)
}

fn compiled_unit_metadata_records(unit: &CompiledScriptUnit) -> usize {
    unit.functions()
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
        .sum()
}

fn compiled_unit_template_bytes(unit: &CompiledScriptUnit) -> usize {
    size_of::<CompiledScriptUnit>()
        + unit
            .functions()
            .iter()
            .map(function_template_bytes)
            .sum::<usize>()
        + size_of_val(unit.atoms())
        + compiled_unit_atom_payload_bytes(unit)
}

fn compiled_unit_atom_payload_bytes(unit: &CompiledScriptUnit) -> usize {
    unit.atoms()
        .iter()
        .map(|(_, atom)| match atom {
            CompiledAtom::Utf8(text) => text.len(),
            CompiledAtom::Utf16(units) => units.len().saturating_mul(size_of::<u16>()),
        })
        .sum()
}

const fn atom_payload_bytes(atoms: &AtomTable) -> usize {
    atoms.payload_bytes()
}

fn function_template_bytes(function: &BytecodeFunction) -> usize {
    size_of::<BytecodeFunction>()
        + size_of_val(function.instructions())
        + size_of_val(function.constants())
        + size_of_val(function.child_functions())
        + size_of_val(function.captures())
        + size_of_val(function.exception_handlers())
        + size_of_val(function.feedback_sites())
        + size_of_val(function.source_map())
        + size_of_val(function.wide_operands())
        + size_of_val(function.safepoints())
        + size_of_val(function.deopt_snapshots())
}

struct RuntimeWatchItems<'a> {
    dense_array_runtime: &'a WorkloadReport,
    iterator_array_runtime: &'a WorkloadReport,
    string_runtime: &'a WorkloadReport,
    regexp_runtime: &'a WorkloadReport,
    regexp_constructor_runtime: &'a WorkloadReport,
    regexp_replace_runtime: &'a WorkloadReport,
    regexp_legacy_static_runtime: &'a WorkloadReport,
    regexp_stable_exec_runtime: &'a WorkloadReport,
    class_runtime: &'a WorkloadReport,
    typed_array_runtime: &'a WorkloadReport,
    seeded_snapshot: &'a RuntimeSnapshot,
}

impl<'a> RuntimeWatchItems<'a> {
    fn collect(
        reports: &'a [WorkloadReport],
        snapshots: &'a [RuntimeSnapshot],
    ) -> BenchResult<Self> {
        let seeded_snapshot = snapshots
            .iter()
            .find(|snapshot| snapshot.label == "runtime.promise-and-backing-store")
            .ok_or_else(|| "runtime.promise-and-backing-store snapshot is missing".to_string())?;

        Ok(Self {
            dense_array_runtime: report_by_name(reports, "array-heavy.literal-indexed-runtime")?,
            iterator_array_runtime: report_by_name(reports, "array-heavy.iterator-runtime")?,
            string_runtime: report_by_name(reports, "string-heavy.concat-runtime")?,
            regexp_runtime: report_by_name(reports, "regexp-heavy.runtime")?,
            regexp_constructor_runtime: report_by_name(
                reports,
                "regexp-constructor-compile.runtime",
            )?,
            regexp_replace_runtime: report_by_name(reports, "regexp-named-replace.runtime")?,
            regexp_legacy_static_runtime: report_by_name(reports, "regexp-legacy-statics.runtime")?,
            regexp_stable_exec_runtime: report_by_name(reports, "regexp-stable-exec.runtime")?,
            class_runtime: report_by_name(reports, "class-heavy.runtime")?,
            typed_array_runtime: report_by_name(reports, "typed-array-heavy.runtime")?,
            seeded_snapshot,
        })
    }
}

fn render_report(
    options: &Options,
    reports: &[WorkloadReport],
    snapshots: &[RuntimeSnapshot],
) -> BenchResult<String> {
    let mut output = String::new();
    let command = format!(
        "cargo run --release -p lyng-js-bench -- runtime --report {}",
        options.report_path
    );
    let watch_items = RuntimeWatchItems::collect(reports, snapshots)?;

    write_runtime_report_intro(&mut output, options, &command);
    write_workload_throughput_section(&mut output, reports);
    write_template_feedback_section(&mut output, reports);
    write_runtime_accounting_section(&mut output, snapshots);
    write_watch_items_section(&mut output, &watch_items);
    write_known_gaps_section(&mut output);

    Ok(output)
}

fn write_runtime_report_intro(output: &mut String, options: &Options, command: &str) {
    let _ = writeln!(output, "# Lyng JS Benchmarks and Memory Report");
    output.push('\n');
    let _ = writeln!(output, "This report is generated by `{command}`.");
    let _ = writeln!(
        output,
        "It covers the current Lyng JS runtime, frontend, and memory benchmark surface with dense-indexed, iterator-driven, string-heavy, RegExp-heavy, class-heavy, and typed-array-heavy runtime baselines, plus the remaining async-heavy and module-heavy non-runtime rows. Executable workload rows sit alongside retained runtime-accounting snapshots."
    );
    output.push('\n');

    let _ = writeln!(output, "## Settings");
    output.push('\n');
    let _ = writeln!(output, "- Profile: `release`");
    let _ = writeln!(output, "- Target OS: `{}`", env::consts::OS);
    let _ = writeln!(output, "- Target architecture: `{}`", env::consts::ARCH);
    let _ = writeln!(output, "- Samples per benchmark: `{}`", options.samples);
    let _ = writeln!(
        output,
        "- Warmup runs per sample: `{}`",
        options.warmup_runs
    );
    let _ = writeln!(
        output,
        "- Timed runs per sample: `{}`",
        options.runs_per_sample
    );
    let _ = writeln!(
        output,
        "- Runtime loop trips: `{}`",
        options.loop_trip_count
    );
    let _ = writeln!(
        output,
        "- Frontend repetition count: `{}`",
        options.frontend_repetitions
    );
    output.push('\n');
}

fn write_workload_throughput_section(output: &mut String, reports: &[WorkloadReport]) {
    let _ = writeln!(output, "## Workload Throughput");
    output.push('\n');
    let _ = writeln!(
        output,
        "| Benchmark | Pipeline | Samples | Runs/sample | Work units/run | Median total | Median us/run | Median ns/work-unit | Note |"
    );
    let _ = writeln!(
        output,
        "| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |"
    );
    for report in reports {
        let _ = writeln!(
            output,
            "| `{}` | `{}` | `{}` | `{}` | `{}` | `{}` | `{:.2}` | `{:.2}` | {} |",
            report.workload.name,
            report.workload.pipeline.label(),
            report.throughput.samples,
            report.throughput.runs_per_sample,
            report.throughput.operations_per_run,
            format_duration(report.throughput.median_total),
            report.throughput.median_us_per_run,
            report.throughput.median_ns_per_operation,
            report.workload.note,
        );
    }
    output.push('\n');
}

fn write_template_feedback_section(output: &mut String, reports: &[WorkloadReport]) {
    let _ = writeln!(output, "## Template and Feedback Memory");
    output.push('\n');
    let _ = writeln!(
        output,
        "| Benchmark | Pipeline | Functions | Encoded bytes | Metadata records | Template bytes | Atom payload bytes | Feedback slots | Live sites | Feedback codes | Allocated feedback bytes | Memory note |"
    );
    let _ = writeln!(
        output,
        "| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |"
    );
    for report in reports {
        let _ = writeln!(
            output,
            "| `{}` | `{}` | {} | {} | {} | {} | `{}` | {} | {} | {} | {} | {} |",
            report.workload.name,
            report.workload.pipeline.label(),
            opt_usize_cell(report.memory.functions),
            opt_usize_cell(report.memory.encoded_bytes),
            opt_usize_cell(report.memory.metadata_records),
            opt_usize_cell(report.memory.template_bytes),
            report.memory.atom_payload_bytes,
            opt_usize_cell(report.memory.feedback_slots),
            opt_usize_cell(report.memory.live_feedback_sites),
            opt_usize_cell(report.memory.allocated_feedback_code_count),
            opt_usize_cell(report.memory.allocated_feedback_bytes),
            report.memory.note,
        );
    }
    output.push('\n');
}

fn write_runtime_accounting_section(output: &mut String, snapshots: &[RuntimeSnapshot]) {
    let _ = writeln!(output, "## Runtime Accounting Snapshots");
    output.push('\n');
    let _ = writeln!(
        output,
        "| Snapshot | Heap live bytes | Heap reserved bytes | Iterator records | RegExp payloads | RegExp literal cache | Module caches | Promise jobs | Backing stores | Total live bytes | Note |"
    );
    let _ = writeln!(
        output,
        "| --- | ---: | ---: | --- | --- | --- | --- | --- | --- | ---: | --- |"
    );
    for snapshot in snapshots {
        let _ = writeln!(
            output,
            "| `{}` | `{}` | `{}` | `{}` | `{}` | `{}` | `{}` | `{}` | `{}` | `{}` | {} |",
            snapshot.label,
            snapshot.accounting.heap.live_bytes,
            snapshot.accounting.heap.reserved_bytes,
            domain_cell(snapshot.accounting.iterator_records),
            domain_cell(snapshot.accounting.regexp_payloads),
            domain_cell(snapshot.accounting.regexp_literal_cache),
            domain_cell(snapshot.accounting.module_caches),
            domain_cell(snapshot.accounting.promise_jobs),
            domain_cell(snapshot.accounting.backing_stores),
            snapshot.accounting.live_bytes,
            snapshot.note,
        );
    }
    output.push('\n');
}

fn write_watch_items_section(output: &mut String, items: &RuntimeWatchItems<'_>) {
    let _ = writeln!(output, "## Watch Items");
    output.push('\n');
    let _ = writeln!(output, "- The executable array runtime baselines now include both dense-indexed and iterator-driven rows. On this run, `array-heavy.literal-indexed-runtime` measured `{:.2}` ns/work-unit and `array-heavy.iterator-runtime` measured `{:.2}` ns/work-unit.",
        items.dense_array_runtime.throughput.median_ns_per_operation,
        items.iterator_array_runtime.throughput.median_ns_per_operation,
    );
    let _ = writeln!(output, "- The iterator-driven row warmed `{}` feedback slots across `{}` live sites with `{}` template bytes, so the benchmark captures real iterator lowering and VM state rather than only the older dense-element hot path.",
        opt_usize_text(items.iterator_array_runtime.memory.feedback_slots),
        opt_usize_text(items.iterator_array_runtime.memory.live_feedback_sites),
        opt_usize_text(items.iterator_array_runtime.memory.template_bytes),
    );
    let _ = writeln!(output, "- `string-heavy.concat-runtime` remains the lower-level string/runtime proxy at `{:.2}` ns/work-unit.",
        items.string_runtime.throughput.median_ns_per_operation,
    );
    let _ = writeln!(output, "- `regexp-heavy.runtime` measured `{:.2}` ns/work-unit while exercising global iteration, sticky state, match indices, and named-capture replacement through the shared matcher path.",
        items.regexp_runtime.throughput.median_ns_per_operation,
    );
    let _ = writeln!(output, "- RegExp observability rows separate constructor compilation (`{:.2}` ns/work-unit), stable default exec/test (`{:.2}` ns/work-unit), named replacement (`{:.2}` ns/work-unit), and legacy static accessor reads (`{:.2}` ns/work-unit).",
        items.regexp_constructor_runtime
            .throughput
            .median_ns_per_operation,
        items.regexp_stable_exec_runtime
            .throughput
            .median_ns_per_operation,
        items.regexp_replace_runtime.throughput.median_ns_per_operation,
        items.regexp_legacy_static_runtime
            .throughput
            .median_ns_per_operation,
    );
    let _ = writeln!(output, "- `class-heavy.runtime` measured `{:.2}` ns/work-unit while warming `{}` feedback slots across `{}` live sites with `{}` template bytes, covering private fields, static blocks, and `super` dispatch on the executable runtime path.",
        items.class_runtime.throughput.median_ns_per_operation,
        opt_usize_text(items.class_runtime.memory.feedback_slots),
        opt_usize_text(items.class_runtime.memory.live_feedback_sites),
        opt_usize_text(items.class_runtime.memory.template_bytes),
    );
    let _ = writeln!(output, "- `async-heavy.frontend` remains the only frontend-only async workload in the current benchmark surface. That is now a benchmark-shape gap rather than a known lowering/runtime hole, so follow-up work should add an executable async runtime row."
    );
    let _ = writeln!(output, "- `typed-array-heavy.runtime` measured `{:.2}` ns/work-unit while warming `{}` feedback slots across `{}` live sites with `{}` template bytes, covering ArrayBuffer-backed views and DataView byte traffic on the executable runtime path.",
        items.typed_array_runtime.throughput.median_ns_per_operation,
        opt_usize_text(items.typed_array_runtime.memory.feedback_slots),
        opt_usize_text(items.typed_array_runtime.memory.live_feedback_sites),
        opt_usize_text(items.typed_array_runtime.memory.template_bytes),
    );
    let _ = writeln!(output, "- The seeded accounting snapshot reports `{}` promise job and `{}` backing store so the retained runtime-accounting surface is exercised by real data. Retained RegExp payloads report as a distinct runtime domain, with payload bytes treated as a lower-bound estimate because the current regex backend does not expose all internally owned tables. Iterator-heavy evidence still lives in the executable array/iterator workload rows because iterator state is transient VM execution state rather than a retained post-run runtime record.",
        items.seeded_snapshot.accounting.promise_jobs.records,
        items.seeded_snapshot.accounting.backing_stores.records,
    );
    output.push('\n');
}

fn write_known_gaps_section(output: &mut String) {
    let _ = writeln!(output, "## Known Gaps");
    output.push('\n');
    let _ = writeln!(output, "- This report complements the dedicated bytecode-density suite rather than replacing it. The density run remains the finer-grained instruction-shape view."
    );
    let _ = writeln!(output, "- The remaining frontend-only row does not report AST or sema heap residency. Its current memory row is limited to atom payload and the explicit absence of code-template or feedback state."
    );
    let _ = writeln!(output, "- Module-cache accounting remains a future retained-runtime domain. Post-run iterator-record rows are still zero because the current iterator state is transient to active VM execution, so the benchmarked array/iterator runtime rows are the authoritative memory/perf signal for that surface."
    );
    let _ = writeln!(output, "- RegExp payload accounting is currently a lower bound: it includes source text, retained UTF-16 source units, flag text, and the `regress::Regex` struct, but not the backend's private instruction vectors, class tables, and capture metadata allocations."
    );
}

fn report_by_name<'a>(
    reports: &'a [WorkloadReport],
    name: &str,
) -> BenchResult<&'a WorkloadReport> {
    reports
        .iter()
        .find(|report| report.workload.name == name)
        .ok_or_else(|| format!("required runtime benchmark row `{name}` is missing"))
}

fn opt_usize_cell(value: Option<usize>) -> String {
    value.map_or_else(|| "n/a".to_string(), |value| format!("`{value}`"))
}

fn opt_usize_text(value: Option<usize>) -> String {
    value.map_or_else(|| "n/a".to_string(), |value| value.to_string())
}

fn domain_cell(accounting: lyng_js_env::RuntimeDomainAccounting) -> String {
    format!(
        "{} rec / {} meta / {} payload / {} live",
        accounting.records,
        accounting.metadata_bytes,
        accounting.payload_bytes,
        accounting.live_bytes
    )
}

fn median_duration(mut durations: Vec<Duration>) -> Duration {
    durations.sort_unstable();
    durations[durations.len() / 2]
}

const fn duration_seconds(duration: Duration) -> f64 {
    duration.as_secs_f64()
}

fn format_duration(duration: Duration) -> String {
    let nanos = duration.as_nanos();
    match nanos.cmp(&1_000) {
        Ordering::Less => format!("{nanos}ns"),
        Ordering::Equal | Ordering::Greater if nanos < 1_000_000 => {
            format!("{:.3}us", duration_seconds(duration) * 1_000_000.0)
        }
        _ if nanos < 1_000_000_000 => format!("{:.3}ms", duration_seconds(duration) * 1_000.0),
        _ => format!("{:.3}s", duration_seconds(duration)),
    }
}

fn write_report(path: &str, report: &str) -> BenchResult<()> {
    let report_path = Path::new(path);
    if let Some(parent) = report_path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "failed to create runtime benchmark report directory {}: {error}",
                parent.display()
            )
        })?;
    }
    fs::write(report_path, report).map_err(|error| {
        format!(
            "failed to write runtime benchmark report {}: {error}",
            report_path.display()
        )
    })
}

fn print_summary(
    path: &str,
    reports: &[WorkloadReport],
    snapshots: &[RuntimeSnapshot],
) -> BenchResult<()> {
    let slowest = reports
        .iter()
        .max_by(|left, right| {
            left.throughput
                .median_ns_per_operation
                .total_cmp(&right.throughput.median_ns_per_operation)
        })
        .ok_or_else(|| "runtime benchmark produced no workload rows".to_string())?;
    let largest_template = reports
        .iter()
        .filter_map(|report| report.memory.template_bytes.map(|bytes| (report, bytes)))
        .max_by_key(|(_, bytes)| *bytes)
        .ok_or_else(|| "runtime benchmark produced no template memory rows".to_string())?;
    let heaviest_snapshot = snapshots
        .iter()
        .max_by_key(|snapshot| snapshot.accounting.live_bytes)
        .ok_or_else(|| "runtime benchmark produced no accounting snapshots".to_string())?;

    println!("Wrote {path}");
    println!(
        "Slowest row: {} at {:.2} ns/work-unit",
        slowest.workload.name, slowest.throughput.median_ns_per_operation
    );
    println!(
        "Largest template footprint: {} at {} bytes",
        largest_template.0.workload.name, largest_template.1
    );
    println!(
        "Heaviest runtime snapshot: {} at {} live bytes",
        heaviest_snapshot.label, heaviest_snapshot.accounting.live_bytes
    );
    Ok(())
}

fn string_heavy_runtime_workload(loop_trip_count: usize) -> String {
    format!(
        r#"
        (function() {{
            function run(limit) {{
                var text = "aa";
                var hits = 0;
                var i = 0;
                while (i < limit) {{
                    text = text + "bc";
                    if (text === "aabcbcbcbcbc") {{
                        hits = hits + 1;
                        text = "aa";
                    }}
                    i = i + 1;
                }}
                return hits;
            }}
            return run({loop_trip_count});
        }})()
        "#
    )
}

fn regexp_heavy_runtime_workload(loop_trip_count: usize) -> String {
    format!(
        r#"
        (function() {{
            function run(limit) {{
                var source = "alpha-123 beta-456 gamma-789 delta-000";
                var execRe = /(?<word>[a-z]+)-(?<digits>\d+)/dg;
                var stickyRe = /[a-z]+/y;
                var total = 0;
                var i = 0;
                while (i < limit) {{
                    execRe.lastIndex = 0;
                    var match;
                    while ((match = execRe.exec(source)) !== null) {{
                        total = total + match[0].length + match.indices[1][0] + match.indices[2][1];
                    }}
                    stickyRe.lastIndex = 0;
                    while (stickyRe.test("alpha beta")) {{
                        total = total + stickyRe.lastIndex;
                        stickyRe.lastIndex = stickyRe.lastIndex + 1;
                    }}
                    total = total + source.replace(execRe, "$<digits>:$<word>").length;
                    i = i + 1;
                }}
                return total;
            }}
            return run({loop_trip_count});
        }})()
        "#
    )
}

fn regexp_constructor_compile_runtime_workload(loop_trip_count: usize) -> String {
    format!(
        r#"
        (function() {{
            function run(limit) {{
                var sources = ["^alpha-", "^beta-", "^(?<word>[a-z]+)-", "^[\\p{{ASCII}}]+-"];
                var flags = ["", "u", "dg", "u"];
                var samples = ["alpha-123", "beta-456", "gamma-789", "ASCII-000"];
                var total = 0;
                var i = 0;
                while (i < limit) {{
                    var slot = i & 3;
                    var re = new RegExp(sources[slot] + "\\d+$", flags[slot]);
                    if (re.test(samples[slot])) {{
                        total = total + re.lastIndex + slot + 1;
                    }}
                    i = i + 1;
                }}
                return total;
            }}
            return run({loop_trip_count});
        }})()
        "#
    )
}

fn regexp_named_replace_runtime_workload(loop_trip_count: usize) -> String {
    format!(
        r#"
        (function() {{
            function run(limit) {{
                var source = "alpha-123 beta-456 gamma-789 delta-000";
                var re = /(?<word>[a-z]+)-(?<digits>\d+)/g;
                var total = 0;
                var i = 0;
                while (i < limit) {{
                    total = total + source.replace(re, "$<digits>:$<word>").length;
                    i = i + 1;
                }}
                return total;
            }}
            return run({loop_trip_count});
        }})()
        "#
    )
}

fn regexp_legacy_static_runtime_workload(loop_trip_count: usize) -> String {
    format!(
        r#"
        (function() {{
            function run(limit) {{
                var re = /(alpha)-(\d+)/;
                var total = 0;
                var i = 0;
                while (i < limit) {{
                    re.exec("xx alpha-123 yy");
                    total = total + RegExp.$1.length + RegExp.$2.length + RegExp.lastMatch.length;
                    i = i + 1;
                }}
                return total;
            }}
            return run({loop_trip_count});
        }})()
        "#
    )
}

fn regexp_stable_exec_runtime_workload(loop_trip_count: usize) -> String {
    format!(
        r#"
        (function() {{
            function run(limit) {{
                var latinSource = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
                var utf16Source = "\u0100\u0100\u0100\u0100";
                var astralSource = "\u{{1F600}}\u{{1F600}}";
                var loneSource = String.fromCharCode(0xD800) + String.fromCharCode(0xD800);
                var latin = /a/g;
                var utf16 = /\u0100/g;
                var astral = /\u{{1F600}}/gu;
                var lone = /\uD800/g;
                var total = 0;
                var i = 0;
                while (i < limit) {{
                    latin.lastIndex = 0;
                    while (latin.exec(latinSource) !== null) {{
                        total = total + latin.lastIndex;
                    }}
                    utf16.lastIndex = 0;
                    while (utf16.test(utf16Source)) {{
                        total = total + utf16.lastIndex;
                    }}
                    astral.lastIndex = 0;
                    while (astral.exec(astralSource) !== null) {{
                        total = total + astral.lastIndex;
                    }}
                    lone.lastIndex = 0;
                    while (lone.test(loneSource)) {{
                        total = total + lone.lastIndex;
                    }}
                    i = i + 1;
                }}
                return total;
            }}
            return run({loop_trip_count});
        }})()
        "#
    )
}

fn array_heavy_runtime_workload(loop_trip_count: usize) -> String {
    format!(
        r"
        (function() {{
            function run(limit) {{
                var total = 0;
                var i = 0;
                while (i < limit) {{
                    var values = [i, i + 1, i + 2, i + 3];
                    values[0] = values[0] + values[3];
                    values[2] = values[2] - 1;
                    total = total + values[0] + values[1] + values[2] + values[3];
                    i = i + 1;
                }}
                return total;
            }}
            return run({loop_trip_count});
        }})()
        "
    )
}

fn array_iterator_runtime_workload(loop_trip_count: usize) -> String {
    format!(
        r"
        (function() {{
            function sum4(a, b, c, d) {{
                return a + b + c + d;
            }}

            function run(limit) {{
                var total = 0;
                var i = 0;
                while (i < limit) {{
                    var values = [i, i + 1, i + 2, i + 3];
                    var iterTotal = 0;
                    for (var value of values) {{
                        iterTotal = iterTotal + value;
                    }}
                    var copy = [0, ...values];
                    total = total + iterTotal + sum4(...values) + copy[1] + copy[4];
                    i = i + 1;
                }}
                return total;
            }}

            return run({loop_trip_count});
        }})()
        "
    )
}

fn class_heavy_runtime_workload(loop_trip_count: usize) -> String {
    format!(
        r"
        (function() {{
            class Base {{
                constructor(seed) {{
                    this.seed = seed;
                }}

                value() {{
                    return this.seed + 1;
                }}
            }}

            class Derived extends Base {{
                #offset;
                field = 2;
                static #instances = 0;
                static cache = 3;

                static {{
                    this.cache = this.cache + 4;
                }}

                constructor(seed) {{
                    super(seed);
                    this.#offset = seed + this.field;
                    Derived.#instances = Derived.#instances + 1;
                }}

                total() {{
                    return super.value() + this.#offset + this.field;
                }}

                static metrics(instance) {{
                    return this.#instances + this.cache + (#offset in instance ? 1 : 0);
                }}
            }}

            function run(limit) {{
                var total = 0;
                var i = 0;
                while (i < limit) {{
                    var instance = new Derived(i);
                    total = total + instance.total() + Derived.metrics(instance);
                    i = i + 1;
                }}
                return total;
            }}

            return run({loop_trip_count});
        }})()
        "
    )
}

fn module_heavy_compile_workload(repetitions: usize) -> String {
    let mut source = String::new();
    for index in 0..repetitions {
        let _ = writeln!(
            source,
            "import {{ value{index} as dep{index} }} from 'dep{index}.js';"
        );
        let _ = writeln!(source, "export const local{index} = dep{index};");
        let _ = writeln!(source, "export {{ local{index} as exported{index} }};");
    }
    source
}

fn async_heavy_frontend_workload(repetitions: usize) -> String {
    let mut source = String::new();
    for index in 0..repetitions {
        let _ = writeln!(
            source,
            "async function asyncValue{index}(p) {{ return await p; }}"
        );
        let _ = writeln!(
            source,
            "async function* asyncSequence{index}(p) {{ yield await p; }}"
        );
        let _ = writeln!(source, "const asyncArrow{index} = async (p) => await p;");
    }
    source
}

fn typed_array_heavy_runtime_workload(loop_trip_count: usize) -> String {
    format!(
        r"
        (function() {{
            function run(limit) {{
                var buffer = new ArrayBuffer(16);
                var bytes = new Uint8Array(buffer);
                var words = new Uint16Array(buffer, 0, 4);
                var view = new DataView(buffer);
                var total = 0;
                var i = 0;
                while (i < limit) {{
                    bytes[0] = i & 255;
                    bytes[1] = (i + 1) & 255;
                    words[1] = i + 5;
                    view.setUint16(4, words[1] + bytes[0], true);
                    total = total + bytes[0] + bytes[1] + words[1] + view.getUint16(4, true);
                    i = i + 1;
                }}
                return total;
            }}

            return run({loop_trip_count});
        }})()
        "
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_workloads_prepare_at_declared_stage() {
        let options = Options {
            report_path: DEFAULT_REPORT_PATH.to_string(),
            samples: 1,
            runs_per_sample: 1,
            warmup_runs: 1,
            loop_trip_count: 16,
            frontend_repetitions: 4,
        };
        let baseline_atom_payload = AtomTable::new().payload_bytes();

        for (index, workload) in build_workloads(16, 4).into_iter().enumerate() {
            let source_id = SourceId::new(u32::try_from(index + 1).unwrap());
            let report = measure_workload(source_id, workload.clone(), &options)
                .expect("generated workload should measure successfully");
            assert_eq!(report.workload.name, workload.name);
            assert!(report.throughput.median_total >= Duration::ZERO);
            assert!(
                report.memory.atom_payload_bytes
                    <= baseline_atom_payload + report.workload.source.len()
            );
            match report.workload.pipeline {
                WorkloadPipeline::ScriptFrontend => {
                    assert!(report.memory.atom_payload_bytes >= baseline_atom_payload);
                }
                WorkloadPipeline::ScriptRuntime | WorkloadPipeline::ModuleCompile => {}
            }
        }
    }

    #[test]
    fn class_heavy_row_is_now_executable_runtime_coverage() {
        let workload = build_workloads(16, 4)
            .into_iter()
            .find(|workload| workload.name == "class-heavy.runtime")
            .expect("class-heavy runtime row should exist");

        assert_eq!(workload.pipeline, WorkloadPipeline::ScriptRuntime);
        assert!(workload.source.contains("class Derived extends Base"));
        assert!(workload.source.contains("static #instances"));
        assert!(workload.source.contains("super(seed);"));
    }

    #[test]
    fn typed_array_heavy_row_is_now_executable_runtime_coverage() {
        let workload = build_workloads(16, 4)
            .into_iter()
            .find(|workload| workload.name == "typed-array-heavy.runtime")
            .expect("typed-array-heavy runtime row should exist");

        assert_eq!(workload.pipeline, WorkloadPipeline::ScriptRuntime);
        assert!(workload.source.contains("new ArrayBuffer"));
        assert!(workload.source.contains("new Uint8Array"));
        assert!(workload.source.contains("new DataView"));
    }

    #[test]
    fn regexp_stable_exec_row_is_focused_runtime_coverage() {
        let workload = build_workloads(16, 4)
            .into_iter()
            .find(|workload| workload.name == "regexp-stable-exec.runtime")
            .expect("stable RegExp exec runtime row should exist");

        assert_eq!(workload.pipeline, WorkloadPipeline::ScriptRuntime);
        assert!(workload.source.contains("latin.exec(latinSource)"));
        assert!(workload.source.contains("utf16.test(utf16Source)"));
        assert!(workload.source.contains(r"/\u{1F600}/gu"));
        assert!(workload.source.contains("String.fromCharCode(0xD800)"));
    }

    #[test]
    fn regexp_observability_rows_split_compile_replace_and_legacy_costs() {
        let workloads = build_workloads(16, 4);
        let compile = workloads
            .iter()
            .find(|workload| workload.name == "regexp-constructor-compile.runtime")
            .expect("RegExp constructor compile row should exist");
        let replace = workloads
            .iter()
            .find(|workload| workload.name == "regexp-named-replace.runtime")
            .expect("RegExp named replacement row should exist");
        let legacy = workloads
            .iter()
            .find(|workload| workload.name == "regexp-legacy-statics.runtime")
            .expect("RegExp legacy statics row should exist");

        assert_eq!(compile.pipeline, WorkloadPipeline::ScriptRuntime);
        assert!(compile.source.contains("new RegExp"));
        assert_eq!(replace.pipeline, WorkloadPipeline::ScriptRuntime);
        assert!(replace.source.contains("$<digits>:$<word>"));
        assert_eq!(legacy.pipeline, WorkloadPipeline::ScriptRuntime);
        assert!(legacy.source.contains("RegExp.lastMatch"));
    }

    #[test]
    fn runtime_snapshots_seed_promise_jobs_and_backing_stores() {
        let snapshots =
            capture_runtime_snapshots().expect("runtime snapshots should capture successfully");
        let seeded = snapshots
            .iter()
            .find(|snapshot| snapshot.label == "runtime.promise-and-backing-store")
            .unwrap();

        assert_eq!(seeded.accounting.promise_jobs.records, 1);
        assert_eq!(seeded.accounting.backing_stores.records, 1);
        assert_eq!(seeded.accounting.backing_stores.payload_bytes, 4096);
        assert_eq!(seeded.accounting.iterator_records.records, 0);
        assert_eq!(seeded.accounting.regexp_payloads.records, 0);
        assert_eq!(seeded.accounting.regexp_literal_cache.records, 0);
        assert_eq!(seeded.accounting.module_caches.records, 0);
    }

    #[test]
    fn runtime_snapshots_report_regexp_literal_cache_accounting() {
        let snapshots =
            capture_runtime_snapshots().expect("runtime snapshots should capture successfully");
        let seeded = snapshots
            .iter()
            .find(|snapshot| snapshot.label == "runtime.regexp-literal-cache")
            .unwrap();

        assert_eq!(seeded.accounting.regexp_literal_cache.records, 1);
        assert!(seeded.accounting.regexp_literal_cache.live_bytes > 0);
        assert!(seeded.accounting.regexp_payloads.records >= 3);
    }

    #[test]
    fn runtime_report_path_drops_phase_naming() {
        assert_eq!(DEFAULT_REPORT_PATH, "reports/js/lyng-js/bench.md");
    }

    #[test]
    fn write_report_returns_filesystem_errors() {
        let path = env::temp_dir().join(format!(
            "lyng-js-bench-runtime-report-dir-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("test report directory should be created");

        let error = write_report(
            path.to_str()
                .expect("temporary report path should be valid UTF-8"),
            "report",
        )
        .expect_err("writing a report to a directory should return an error");

        assert!(error.contains("failed to write runtime benchmark report"));
        let _ = fs::remove_dir_all(&path);
    }

    #[test]
    fn compile_script_unit_returns_parse_errors() {
        let mut atoms = AtomTable::new();
        let error = compile_script_unit(SourceId::new(1), "function", &mut atoms)
            .expect_err("invalid benchmark source should return a compile error");

        assert!(error.contains("parse errors in benchmark workload"));
    }
}
