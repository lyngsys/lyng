use std::fmt::Write as _;
use std::fs;
use std::hint::black_box;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use lyng_js_test262::{
    prepare_diagnostic_suite, Test262DiagnosticConfig, Test262DiagnosticOutcome,
    Test262DiagnosticProposalStage, Test262DiagnosticSuite,
};
use serde_json::{json, Value};

pub const DEFAULT_REPORT_PATH: &str = "reports/js/lyng-js/test262-perf.md";
pub const DEFAULT_JSON_PATH: &str = "reports/js/lyng-js/test262-perf.json";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Test262Options {
    pub report_path: String,
    pub json_path: String,
    pub filter: Option<String>,
    pub manifest_path: String,
    pub no_skip: bool,
    pub proposal_stage: Test262DiagnosticProposalStage,
    pub mode: Test262Mode,
    pub samples: usize,
    pub warmup_samples: usize,
    pub sample_files: usize,
    pub jobs: usize,
    pub timeout_ms: u64,
    pub profile_loop_ms: Option<u64>,
    pub print_counters: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Test262Mode {
    Hybrid,
    Scan,
    Sample,
}

impl Test262Mode {
    const fn label(self) -> &'static str {
        match self {
            Self::Hybrid => "hybrid",
            Self::Scan => "scan",
            Self::Sample => "sample",
        }
    }
}

impl Test262Options {
    #[must_use]
    pub fn default_for_test() -> Self {
        Self {
            report_path: DEFAULT_REPORT_PATH.to_string(),
            json_path: DEFAULT_JSON_PATH.to_string(),
            filter: None,
            manifest_path: "reports/js/lyng-js/test262-exclusions.txt".to_string(),
            no_skip: false,
            proposal_stage: Test262DiagnosticProposalStage::Stage3,
            mode: Test262Mode::Hybrid,
            samples: 5,
            warmup_samples: 1,
            sample_files: 25,
            jobs: 1,
            timeout_ms: 1_000,
            profile_loop_ms: None,
            print_counters: false,
        }
    }
}

impl Default for Test262Options {
    fn default() -> Self {
        Self::default_for_test()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Test262PhaseTimings {
    pub read_source: Duration,
    pub runtime_assembly: Duration,
    pub frontend_check: Duration,
    pub parse: Duration,
    pub sema: Duration,
    pub lowering: Duration,
    pub script_install: Duration,
    pub realm_bootstrap: Duration,
    pub extension_install: Duration,
    pub global_instantiation: Duration,
    pub bytecode_execution: Duration,
    pub job_checkpoint: Duration,
    pub install_or_load: Duration,
    pub evaluation: Duration,
    pub total: Duration,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Test262VariantDiagnostics {
    pub function_count: usize,
    pub instruction_words: usize,
    pub wide_prefixes: usize,
    pub metadata_records: usize,
    pub constants: usize,
    pub source_map_entries: usize,
    pub safepoints: usize,
    pub deopt_snapshots: usize,
    pub feedback_slots: usize,
    pub live_feedback_sites: usize,
    pub megamorphic_sites: usize,
    pub tier_hotness: u32,
    pub tier_feedback_events: u32,
    pub tier_backedge_events: u32,
    pub runtime_live_bytes_before: usize,
    pub runtime_live_bytes_after: usize,
    pub runtime_live_bytes_delta: isize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Test262Sample {
    pub outcome: String,
    pub timings: Test262PhaseTimings,
    pub diagnostics: Option<Test262VariantDiagnostics>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Test262VariantIdentity {
    pub file: String,
    pub variant: Option<String>,
    pub category: String,
    pub flags: Vec<String>,
    pub features: Vec<String>,
    pub includes: Vec<String>,
    pub negative_phase: Option<String>,
    pub async_test: bool,
    pub module_goal: bool,
    pub timeout_ms: u64,
}

impl Test262VariantIdentity {
    fn display_name(&self) -> String {
        self.variant.as_ref().map_or_else(
            || self.file.clone(),
            |variant| format!("{} [{variant}]", self.file),
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Test262Aggregate {
    pub identity: Test262VariantIdentity,
    pub samples: Vec<Test262Sample>,
    pub median_total: Duration,
    pub min_total: Duration,
    pub max_total: Duration,
    pub median_evaluation: Duration,
    pub dominant_phase: String,
    pub cause_hints: Vec<String>,
}

/// Run Test262 performance diagnostics and write Markdown plus JSON reports.
///
/// # Errors
///
/// Returns an error when CLI arguments are invalid, Test262 preparation fails,
/// diagnostic execution fails, or reports cannot be written.
pub fn run(args: &[String]) -> Result<(), String> {
    let options = parse_options(args)?;
    if cfg!(debug_assertions) {
        eprintln!("warning: build with --release for meaningful measurements");
    }

    let suite = prepare_diagnostic_suite(&Test262DiagnosticConfig {
        filter: options.filter.clone(),
        manifest_path: options.manifest_path.clone(),
        no_skip: options.no_skip,
        timeout_ms: options.timeout_ms,
        proposal_stage: options.proposal_stage,
    })?;
    eprintln!(
        "Prepared {} runnable Test262 variants from {} candidate files",
        suite.tests().len(),
        suite.candidate_total()
    );

    let scan = run_scan(&suite, options.jobs)?;
    let sampled = match options.mode {
        Test262Mode::Scan => scan
            .iter()
            .map(|outcome| {
                (
                    identity_from_outcome(outcome),
                    vec![sample_from_outcome(outcome)],
                )
            })
            .collect::<Vec<_>>(),
        Test262Mode::Hybrid | Test262Mode::Sample => run_sampled_variants(&suite, &scan, &options)?,
    };
    let aggregates = aggregate_sampled_variants(sampled);
    let previous = read_previous_json(&options.json_path);
    let markdown = render_markdown_report(&options, &aggregates, previous.as_ref());
    let json = render_json_report(&options, &aggregates, previous.as_ref());
    write_text_report(&options.report_path, &markdown)?;
    write_text_report(
        &options.json_path,
        &serde_json::to_string_pretty(&json)
            .map_err(|error| format!("failed to render JSON report: {error}"))?,
    )?;
    print_summary(&options, &aggregates);
    if options.print_counters {
        if let Some(aggregate) = aggregates.first() {
            eprintln!("{}", render_profile_counter_summary(aggregate));
        } else {
            eprintln!("Profile counters: no sampled Test262 variant available");
        }
    }
    if let Some(profile_loop_ms) = options.profile_loop_ms {
        run_profile_loop(
            &suite,
            &scan,
            &aggregates,
            Duration::from_millis(profile_loop_ms),
        )?;
    }
    Ok(())
}

fn parse_options(args: &[String]) -> Result<Test262Options, String> {
    let mut options = Test262Options::default();
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--preset" => {
                apply_preset(&mut options, required_value("--preset", args.next())?)?;
            }
            "--filter" => {
                options.filter = Some(required_value("--filter", args.next())?.to_string());
            }
            "--report" => {
                options.report_path = required_value("--report", args.next())?.to_string();
            }
            "--json" => {
                options.json_path = required_value("--json", args.next())?.to_string();
            }
            "--manifest" => {
                options.manifest_path = required_value("--manifest", args.next())?.to_string();
            }
            "--no-skip" => {
                options.no_skip = true;
            }
            "--proposal-stage" => {
                options.proposal_stage =
                    parse_proposal_stage(required_value("--proposal-stage", args.next())?)?;
            }
            "--mode" => {
                options.mode = parse_mode(required_value("--mode", args.next())?)?;
            }
            "--samples" => {
                options.samples = parse_usize("--samples", args.next())?;
            }
            "--warmup-samples" => {
                options.warmup_samples = parse_usize("--warmup-samples", args.next())?;
            }
            "--sample-files" => {
                options.sample_files = parse_usize("--sample-files", args.next())?;
            }
            "--timeout-ms" => {
                options.timeout_ms = parse_u64("--timeout-ms", args.next())?.max(1);
            }
            "--profile-loop-ms" => {
                options.profile_loop_ms = Some(parse_u64("--profile-loop-ms", args.next())?);
            }
            "--print-counters" => {
                options.print_counters = true;
            }
            "--jobs" | "-j" => {
                options.jobs = parse_usize(arg, args.next())?.max(1);
            }
            "--help" | "-h" => return Err(usage()),
            unknown if unknown.starts_with("-j") && unknown.len() > 2 => {
                options.jobs = unknown[2..]
                    .parse::<usize>()
                    .map_err(|_| "-j expects a positive integer".to_string())?
                    .max(1);
            }
            unknown => return Err(format!("Unknown argument: {unknown}\n\n{}", usage())),
        }
    }
    if options.samples == 0 {
        return Err("--samples must be greater than zero".to_string());
    }
    if options.sample_files == 0 {
        return Err("--sample-files must be greater than zero".to_string());
    }
    if matches!(options.profile_loop_ms, Some(0)) {
        return Err("--profile-loop-ms must be greater than zero".to_string());
    }
    Ok(options)
}

/// Parse Test262 benchmark options for integration tests without running the suite.
///
/// # Errors
///
/// Returns an error if the supplied arguments are invalid.
pub fn parse_options_for_test(args: &[String]) -> Result<Test262Options, String> {
    parse_options(args)
}

fn usage() -> String {
    [
        "Usage: lyng-js-bench test262 [--preset <smoke|inner-loop|baseline|ci-regression|profile-target>] [--filter <path-or-fragment>] [--report <path>] [--json <path>] [--mode hybrid|scan|sample] [--samples <n>] [--warmup-samples <n>] [--sample-files <n>] [--manifest <path>] [--proposal-stage <4|3|2.7>] [--no-skip] [--timeout-ms <N>] [--profile-loop-ms <N>] [--print-counters] [-j <N>]",
        "",
        "Runs Test262 performance diagnostics for agent triage.",
    ]
    .join("\n")
}

fn apply_preset(options: &mut Test262Options, preset: &str) -> Result<(), String> {
    match preset {
        "smoke" => {
            options.mode = Test262Mode::Hybrid;
            options.samples = 1;
            options.warmup_samples = 0;
            options.sample_files = 2;
            options.timeout_ms = 3_000;
        }
        "inner-loop" => {
            options.mode = Test262Mode::Hybrid;
            options.samples = 3;
            options.warmup_samples = 1;
            options.sample_files = 5;
            options.timeout_ms = 3_000;
        }
        "baseline" => {
            let defaults = Test262Options::default();
            options.mode = defaults.mode;
            options.samples = defaults.samples;
            options.warmup_samples = defaults.warmup_samples;
            options.sample_files = defaults.sample_files;
            options.timeout_ms = defaults.timeout_ms;
        }
        "ci-regression" => {
            options.mode = Test262Mode::Hybrid;
            options.samples = 3;
            options.warmup_samples = 1;
            options.sample_files = 25;
            options.timeout_ms = 1_000;
        }
        "profile-target" => {
            options.mode = Test262Mode::Sample;
            options.samples = 1;
            options.warmup_samples = 0;
            options.sample_files = 1;
            options.jobs = 1;
            options.timeout_ms = 10_000;
            options.profile_loop_ms = Some(30_000);
        }
        _ => {
            return Err(format!(
                "invalid --preset value `{preset}`; expected smoke, inner-loop, baseline, ci-regression, or profile-target"
            ));
        }
    }
    Ok(())
}

fn required_value<'a>(flag: &str, value: Option<&'a String>) -> Result<&'a str, String> {
    value
        .map(String::as_str)
        .ok_or_else(|| format!("{flag} requires a value"))
}

fn parse_usize(flag: &str, value: Option<&String>) -> Result<usize, String> {
    required_value(flag, value)?
        .parse()
        .map_err(|_| format!("{flag} expects a positive integer"))
}

fn parse_u64(flag: &str, value: Option<&String>) -> Result<u64, String> {
    required_value(flag, value)?
        .parse()
        .map_err(|_| format!("{flag} expects a positive integer"))
}

fn parse_mode(value: &str) -> Result<Test262Mode, String> {
    match value {
        "hybrid" => Ok(Test262Mode::Hybrid),
        "scan" => Ok(Test262Mode::Scan),
        "sample" => Ok(Test262Mode::Sample),
        _ => Err(format!("invalid --mode value `{value}`")),
    }
}

fn parse_proposal_stage(value: &str) -> Result<Test262DiagnosticProposalStage, String> {
    match value {
        "4" => Ok(Test262DiagnosticProposalStage::Stage4),
        "3" => Ok(Test262DiagnosticProposalStage::Stage3),
        "2.7" => Ok(Test262DiagnosticProposalStage::Stage2_7),
        _ => Err(format!("invalid --proposal-stage value `{value}`")),
    }
}

fn run_scan(
    suite: &Test262DiagnosticSuite,
    jobs: usize,
) -> Result<Vec<Test262DiagnosticOutcome>, String> {
    if suite.tests().is_empty() {
        return Ok(Vec::new());
    }
    let next_index = AtomicUsize::new(0);
    let outcomes = Mutex::new(Vec::with_capacity(suite.tests().len()));
    let errors = Mutex::new(Vec::new());
    let job_count = jobs.min(suite.tests().len()).max(1);

    std::thread::scope(|scope| {
        for _ in 0..job_count {
            scope.spawn(|| loop {
                let index = next_index.fetch_add(1, Ordering::Relaxed);
                if index >= suite.tests().len() {
                    break;
                }
                match suite.run_diagnostic(index) {
                    Ok(outcome) => match outcomes.lock() {
                        Ok(mut outcomes) => outcomes.push(outcome),
                        Err(poisoned) => poisoned.into_inner().push(outcome),
                    },
                    Err(error) => match errors.lock() {
                        Ok(mut errors) => errors.push(error),
                        Err(poisoned) => poisoned.into_inner().push(error),
                    },
                }
            });
        }
    });

    let errors = errors
        .into_inner()
        .map_err(|_| "diagnostic error lock poisoned".to_string())?;
    if let Some(error) = errors.into_iter().next() {
        return Err(error);
    }
    let mut outcomes = outcomes
        .into_inner()
        .map_err(|_| "diagnostic outcome lock poisoned".to_string())?;
    outcomes.sort_by(|left, right| {
        right
            .timings
            .total
            .cmp(&left.timings.total)
            .then_with(|| left.identity.file.cmp(&right.identity.file))
            .then_with(|| left.identity.variant.cmp(&right.identity.variant))
    });
    Ok(outcomes)
}

fn run_sampled_variants(
    suite: &Test262DiagnosticSuite,
    scan: &[Test262DiagnosticOutcome],
    options: &Test262Options,
) -> Result<Vec<(Test262VariantIdentity, Vec<Test262Sample>)>, String> {
    let mut selected = Vec::new();
    for candidate in scan.iter().take(options.sample_files) {
        for _ in 0..options.warmup_samples {
            let _ = suite.run_diagnostic(candidate.identity.index)?;
        }
        let mut samples = Vec::with_capacity(options.samples);
        for _ in 0..options.samples {
            samples.push(sample_from_outcome(
                &suite.run_diagnostic(candidate.identity.index)?,
            ));
        }
        selected.push((identity_from_outcome(candidate), samples));
    }
    Ok(selected)
}

fn run_profile_loop(
    suite: &Test262DiagnosticSuite,
    scan: &[Test262DiagnosticOutcome],
    aggregates: &[Test262Aggregate],
    minimum_duration: Duration,
) -> Result<(), String> {
    let Some((target_index, target_name)) = profile_target(scan, aggregates) else {
        eprintln!("Profile loop skipped: no sampled Test262 variant available");
        return Ok(());
    };

    eprintln!(
        "Profile loop target: `{target_name}` for at least {}",
        format_duration(minimum_duration)
    );
    let started = Instant::now();
    let mut iterations = 0usize;
    while iterations == 0 || started.elapsed() < minimum_duration {
        let outcome = suite.run_diagnostic(target_index)?;
        black_box(outcome);
        iterations += 1;
    }
    eprintln!(
        "Profile loop completed: target=`{target_name}` iterations={iterations} elapsed={}",
        format_duration(started.elapsed())
    );
    Ok(())
}

fn profile_target(
    scan: &[Test262DiagnosticOutcome],
    aggregates: &[Test262Aggregate],
) -> Option<(usize, String)> {
    let aggregate_identity = &aggregates.first()?.identity;
    let outcome = scan.iter().find(|outcome| {
        outcome.identity.file == aggregate_identity.file
            && outcome.identity.variant == aggregate_identity.variant
    })?;
    Some((outcome.identity.index, aggregate_identity.display_name()))
}

fn identity_from_outcome(outcome: &Test262DiagnosticOutcome) -> Test262VariantIdentity {
    Test262VariantIdentity {
        file: outcome.identity.file.clone(),
        variant: outcome.identity.variant.clone(),
        category: outcome.identity.category.clone(),
        flags: outcome.identity.flags.clone(),
        features: outcome.identity.features.clone(),
        includes: outcome.identity.includes.clone(),
        negative_phase: outcome.identity.negative_phase.clone(),
        async_test: outcome.identity.async_test,
        module_goal: outcome.identity.module_goal,
        timeout_ms: outcome.identity.timeout_ms,
    }
}

fn sample_from_outcome(outcome: &Test262DiagnosticOutcome) -> Test262Sample {
    Test262Sample {
        outcome: outcome.outcome.clone(),
        timings: Test262PhaseTimings {
            read_source: outcome.timings.read_source,
            runtime_assembly: outcome.timings.runtime_assembly,
            frontend_check: outcome.timings.frontend_check,
            parse: outcome.timings.parse,
            sema: outcome.timings.sema,
            lowering: outcome.timings.lowering,
            script_install: outcome.timings.script_install,
            realm_bootstrap: outcome.timings.realm_bootstrap,
            extension_install: outcome.timings.extension_install,
            global_instantiation: outcome.timings.global_instantiation,
            bytecode_execution: outcome.timings.bytecode_execution,
            job_checkpoint: outcome.timings.job_checkpoint,
            install_or_load: outcome.timings.install_or_load,
            evaluation: outcome.timings.evaluation,
            total: outcome.timings.total,
        },
        diagnostics: outcome
            .diagnostics
            .as_ref()
            .map(|diagnostics| Test262VariantDiagnostics {
                function_count: diagnostics.function_count,
                instruction_words: diagnostics.instruction_words,
                wide_prefixes: diagnostics.wide_prefixes,
                metadata_records: diagnostics.metadata_records,
                constants: diagnostics.constants,
                source_map_entries: diagnostics.source_map_entries,
                safepoints: diagnostics.safepoints,
                deopt_snapshots: diagnostics.deopt_snapshots,
                feedback_slots: diagnostics.feedback_slots,
                live_feedback_sites: diagnostics.live_feedback_sites,
                megamorphic_sites: diagnostics.megamorphic_sites,
                tier_hotness: diagnostics.tier_hotness,
                tier_feedback_events: diagnostics.tier_feedback_events,
                tier_backedge_events: diagnostics.tier_backedge_events,
                runtime_live_bytes_before: diagnostics.runtime_live_bytes_before,
                runtime_live_bytes_after: diagnostics.runtime_live_bytes_after,
                runtime_live_bytes_delta: diagnostics.runtime_live_bytes_delta,
            }),
    }
}

fn read_previous_json(path: &str) -> Option<Value> {
    fs::read_to_string(path)
        .ok()
        .and_then(|source| serde_json::from_str(&source).ok())
}

fn write_text_report(path: &str, text: &str) -> Result<(), String> {
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create report directory: {error}"))?;
    }
    fs::write(path, text).map_err(|error| format!("failed to write {path}: {error}"))
}

fn print_summary(options: &Test262Options, aggregates: &[Test262Aggregate]) {
    println!("\n========== Lyng JS Test262 Performance ==========");
    println!("Mode:                 {}", options.mode.label());
    println!(
        "Filter:               {}",
        options.filter.as_deref().unwrap_or("whole corpus")
    );
    println!("Sampled variants:     {}", aggregates.len());
    if let Some(slowest) = aggregates.first() {
        println!(
            "Slowest median:       {} ({})",
            slowest.identity.display_name(),
            format_duration(slowest.median_total)
        );
        println!("Dominant phase:       {}", slowest.dominant_phase);
    }
    println!("Markdown report:      {}", options.report_path);
    println!("JSON report:          {}", options.json_path);
    println!(
        "Profile loop:         {}",
        options.profile_loop_ms.map_or_else(
            || "disabled".to_string(),
            |ms| format_duration(Duration::from_millis(ms)),
        )
    );
}

#[must_use]
pub fn aggregate_sampled_variants(
    variants: Vec<(Test262VariantIdentity, Vec<Test262Sample>)>,
) -> Vec<Test262Aggregate> {
    let mut aggregates = variants
        .into_iter()
        .filter_map(|(identity, samples)| {
            if samples.is_empty() {
                return None;
            }
            let totals = samples
                .iter()
                .map(|sample| sample.timings.total)
                .collect::<Vec<_>>();
            let evaluations = samples
                .iter()
                .map(|sample| sample.timings.evaluation)
                .collect::<Vec<_>>();
            let median_total = median_duration(totals.clone());
            let min_total = totals.iter().copied().min().unwrap_or(Duration::ZERO);
            let max_total = totals.iter().copied().max().unwrap_or(Duration::ZERO);
            let median_evaluation = median_duration(evaluations);
            let dominant_phase = dominant_phase(&samples);
            let diagnostics = samples
                .iter()
                .rev()
                .find_map(|sample| sample.diagnostics.as_ref());
            let cause_hints = cause_hints_for_aggregate(
                &identity.file,
                &dominant_phase,
                diagnostics,
                median_total,
                min_total,
                max_total,
                identity.timeout_ms,
            );
            Some(Test262Aggregate {
                identity,
                samples,
                median_total,
                min_total,
                max_total,
                median_evaluation,
                dominant_phase,
                cause_hints,
            })
        })
        .collect::<Vec<_>>();
    aggregates.sort_by(|left, right| {
        right
            .median_total
            .cmp(&left.median_total)
            .then_with(|| left.identity.file.cmp(&right.identity.file))
            .then_with(|| left.identity.variant.cmp(&right.identity.variant))
    });
    aggregates
}

#[must_use]
pub fn cause_hints_for_aggregate(
    file: &str,
    dominant_phase: &str,
    diagnostics: Option<&Test262VariantDiagnostics>,
    median_total: Duration,
    min_total: Duration,
    max_total: Duration,
    timeout_ms: u64,
) -> Vec<String> {
    let mut hints = Vec::new();
    if dominant_phase == "evaluation" {
        hints.push("evaluation dominated".to_string());
    }
    if dominant_phase == "bytecode_execution" {
        hints.push("bytecode execution dominated".to_string());
    }
    if dominant_phase == "job_checkpoint" {
        hints.push("job checkpoint dominated".to_string());
    }
    if matches!(
        dominant_phase,
        "script_install" | "realm_bootstrap" | "extension_install" | "global_instantiation"
    ) {
        hints.push("runtime setup dominated".to_string());
    }
    if matches!(
        dominant_phase,
        "parse" | "sema" | "lowering" | "frontend_check"
    ) {
        hints.push("frontend dominated".to_string());
    }
    if file.contains("/Date/") || file.contains("Date/") || file.contains("dst-offset") {
        hints.push("Date/timezone candidate".to_string());
    }
    if file.contains("/RegExp/") {
        hints.push("RegExp candidate".to_string());
    }
    if file.contains("TypedArray") || file.contains("ArrayBuffer") || file.contains("DataView") {
        hints.push("typed-array/binary-data candidate".to_string());
    }
    if file.contains("/module") || file.contains("import") || file.contains("export") {
        hints.push("module-load candidate".to_string());
    }
    if let Some(diagnostics) = diagnostics {
        if diagnostics.megamorphic_sites > 0 {
            hints.push("megamorphic inline-cache activity".to_string());
        }
        if diagnostics.feedback_slots >= 16 || diagnostics.live_feedback_sites >= 16 {
            hints.push("large feedback-vector surface".to_string());
        }
        if diagnostics.tier_hotness > 0 || diagnostics.tier_backedge_events > 0 {
            hints.push("tiering hotness signal".to_string());
        }
        if diagnostics.runtime_live_bytes_delta > 1_000_000 {
            hints.push("large runtime memory growth".to_string());
        }
    }
    if max_total > min_total.saturating_mul(2) {
        hints.push("high timing variance".to_string());
    }
    let timeout = Duration::from_millis(timeout_ms);
    if timeout > Duration::ZERO && median_total >= timeout.mul_f64(0.80) {
        hints.push("near timeout".to_string());
    }
    hints.sort();
    hints.dedup();
    hints
}

pub fn render_markdown_report(
    options: &Test262Options,
    aggregates: &[Test262Aggregate],
    previous: Option<&Value>,
) -> String {
    let mut out = String::new();
    let filter = options
        .filter
        .as_deref()
        .map_or("whole corpus", |filter| filter);
    let _ = writeln!(out, "# Lyng JS Test262 Performance Diagnostics");
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "This report is generated by `cargo run --release -p lyng-js-bench -- test262 --report {}`.",
        options.report_path
    );
    let _ = writeln!(
        out,
        "It is optimized for agent triage: slow variants, dominant phases, diagnostics, and cause hints."
    );
    let _ = writeln!(out);
    let _ = writeln!(out, "## Settings");
    write_markdown_settings(&mut out, options, filter);
    let _ = writeln!(out);
    let _ = writeln!(out, "## Sampled Bottlenecks");
    let _ = writeln!(out);
    if previous.is_some() {
        let _ = writeln!(
            out,
            "| Test variant | Median total | Median total delta | Min | Max | Median eval | Dominant phase | Cause hints |"
        );
        let _ = writeln!(
            out,
            "| --- | ---: | ---: | ---: | ---: | ---: | --- | --- |"
        );
    } else {
        let _ = writeln!(
            out,
            "| Test variant | Median total | Min | Max | Median eval | Dominant phase | Cause hints |"
        );
        let _ = writeln!(out, "| --- | ---: | ---: | ---: | ---: | --- | --- |");
    }
    for aggregate in aggregates {
        let hints = if aggregate.cause_hints.is_empty() {
            "n/a".to_string()
        } else {
            aggregate.cause_hints.join(", ")
        };
        if let Some(previous) = previous {
            let delta = previous_median_total_ms(previous, &aggregate.identity)
                .map(|previous| duration_ms(aggregate.median_total) - previous)
                .map_or_else(|| "n/a".to_string(), format_delta_ms);
            let _ = writeln!(
                out,
                "| `{}` | `{}` | `{}` | `{}` | `{}` | `{}` | `{}` | {} |",
                aggregate.identity.display_name(),
                format_duration(aggregate.median_total),
                delta,
                format_duration(aggregate.min_total),
                format_duration(aggregate.max_total),
                format_duration(aggregate.median_evaluation),
                aggregate.dominant_phase,
                hints,
            );
        } else {
            let _ = writeln!(
                out,
                "| `{}` | `{}` | `{}` | `{}` | `{}` | `{}` | {} |",
                aggregate.identity.display_name(),
                format_duration(aggregate.median_total),
                format_duration(aggregate.min_total),
                format_duration(aggregate.max_total),
                format_duration(aggregate.median_evaluation),
                aggregate.dominant_phase,
                hints,
            );
        }
    }
    let _ = writeln!(out);
    let _ = writeln!(out, "## Agent Notes");
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "- Dominant phase points at the first subsystem to inspect; use JSON samples for raw timings and diagnostics."
    );
    if previous.is_some() {
        let _ = writeln!(
            out,
            "- Previous JSON was available; deltas are report-only in v1."
        );
    } else {
        let _ = writeln!(
            out,
            "- No previous JSON baseline was available at this report path."
        );
    }
    out
}

fn write_markdown_settings(out: &mut String, options: &Test262Options, filter: &str) {
    let _ = writeln!(out);
    let _ = writeln!(out, "- Filter: `{filter}`");
    let _ = writeln!(out, "- Samples: `{}`", options.samples);
    let _ = writeln!(out, "- Warmup samples: `{}`", options.warmup_samples);
    let _ = writeln!(out, "- Sample files: `{}`", options.sample_files);
    let _ = writeln!(out, "- Jobs: `{}`", options.jobs);
    let profile_loop = options
        .profile_loop_ms
        .map_or_else(|| "disabled".to_string(), |ms| format!("{ms}ms"));
    let _ = writeln!(out, "- Profile loop: `{profile_loop}`");
    let _ = writeln!(out, "- Print counters: `{}`", options.print_counters);
    let _ = writeln!(out, "- JSON: `{}`", options.json_path);
}

#[must_use]
pub fn render_json_report(
    options: &Test262Options,
    aggregates: &[Test262Aggregate],
    previous: Option<&Value>,
) -> Value {
    json!({
        "schema_version": 1,
        "tool": "lyng-js-bench test262",
        "settings": {
            "filter": options.filter,
            "samples": options.samples,
            "warmup_samples": options.warmup_samples,
            "sample_files": options.sample_files,
            "jobs": options.jobs,
            "timeout_ms": options.timeout_ms,
            "profile_loop_ms": options.profile_loop_ms,
            "print_counters": options.print_counters,
            "report_path": options.report_path,
            "json_path": options.json_path,
        },
        "has_previous": previous.is_some(),
        "aggregates": aggregates
            .iter()
            .map(|aggregate| aggregate_json(aggregate, previous))
            .collect::<Vec<_>>(),
    })
}

fn aggregate_json(aggregate: &Test262Aggregate, previous: Option<&Value>) -> Value {
    let delta = previous
        .and_then(|previous| previous_median_total_ms(previous, &aggregate.identity))
        .map(|previous| {
            json!({
                "median_total_ms": duration_ms(aggregate.median_total) - previous,
            })
        });
    json!({
        "identity": {
            "file": aggregate.identity.file,
            "variant": aggregate.identity.variant,
            "category": aggregate.identity.category,
            "flags": aggregate.identity.flags,
            "features": aggregate.identity.features,
            "includes": aggregate.identity.includes,
            "negative_phase": aggregate.identity.negative_phase,
            "async_test": aggregate.identity.async_test,
            "module_goal": aggregate.identity.module_goal,
            "timeout_ms": aggregate.identity.timeout_ms,
        },
        "median_total_ms": duration_ms(aggregate.median_total),
        "min_total_ms": duration_ms(aggregate.min_total),
        "max_total_ms": duration_ms(aggregate.max_total),
        "median_evaluation_ms": duration_ms(aggregate.median_evaluation),
        "dominant_phase": aggregate.dominant_phase,
        "delta": delta,
        "cause_hints": aggregate.cause_hints,
        "samples": aggregate.samples.iter().map(sample_json).collect::<Vec<_>>(),
    })
}

fn sample_json(sample: &Test262Sample) -> Value {
    json!({
        "outcome": sample.outcome,
        "timings": phase_json(&sample.timings),
        "diagnostics": sample.diagnostics.as_ref().map(diagnostics_json),
    })
}

fn phase_json(timings: &Test262PhaseTimings) -> Value {
    json!({
        "read_source_ms": duration_ms(timings.read_source),
        "runtime_assembly_ms": duration_ms(timings.runtime_assembly),
        "frontend_check_ms": duration_ms(timings.frontend_check),
        "parse_ms": duration_ms(timings.parse),
        "sema_ms": duration_ms(timings.sema),
        "lowering_ms": duration_ms(timings.lowering),
        "script_install_ms": duration_ms(timings.script_install),
        "realm_bootstrap_ms": duration_ms(timings.realm_bootstrap),
        "extension_install_ms": duration_ms(timings.extension_install),
        "global_instantiation_ms": duration_ms(timings.global_instantiation),
        "bytecode_execution_ms": duration_ms(timings.bytecode_execution),
        "job_checkpoint_ms": duration_ms(timings.job_checkpoint),
        "install_or_load_ms": duration_ms(timings.install_or_load),
        "evaluation_ms": duration_ms(timings.evaluation),
        "total_ms": duration_ms(timings.total),
    })
}

fn diagnostics_json(diagnostics: &Test262VariantDiagnostics) -> Value {
    json!({
        "function_count": diagnostics.function_count,
        "instruction_words": diagnostics.instruction_words,
        "wide_prefixes": diagnostics.wide_prefixes,
        "metadata_records": diagnostics.metadata_records,
        "constants": diagnostics.constants,
        "source_map_entries": diagnostics.source_map_entries,
        "safepoints": diagnostics.safepoints,
        "deopt_snapshots": diagnostics.deopt_snapshots,
        "feedback_slots": diagnostics.feedback_slots,
        "live_feedback_sites": diagnostics.live_feedback_sites,
        "megamorphic_sites": diagnostics.megamorphic_sites,
        "tier_hotness": diagnostics.tier_hotness,
        "tier_feedback_events": diagnostics.tier_feedback_events,
        "tier_backedge_events": diagnostics.tier_backedge_events,
        "runtime_live_bytes_before": diagnostics.runtime_live_bytes_before,
        "runtime_live_bytes_after": diagnostics.runtime_live_bytes_after,
        "runtime_live_bytes_delta": diagnostics.runtime_live_bytes_delta,
    })
}

#[must_use]
pub fn render_profile_counter_summary(aggregate: &Test262Aggregate) -> String {
    let mut out = String::new();
    let _ = write!(
        out,
        "Profile counters: target=\"{}\" dominant_phase={} median_total_ms={:.3} median_evaluation_ms={:.3}",
        aggregate.identity.display_name(),
        aggregate.dominant_phase,
        duration_ms(aggregate.median_total),
        duration_ms(aggregate.median_evaluation),
    );
    if let Some(sample) = aggregate
        .samples
        .iter()
        .rev()
        .find(|sample| sample.diagnostics.is_some())
        .or_else(|| aggregate.samples.last())
    {
        let timings = &sample.timings;
        let _ = write!(
            out,
            " script_install_ms={:.3} bytecode_execution_ms={:.3} job_checkpoint_ms={:.3} evaluation_ms={:.3} total_ms={:.3}",
            duration_ms(timings.script_install),
            duration_ms(timings.bytecode_execution),
            duration_ms(timings.job_checkpoint),
            duration_ms(timings.evaluation),
            duration_ms(timings.total),
        );
        if let Some(diagnostics) = sample.diagnostics.as_ref() {
            let _ = write!(
                out,
                " function_count={} instruction_words={} feedback_slots={} live_feedback_sites={} megamorphic_sites={} tier_hotness={} runtime_live_bytes_delta={}",
                diagnostics.function_count,
                diagnostics.instruction_words,
                diagnostics.feedback_slots,
                diagnostics.live_feedback_sites,
                diagnostics.megamorphic_sites,
                diagnostics.tier_hotness,
                diagnostics.runtime_live_bytes_delta,
            );
        } else {
            out.push_str(" counters=unavailable");
        }
    } else {
        out.push_str(" samples=none counters=unavailable");
    }
    out
}

fn dominant_phase(samples: &[Test262Sample]) -> String {
    let refined_runtime_total: Duration = samples
        .iter()
        .map(|sample| {
            sample.timings.script_install
                + sample.timings.realm_bootstrap
                + sample.timings.extension_install
                + sample.timings.global_instantiation
                + sample.timings.bytecode_execution
                + sample.timings.job_checkpoint
        })
        .sum();
    if refined_runtime_total > Duration::ZERO {
        let mut totals = [
            ("read_source", Duration::ZERO),
            ("runtime_assembly", Duration::ZERO),
            ("frontend_check", Duration::ZERO),
            ("parse", Duration::ZERO),
            ("sema", Duration::ZERO),
            ("lowering", Duration::ZERO),
            ("script_install", Duration::ZERO),
            ("realm_bootstrap", Duration::ZERO),
            ("extension_install", Duration::ZERO),
            ("global_instantiation", Duration::ZERO),
            ("bytecode_execution", Duration::ZERO),
            ("job_checkpoint", Duration::ZERO),
        ];
        for sample in samples {
            totals[0].1 += sample.timings.read_source;
            totals[1].1 += sample.timings.runtime_assembly;
            totals[2].1 += sample.timings.frontend_check;
            totals[3].1 += sample.timings.parse;
            totals[4].1 += sample.timings.sema;
            totals[5].1 += sample.timings.lowering;
            totals[6].1 += sample.timings.script_install;
            totals[7].1 += sample.timings.realm_bootstrap;
            totals[8].1 += sample.timings.extension_install;
            totals[9].1 += sample.timings.global_instantiation;
            totals[10].1 += sample.timings.bytecode_execution;
            totals[11].1 += sample.timings.job_checkpoint;
        }
        return totals
            .into_iter()
            .max_by_key(|(_, duration)| *duration)
            .map_or_else(|| "unknown".to_string(), |(name, _)| name.to_string());
    }

    let mut totals = [
        ("read_source", Duration::ZERO),
        ("runtime_assembly", Duration::ZERO),
        ("frontend_check", Duration::ZERO),
        ("parse", Duration::ZERO),
        ("sema", Duration::ZERO),
        ("lowering", Duration::ZERO),
        ("install_or_load", Duration::ZERO),
        ("evaluation", Duration::ZERO),
    ];
    for sample in samples {
        totals[0].1 += sample.timings.read_source;
        totals[1].1 += sample.timings.runtime_assembly;
        totals[2].1 += sample.timings.frontend_check;
        totals[3].1 += sample.timings.parse;
        totals[4].1 += sample.timings.sema;
        totals[5].1 += sample.timings.lowering;
        totals[6].1 += sample.timings.install_or_load;
        totals[7].1 += sample.timings.evaluation;
    }
    totals
        .into_iter()
        .max_by_key(|(_, duration)| *duration)
        .map_or_else(|| "unknown".to_string(), |(name, _)| name.to_string())
}

fn median_duration(mut durations: Vec<Duration>) -> Duration {
    durations.sort();
    durations[durations.len() / 2]
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

fn duration_ms(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1_000.0
}

fn previous_median_total_ms(previous: &Value, identity: &Test262VariantIdentity) -> Option<f64> {
    previous
        .get("aggregates")?
        .as_array()?
        .iter()
        .find(|aggregate| {
            aggregate.pointer("/identity/file").and_then(Value::as_str)
                == Some(identity.file.as_str())
                && json_variant_matches(
                    aggregate.pointer("/identity/variant"),
                    identity.variant.as_ref(),
                )
        })
        .and_then(|aggregate| aggregate.get("median_total_ms"))
        .and_then(Value::as_f64)
}

fn json_variant_matches(value: Option<&Value>, expected: Option<&String>) -> bool {
    match (value, expected) {
        (Some(Value::String(actual)), Some(expected)) => actual == expected,
        (Some(Value::Null) | None, None) => true,
        _ => false,
    }
}

fn format_delta_ms(delta: f64) -> String {
    format!("{delta:+.3}ms")
}
