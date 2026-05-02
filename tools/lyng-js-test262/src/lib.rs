mod cli;
mod execution;
mod extensions;
mod helpers;
mod metadata;
mod report;
mod selection;

use std::collections::HashMap;
use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use cli::RunnerConfig;
use execution::{PreparedTest, RunOutcome, WorkerHandle};
use helpers::HelperCatalog;
use metadata::{parse_metadata, variants_for_metadata};
use report::{CategoryStats, SuiteReport, TestTiming};
use selection::{
    category_for_test, disabled_manifest, load_manifest, relative_test_path, select_test_paths,
    skip_decision,
};
use selection::{ExclusionManifest, SkipDecision};

struct PreparedSuite {
    candidate_total: usize,
    prepared: Vec<PreparedTest>,
    category_stats: HashMap<String, CategoryStats>,
    selected_counts: HashMap<String, usize>,
    variant_category_stats: HashMap<String, CategoryStats>,
    selected_variant_counts: HashMap<String, usize>,
    skip_reasons: HashMap<String, u32>,
    exclusion_reasons: HashMap<String, u32>,
    failures: Vec<String>,
}

impl PreparedSuite {
    fn selected_total(&self) -> usize {
        self.selected_counts.values().sum()
    }

    fn selected_variant_total(&self) -> usize {
        self.selected_variant_counts.values().sum()
    }
}

struct ExecutionResults {
    elapsed: Duration,
    failures: Vec<(String, String)>,
    outcomes: Vec<PreparedOutcome>,
}

#[derive(Clone, Debug)]
struct PreparedOutcome {
    index: usize,
    outcome: RunOutcome,
    elapsed: Duration,
}

struct SummaryView<'a> {
    options: &'a RunnerConfig,
    candidate_total: usize,
    selected_total: usize,
    selected_variant_total: usize,
    jobs: usize,
    elapsed: Duration,
    selected_counts: &'a HashMap<String, usize>,
    category_stats: &'a HashMap<String, CategoryStats>,
    selected_variant_counts: &'a HashMap<String, usize>,
    variant_category_stats: &'a HashMap<String, CategoryStats>,
    failures: &'a [String],
}

fn push_failure(failures_lock: &Mutex<Vec<(String, String)>>, category: &str, failure: String) {
    match failures_lock.lock() {
        Ok(mut failures) => failures.push((category.to_string(), failure)),
        Err(poisoned) => poisoned.into_inner().push((category.to_string(), failure)),
    }
}

fn push_outcome(
    outcomes_lock: &Mutex<Vec<PreparedOutcome>>,
    index: usize,
    outcome: RunOutcome,
    elapsed: Duration,
) {
    match outcomes_lock.lock() {
        Ok(mut outcomes) => outcomes.push(PreparedOutcome {
            index,
            outcome,
            elapsed,
        }),
        Err(poisoned) => poisoned.into_inner().push(PreparedOutcome {
            index,
            outcome,
            elapsed,
        }),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AggregatedOutcome {
    Pass,
    Fail,
    Panic,
}

impl AggregatedOutcome {
    fn from_run(outcome: &RunOutcome) -> Self {
        match outcome {
            RunOutcome::Pass => Self::Pass,
            RunOutcome::Fail(message) if message.starts_with("PANIC") => Self::Panic,
            RunOutcome::Fail(_) => Self::Fail,
        }
    }

    fn merge(self, next: Self) -> Self {
        match (self, next) {
            (Self::Panic, _) | (_, Self::Panic) => Self::Panic,
            (Self::Fail, _) | (_, Self::Fail) => Self::Fail,
            (Self::Pass, Self::Pass) => Self::Pass,
        }
    }

    fn record(self, stats: &mut CategoryStats) {
        match self {
            Self::Pass => stats.pass += 1,
            Self::Fail => stats.fail += 1,
            Self::Panic => stats.panic += 1,
        }
    }
}

fn record_execution_outcomes(suite: &mut PreparedSuite, outcomes: &[PreparedOutcome]) {
    let mut file_outcomes: HashMap<(PathBuf, String), AggregatedOutcome> = HashMap::new();

    for outcome in outcomes {
        let test = &suite.prepared[outcome.index];
        let category = test.category.clone();
        let file_key = (test.path.clone(), category.clone());
        let aggregated = AggregatedOutcome::from_run(&outcome.outcome);

        aggregated.record(suite.variant_category_stats.entry(category).or_default());
        file_outcomes
            .entry(file_key)
            .and_modify(|current| *current = current.merge(aggregated))
            .or_insert(aggregated);
    }

    for ((_, category), outcome) in file_outcomes {
        outcome.record(suite.category_stats.entry(category).or_default());
    }
}

pub fn main_entry() {
    let workspace_root = workspace_root_or_exit();
    let options = cli::parse_args();
    let helpers = load_helpers_or_exit(&workspace_root);

    if options.worker {
        execution::worker_main(&helpers);
    }

    let manifest = load_manifest_or_exit(&workspace_root, &options);
    let test_dir = helpers.test_dir();
    ensure_test_dir_or_exit(&test_dir);
    let mut suite = prepare_suite(&options, &test_dir, &manifest, &helpers);
    let selected_total = suite.selected_total();
    let selected_variant_total = suite.selected_variant_total();
    let skipped_total = suite
        .category_stats
        .values()
        .map(|stats| stats.skip)
        .sum::<u32>();
    let runnable_file_total = selected_total.saturating_sub(skipped_total as usize);
    let jobs = options.jobs.min(suite.prepared.len().max(1));
    eprintln!("Found {} candidate files", suite.candidate_total);
    eprintln!(
        "Prepared {} runnable variant executions for {} runnable files after {} explicit file skips",
        suite.prepared.len(),
        runnable_file_total,
        skipped_total
    );

    let timeout = Duration::from_millis(options.timeout_ms);
    let execution = execute_suite(&suite.prepared, &test_dir, timeout, jobs);
    for (_, failure) in execution.failures {
        suite.failures.push(failure);
    }
    record_execution_outcomes(&mut suite, &execution.outcomes);
    let timings = test_timings(&suite.prepared, &execution.outcomes, &test_dir);

    print_summary(&SummaryView {
        options: &options,
        candidate_total: suite.candidate_total,
        selected_total,
        selected_variant_total,
        jobs,
        elapsed: execution.elapsed,
        selected_counts: &suite.selected_counts,
        category_stats: &suite.category_stats,
        selected_variant_counts: &suite.selected_variant_counts,
        variant_category_stats: &suite.variant_category_stats,
        failures: &suite.failures,
    });

    report::write_report(&SuiteReport {
        report_path: &options.report_path,
        manifest: &manifest,
        filter: options.filter.as_deref(),
        no_skip: options.no_skip,
        proposal_stage: options.proposal_stage,
        jobs,
        timeout,
        candidate_total: suite.candidate_total,
        selected_total,
        selected_variant_total,
        selected_counts: &suite.selected_counts,
        category_stats: &suite.category_stats,
        selected_variant_counts: &suite.selected_variant_counts,
        variant_category_stats: &suite.variant_category_stats,
        skip_reasons: &suite.skip_reasons,
        exclusion_reasons: &suite.exclusion_reasons,
        failures: &suite.failures,
        timings: &timings,
        elapsed: execution.elapsed,
    });
}

fn workspace_root_or_exit() -> PathBuf {
    match Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
    {
        Ok(path) => path,
        Err(error) => {
            eprintln!("failed to resolve workspace root: {error}");
            std::process::exit(1);
        }
    }
}

fn load_helpers_or_exit(workspace_root: &Path) -> Arc<HelperCatalog> {
    match HelperCatalog::load(workspace_root) {
        Ok(helpers) => Arc::new(helpers),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}

fn load_manifest_or_exit(workspace_root: &Path, options: &RunnerConfig) -> ExclusionManifest {
    if options.no_skip {
        return disabled_manifest(&options.manifest_path);
    }

    match load_manifest(workspace_root, &options.manifest_path) {
        Ok(manifest) => manifest,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}

fn ensure_test_dir_or_exit(test_dir: &Path) {
    if test_dir.exists() {
        return;
    }
    eprintln!(
        "test262 test directory not found: {}\nClone test262 into testdata/test262",
        test_dir.display()
    );
    std::process::exit(1);
}

fn prepare_suite(
    options: &RunnerConfig,
    test_dir: &Path,
    manifest: &ExclusionManifest,
    helpers: &Arc<HelperCatalog>,
) -> PreparedSuite {
    let mut all_paths = match select_test_paths(options.filter.as_deref(), test_dir) {
        Ok(paths) => paths,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    };
    all_paths.sort();
    all_paths.dedup();

    let candidate_total = all_paths.len();
    let mut prepared = Vec::with_capacity(candidate_total);
    let mut category_stats: HashMap<String, CategoryStats> = HashMap::new();
    let mut selected_counts: HashMap<String, usize> = HashMap::new();
    let mut variant_category_stats: HashMap<String, CategoryStats> = HashMap::new();
    let mut selected_variant_counts: HashMap<String, usize> = HashMap::new();
    let mut skip_reasons: HashMap<String, u32> = HashMap::new();
    let mut exclusion_reasons: HashMap<String, u32> = HashMap::new();
    let mut failures = Vec::new();

    for path in &all_paths {
        let source = match fs::read_to_string(path) {
            Ok(source) => source,
            Err(error) => {
                failures.push(format!(
                    "{}: read error: {error}",
                    relative_test_path(path, test_dir)
                ));
                continue;
            }
        };
        let metadata = parse_metadata(&source);
        let category = category_for_test(path, test_dir);
        let variants = variants_for_metadata(&metadata);
        let variant_count = variants.len();

        match skip_decision(
            path,
            test_dir,
            manifest,
            &metadata,
            helpers,
            options.no_skip,
            options.proposal_stage,
        ) {
            Some(SkipDecision::ExcludedFromSelection(reason)) => {
                *exclusion_reasons.entry(reason).or_default() += 1;
                continue;
            }
            Some(SkipDecision::Skip(reason)) => {
                *selected_counts.entry(category.clone()).or_default() += 1;
                *selected_variant_counts.entry(category.clone()).or_default() += variant_count;
                category_stats.entry(category.clone()).or_default().skip += 1;
                variant_category_stats.entry(category).or_default().skip +=
                    u32::try_from(variant_count).unwrap_or(u32::MAX);
                *skip_reasons.entry(reason).or_default() += 1;
                continue;
            }
            None => {}
        }

        *selected_counts.entry(category.clone()).or_default() += 1;
        *selected_variant_counts.entry(category.clone()).or_default() += variant_count;
        for variant in variants {
            prepared.push(PreparedTest {
                path: path.clone(),
                category: category.clone(),
                metadata: metadata.clone(),
                variant,
            });
        }
        category_stats.entry(category.clone()).or_default();
        variant_category_stats.entry(category).or_default();
    }

    PreparedSuite {
        candidate_total,
        prepared,
        category_stats,
        selected_counts,
        variant_category_stats,
        selected_variant_counts,
        skip_reasons,
        exclusion_reasons,
        failures,
    }
}

fn execute_suite(
    prepared: &[PreparedTest],
    test_dir: &Path,
    timeout: Duration,
    jobs: usize,
) -> ExecutionResults {
    let start = Instant::now();
    let next_index = AtomicUsize::new(0);
    let done_count = AtomicU32::new(0);
    let pass_count = AtomicU32::new(0);
    let fail_count = AtomicU32::new(0);
    let panic_count = AtomicU32::new(0);
    let failures_lock: Mutex<Vec<(String, String)>> = Mutex::new(Vec::new());
    let outcomes_lock: Mutex<Vec<PreparedOutcome>> = Mutex::new(Vec::new());

    std::thread::scope(|scope| {
        for _ in 0..jobs {
            scope.spawn(|| {
                let mut worker = None;

                loop {
                    let index = next_index.fetch_add(1, Ordering::Relaxed);
                    if index >= prepared.len() {
                        break;
                    }

                    let test = &prepared[index];
                    if worker.is_none() {
                        worker = match WorkerHandle::spawn() {
                            Ok(handle) => Some(handle),
                            Err(error) => {
                                fail_count.fetch_add(1, Ordering::Relaxed);
                                let outcome = RunOutcome::Fail(error.clone());
                                push_failure(
                                    &failures_lock,
                                    &test.category,
                                    format!(
                                        "{}: {error}",
                                        relative_prepared_test_name(test, test_dir)
                                    ),
                                );
                                push_outcome(&outcomes_lock, index, outcome, Duration::ZERO);
                                let done = done_count.fetch_add(1, Ordering::Relaxed) + 1;
                                maybe_print_progress(
                                    done,
                                    prepared.len(),
                                    &pass_count,
                                    &fail_count,
                                    &panic_count,
                                    start.elapsed(),
                                );
                                continue;
                            }
                        };
                    }

                    let test_timeout = timeout_for_test(test, timeout);
                    let test_start = Instant::now();
                    let execution = worker
                        .as_mut()
                        .expect("worker should exist after spawn")
                        .run_test(test, test_timeout);
                    let test_elapsed = test_start.elapsed();
                    record_variant_execution(
                        &execution.outcome,
                        test,
                        test_dir,
                        &failures_lock,
                        &pass_count,
                        &fail_count,
                        &panic_count,
                    );
                    push_outcome(
                        &outcomes_lock,
                        index,
                        execution.outcome.clone(),
                        test_elapsed,
                    );

                    let should_recycle = worker.as_ref().is_some_and(WorkerHandle::should_recycle);
                    if !execution.reusable || should_recycle {
                        if let Some(mut stale_worker) = worker.take() {
                            stale_worker.shutdown();
                        }
                    }

                    let done = done_count.fetch_add(1, Ordering::Relaxed) + 1;
                    maybe_print_progress(
                        done,
                        prepared.len(),
                        &pass_count,
                        &fail_count,
                        &panic_count,
                        start.elapsed(),
                    );
                }

                if let Some(mut stale_worker) = worker {
                    stale_worker.shutdown();
                }
            });
        }
    });
    eprintln!();

    let failures = match failures_lock.into_inner() {
        Ok(failures) => failures,
        Err(poisoned) => poisoned.into_inner(),
    };
    let outcomes = match outcomes_lock.into_inner() {
        Ok(outcomes) => outcomes,
        Err(poisoned) => poisoned.into_inner(),
    };
    ExecutionResults {
        elapsed: start.elapsed(),
        failures,
        outcomes,
    }
}

fn test_timings(
    prepared: &[PreparedTest],
    outcomes: &[PreparedOutcome],
    test_dir: &Path,
) -> Vec<TestTiming> {
    let mut timings = Vec::with_capacity(outcomes.len());
    for outcome in outcomes {
        let test = &prepared[outcome.index];
        timings.push(TestTiming {
            file: relative_test_path(&test.path, test_dir),
            variant: test.variant.report_label().map(ToString::to_string),
            outcome: timing_outcome_label(&outcome.outcome).to_string(),
            elapsed: outcome.elapsed,
        });
    }
    timings
}

fn timing_outcome_label(outcome: &RunOutcome) -> &'static str {
    match outcome {
        RunOutcome::Pass => "pass",
        RunOutcome::Fail(message) if message.starts_with("PANIC") => "panic",
        RunOutcome::Fail(_) => "fail",
    }
}

fn record_variant_execution(
    outcome: &RunOutcome,
    test: &PreparedTest,
    test_dir: &Path,
    failures_lock: &Mutex<Vec<(String, String)>>,
    pass_count: &AtomicU32,
    fail_count: &AtomicU32,
    panic_count: &AtomicU32,
) {
    match outcome {
        RunOutcome::Pass => {
            pass_count.fetch_add(1, Ordering::Relaxed);
        }
        RunOutcome::Fail(message) => {
            if message.starts_with("PANIC") {
                panic_count.fetch_add(1, Ordering::Relaxed);
            } else {
                fail_count.fetch_add(1, Ordering::Relaxed);
            }
            push_failure(
                failures_lock,
                &test.category,
                format!("{}: {message}", relative_prepared_test_name(test, test_dir)),
            );
        }
    }
}

fn timeout_for_test(test: &PreparedTest, default: Duration) -> Duration {
    if is_exhaustive_uri_legacy_test(&test.path) {
        return default.max(Duration::from_secs(30));
    }
    if is_generated_regexp_test(test) || is_exhaustive_regexp_legacy_test(&test.path) {
        return default.max(Duration::from_secs(30));
    }
    default
}

fn is_generated_regexp_test(test: &PreparedTest) -> bool {
    let has_regexp_component = test
        .path
        .components()
        .any(|component| component.as_os_str() == "RegExp");
    has_regexp_component && test.metadata.flags.iter().any(|flag| flag == "generated")
        || test
            .path
            .components()
            .collect::<Vec<_>>()
            .windows(3)
            .any(|components| {
                components[0].as_os_str() == "RegExp"
                    && components[1].as_os_str() == "property-escapes"
                    && components[2].as_os_str() == "generated"
            })
}

fn is_exhaustive_regexp_legacy_test(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some(
            "character-class-escape-non-whitespace.js"
                | "S7.8.5_A1.1_T2.js"
                | "S7.8.5_A1.4_T2.js"
                | "S7.8.5_A2.1_T2.js"
                | "S7.8.5_A2.4_T2.js"
        )
    )
}

fn relative_prepared_test_name(test: &PreparedTest, test_dir: &Path) -> String {
    let mut name = relative_test_path(&test.path, test_dir);
    if let Some(label) = test.variant.report_label() {
        name.push_str(" [");
        name.push_str(label);
        name.push(']');
    }
    name
}

fn is_exhaustive_uri_legacy_test(path: &Path) -> bool {
    [
        "built-ins/encodeURI/S15.1.3.3_A2.3_T1.js",
        "built-ins/encodeURIComponent/S15.1.3.4_A2.3_T1.js",
        "built-ins/decodeURI/S15.1.3.1_A2.5_T1.js",
        "built-ins/decodeURIComponent/S15.1.3.2_A2.5_T1.js",
    ]
    .iter()
    .any(|suffix| path.ends_with(suffix))
}

fn maybe_print_progress(
    done: u32,
    total: usize,
    pass_count: &AtomicU32,
    fail_count: &AtomicU32,
    panic_count: &AtomicU32,
    elapsed: Duration,
) {
    if done.is_multiple_of(100) || usize::try_from(done).ok() == Some(total) {
        print_progress(done, total, pass_count, fail_count, panic_count, elapsed);
    }
}

fn print_progress(
    done: u32,
    total: usize,
    pass_count: &AtomicU32,
    fail_count: &AtomicU32,
    panic_count: &AtomicU32,
    elapsed: Duration,
) {
    let elapsed_secs = elapsed.as_secs_f64();
    let rate = if elapsed_secs > 0.0 {
        f64::from(done) / elapsed_secs
    } else {
        0.0
    };
    eprint!(
        "\r[{done}/{total}] pass={} fail={} panic={} ({rate:.0}/s)       ",
        pass_count.load(Ordering::Relaxed),
        fail_count.load(Ordering::Relaxed),
        panic_count.load(Ordering::Relaxed),
    );
    let _ = std::io::stderr().flush();
}

fn print_summary(summary: &SummaryView<'_>) {
    let total_pass = summary
        .category_stats
        .values()
        .map(|stats| stats.pass)
        .sum::<u32>();
    let total_fail = summary
        .category_stats
        .values()
        .map(report::CategoryStats::reported_failures)
        .sum::<u32>();
    let total_skip = summary
        .category_stats
        .values()
        .map(|stats| stats.skip)
        .sum::<u32>();
    let total_panic = summary
        .category_stats
        .values()
        .map(|stats| stats.panic)
        .sum::<u32>();
    let runnable = summary
        .category_stats
        .values()
        .map(report::CategoryStats::attempted)
        .sum::<u32>();
    let runnable_variants = summary
        .variant_category_stats
        .values()
        .map(report::CategoryStats::attempted)
        .sum::<u32>();

    println!("\n========== Lyng JS Whole-Suite Test262 ==========");
    println!("Candidate files:      {}", summary.candidate_total);
    println!("Selected files:       {}", summary.selected_total);
    println!("Runnable files:       {runnable}");
    println!("Passed files:         {total_pass}");
    println!("Failed files:         {total_fail}");
    println!("Panicked files:       {total_panic}");
    println!("Skipped files:        {total_skip}");
    println!("Selected variants:    {}", summary.selected_variant_total);
    println!("Runnable variants:    {runnable_variants}");
    println!(
        "Time:                 {:.1}s ({} threads)",
        summary.elapsed.as_secs_f64(),
        summary.jobs
    );
    println!(
        "Filter:               {}",
        summary.options.filter.as_deref().unwrap_or("whole corpus")
    );
    println!();

    println!("--- File Category Breakdown ---");
    let mut categories: Vec<_> = summary.category_stats.iter().collect();
    categories.sort_by(|left, right| left.0.cmp(right.0));
    for (category, stats) in categories {
        println!(
            "  {:<28} selected_files={:<5} runnable_files={:<5} pass={:<5} fail={:<5} skip={:<5} panic={:<5} ({}%)",
            category,
            summary.selected_counts.get(category).copied().unwrap_or(0),
            stats.attempted(),
            stats.pass,
            stats.reported_failures(),
            stats.skip,
            stats.panic,
            report::format_pass_rate(stats.pass_rate()),
        );
    }

    println!();
    println!("--- Variant Execution Breakdown ---");
    let mut variant_categories: Vec<_> = summary.variant_category_stats.iter().collect();
    variant_categories.sort_by(|left, right| left.0.cmp(right.0));
    for (category, stats) in variant_categories {
        println!(
            "  {:<28} selected_variants={:<5} runnable_variants={:<5} pass={:<5} fail={:<5} skip={:<5} panic={:<5} ({}%)",
            category,
            summary
                .selected_variant_counts
                .get(category)
                .copied()
                .unwrap_or(0),
            stats.attempted(),
            stats.pass,
            stats.reported_failures(),
            stats.skip,
            stats.panic,
            report::format_pass_rate(stats.pass_rate()),
        );
    }

    if summary.options.list_failures && !summary.failures.is_empty() {
        println!();
        println!("--- Failures ---");
        for failure in summary.failures {
            println!("  {failure}");
        }
    }
}

#[cfg(test)]
mod prepare_suite_tests {
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::cli::{RunnerConfig, DEFAULT_MANIFEST_PATH};
    use crate::helpers::HelperCatalog;
    use crate::selection::{disabled_manifest, ProposalStage};

    use super::prepare_suite;

    static TEMP_DIR_COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn make_temp_dir() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        let counter = TEMP_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "lyng-js-test262-prepare-{}-{}-{}",
            std::process::id(),
            nonce,
            counter
        ));
        fs::create_dir_all(&path).expect("temp dir should be created");
        path
    }

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("workspace root should exist")
    }

    fn options_for_filter(filter: &str) -> RunnerConfig {
        RunnerConfig {
            filter: Some(filter.to_string()),
            report_path: "/tmp/lyng-js-test262-test.md".to_string(),
            manifest_path: DEFAULT_MANIFEST_PATH.to_string(),
            no_skip: false,
            list_failures: false,
            jobs: 1,
            timeout_ms: 1000,
            proposal_stage: ProposalStage::Stage3,
            worker: false,
        }
    }

    #[test]
    fn prepare_suite_expands_default_script_tests_into_sloppy_and_strict_runs() {
        let root = make_temp_dir();
        let entry_path = root.join("default-script.js");
        fs::write(
            &entry_path,
            r"
            /*---
            description: default script should run in both modes
            ---*/
            assert.sameValue(1, 1);
            ",
        )
        .expect("fixture should be written");

        let helpers = Arc::new(HelperCatalog::load(&workspace_root()).expect("helper catalog"));
        let suite = prepare_suite(
            &options_for_filter(entry_path.to_str().expect("path should be utf-8")),
            &root,
            &disabled_manifest(DEFAULT_MANIFEST_PATH),
            &helpers,
        );

        assert_eq!(suite.prepared.len(), 2);
        assert_eq!(suite.selected_total(), 1);

        let _ = fs::remove_dir_all(root);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exhaustive_uri_legacy_tests_receive_extended_timeout() {
        let test = PreparedTest {
            path: PathBuf::from("test/built-ins/decodeURI/S15.1.3.1_A2.5_T1.js"),
            category: "built-ins".to_string(),
            metadata: parse_metadata(""),
            variant: crate::metadata::TestVariant::Default,
        };

        assert_eq!(
            timeout_for_test(&test, Duration::from_secs(1)),
            Duration::from_secs(30)
        );
    }

    #[test]
    fn ordinary_tests_keep_default_timeout() {
        let test = PreparedTest {
            path: PathBuf::from("test/built-ins/String/prototype/slice/basic.js"),
            category: "built-ins".to_string(),
            metadata: parse_metadata(""),
            variant: crate::metadata::TestVariant::Default,
        };

        assert_eq!(
            timeout_for_test(&test, Duration::from_secs(1)),
            Duration::from_secs(1)
        );
    }

    #[test]
    fn generated_regexp_tests_receive_extended_timeout() {
        let test = PreparedTest {
            path: PathBuf::from(
                "test/built-ins/RegExp/CharacterClassEscapes/character-class-digit-class-escape-negative-cases.js",
            ),
            category: "built-ins".to_string(),
            metadata: parse_metadata("/*---\nflags: [generated]\n---*/"),
            variant: crate::metadata::TestVariant::Default,
        };

        assert_eq!(
            timeout_for_test(&test, Duration::from_secs(1)),
            Duration::from_secs(30)
        );
    }

    #[test]
    fn generated_regexp_property_escape_tests_receive_extended_timeout() {
        let test = PreparedTest {
            path: PathBuf::from(
                "test/built-ins/RegExp/property-escapes/generated/General_Category_-_Mark.js",
            ),
            category: "built-ins".to_string(),
            metadata: parse_metadata("/*---\nfeatures: [regexp-unicode-property-escapes]\n---*/"),
            variant: crate::metadata::TestVariant::Default,
        };

        assert_eq!(
            timeout_for_test(&test, Duration::from_secs(1)),
            Duration::from_secs(30)
        );
    }

    #[test]
    fn exhaustive_regexp_legacy_tests_receive_extended_timeout() {
        let test = PreparedTest {
            path: PathBuf::from("test/built-ins/RegExp/character-class-escape-non-whitespace.js"),
            category: "built-ins".to_string(),
            metadata: parse_metadata("/*---\nesid: sec-characterclassescape\n---*/"),
            variant: crate::metadata::TestVariant::Default,
        };

        assert_eq!(
            timeout_for_test(&test, Duration::from_secs(1)),
            Duration::from_secs(30)
        );
    }

    #[test]
    fn exhaustive_regexp_literal_legacy_tests_receive_extended_timeout() {
        let test = PreparedTest {
            path: PathBuf::from("test/language/literals/regexp/S7.8.5_A1.1_T2.js"),
            category: "language".to_string(),
            metadata: parse_metadata("/*---\nes5id: 7.8.5_A1.1_T2\n---*/"),
            variant: crate::metadata::TestVariant::NonStrict,
        };

        assert_eq!(
            timeout_for_test(&test, Duration::from_secs(1)),
            Duration::from_secs(30)
        );
    }

    #[test]
    fn file_stats_fail_when_any_strictness_variant_fails() {
        let metadata = parse_metadata("");
        let path = PathBuf::from("test/language/default-script.js");
        let mut suite = PreparedSuite {
            candidate_total: 1,
            prepared: vec![
                PreparedTest {
                    path: path.clone(),
                    category: "language".to_string(),
                    metadata: metadata.clone(),
                    variant: crate::metadata::TestVariant::NonStrict,
                },
                PreparedTest {
                    path,
                    category: "language".to_string(),
                    metadata,
                    variant: crate::metadata::TestVariant::Strict,
                },
            ],
            category_stats: HashMap::new(),
            selected_counts: HashMap::from([("language".to_string(), 1)]),
            variant_category_stats: HashMap::new(),
            selected_variant_counts: HashMap::from([("language".to_string(), 2)]),
            skip_reasons: HashMap::new(),
            exclusion_reasons: HashMap::new(),
            failures: Vec::new(),
        };

        record_execution_outcomes(
            &mut suite,
            &[
                PreparedOutcome {
                    index: 0,
                    outcome: RunOutcome::Pass,
                    elapsed: Duration::from_millis(10),
                },
                PreparedOutcome {
                    index: 1,
                    outcome: RunOutcome::Fail("runtime error: Test262Error".to_string()),
                    elapsed: Duration::from_millis(20),
                },
            ],
        );

        let file_stats = suite
            .category_stats
            .get("language")
            .expect("file stats should be recorded");
        assert_eq!(file_stats.pass, 0);
        assert_eq!(file_stats.fail, 1);

        let variant_stats = suite
            .variant_category_stats
            .get("language")
            .expect("variant stats should be recorded");
        assert_eq!(variant_stats.pass, 1);
        assert_eq!(variant_stats.fail, 1);
    }
}
