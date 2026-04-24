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
use metadata::parse_metadata;
use report::{CategoryStats, SuiteReport};
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
    skip_reasons: HashMap<String, u32>,
    failures: Vec<String>,
}

impl PreparedSuite {
    fn selected_total(&self) -> usize {
        self.selected_counts.values().sum()
    }
}

struct ExecutionResults {
    elapsed: Duration,
    failures: Vec<(String, String)>,
}

struct SummaryView<'a> {
    options: &'a RunnerConfig,
    candidate_total: usize,
    selected_total: usize,
    jobs: usize,
    elapsed: Duration,
    selected_counts: &'a HashMap<String, usize>,
    category_stats: &'a HashMap<String, CategoryStats>,
    failures: &'a [String],
}

fn push_failure(failures_lock: &Mutex<Vec<(String, String)>>, category: &str, failure: String) {
    match failures_lock.lock() {
        Ok(mut failures) => failures.push((category.to_string(), failure)),
        Err(poisoned) => poisoned.into_inner().push((category.to_string(), failure)),
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
    let jobs = options.jobs.min(suite.candidate_total.max(1));
    eprintln!("Found {} candidate tests", suite.candidate_total);
    eprintln!(
        "Prepared {} runnable tests after {} explicit skips",
        suite.prepared.len(),
        selected_total.saturating_sub(suite.prepared.len())
    );

    let timeout = Duration::from_millis(options.timeout_ms);
    let execution = execute_suite(&suite.prepared, &test_dir, timeout, jobs);
    let mut failures_by_category: HashMap<String, (u32, u32)> = HashMap::new();
    for (category, failure) in execution.failures {
        let entry = failures_by_category.entry(category.clone()).or_default();
        if failure.contains("PANIC") {
            entry.1 += 1;
        } else {
            entry.0 += 1;
        }
        suite.failures.push(failure);
    }

    let mut pass_by_category: HashMap<String, u32> = HashMap::new();
    for test in &suite.prepared {
        *pass_by_category.entry(test.category.clone()).or_default() += 1;
    }
    for (category, (fail, panic)) in &failures_by_category {
        if let Some(pass) = pass_by_category.get_mut(category) {
            *pass = pass.saturating_sub(*fail + *panic);
        }
    }
    for (category, pass) in pass_by_category {
        suite.category_stats.entry(category).or_default().pass += pass;
    }
    for (category, (fail, panic)) in failures_by_category {
        let stats = suite.category_stats.entry(category).or_default();
        stats.fail += fail;
        stats.panic += panic;
    }

    print_summary(&SummaryView {
        options: &options,
        candidate_total: suite.candidate_total,
        selected_total,
        jobs,
        elapsed: execution.elapsed,
        selected_counts: &suite.selected_counts,
        category_stats: &suite.category_stats,
        failures: &suite.failures,
    });

    report::write_report(&SuiteReport {
        report_path: &options.report_path,
        manifest: &manifest,
        filter: options.filter.as_deref(),
        no_skip: options.no_skip,
        proposal_stage: options.proposal_stage,
        timeout,
        candidate_total: suite.candidate_total,
        selected_total,
        selected_counts: &suite.selected_counts,
        category_stats: &suite.category_stats,
        skip_reasons: &suite.skip_reasons,
        failures: &suite.failures,
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
    let mut skip_reasons: HashMap<String, u32> = HashMap::new();
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

        match skip_decision(
            path,
            test_dir,
            manifest,
            &metadata,
            helpers,
            options.no_skip,
            options.proposal_stage,
        ) {
            Some(SkipDecision::ExcludedFromSelection) => continue,
            Some(SkipDecision::Skip(reason)) => {
                *selected_counts.entry(category.clone()).or_default() += 1;
                category_stats.entry(category).or_default().skip += 1;
                *skip_reasons.entry(reason).or_default() += 1;
                continue;
            }
            None => {}
        }

        *selected_counts.entry(category.clone()).or_default() += 1;
        prepared.push(PreparedTest {
            path: path.clone(),
            category: category.clone(),
            metadata,
        });
        category_stats.entry(category).or_default();
    }

    PreparedSuite {
        candidate_total,
        prepared,
        category_stats,
        selected_counts,
        skip_reasons,
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
                                push_failure(
                                    &failures_lock,
                                    &test.category,
                                    format!(
                                        "{}: {error}",
                                        relative_test_path(&test.path, test_dir)
                                    ),
                                );
                                let done = done_count.fetch_add(1, Ordering::Relaxed) + 1;
                                if done.is_multiple_of(100)
                                    || usize::try_from(done).ok() == Some(prepared.len())
                                {
                                    print_progress(
                                        done,
                                        prepared.len(),
                                        &pass_count,
                                        &fail_count,
                                        &panic_count,
                                        start.elapsed(),
                                    );
                                }
                                continue;
                            }
                        };
                    }

                    let execution = worker
                        .as_mut()
                        .expect("worker should exist after spawn")
                        .run_test(test, timeout);
                    match &execution.outcome {
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
                                &failures_lock,
                                &test.category,
                                format!("{}: {message}", relative_test_path(&test.path, test_dir)),
                            );
                        }
                    }

                    let should_recycle = worker.as_ref().is_some_and(WorkerHandle::should_recycle);
                    if !execution.reusable || should_recycle {
                        if let Some(mut stale_worker) = worker.take() {
                            stale_worker.shutdown();
                        }
                    }

                    let done = done_count.fetch_add(1, Ordering::Relaxed) + 1;
                    if done.is_multiple_of(100)
                        || usize::try_from(done).ok() == Some(prepared.len())
                    {
                        print_progress(
                            done,
                            prepared.len(),
                            &pass_count,
                            &fail_count,
                            &panic_count,
                            start.elapsed(),
                        );
                    }
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
    ExecutionResults {
        elapsed: start.elapsed(),
        failures,
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

    println!("\n========== Lyng JS Whole-Suite Test262 ==========");
    println!("Candidate:      {}", summary.candidate_total);
    println!("Selected:       {}", summary.selected_total);
    println!("Runnable:       {runnable}");
    println!("Passed:         {total_pass}");
    println!("Failed:         {total_fail}");
    println!("Panicked:       {total_panic}");
    println!("Skipped:        {total_skip}");
    println!(
        "Time:           {:.1}s ({} threads)",
        summary.elapsed.as_secs_f64(),
        summary.jobs
    );
    println!(
        "Filter:         {}",
        summary.options.filter.as_deref().unwrap_or("whole corpus")
    );
    println!();

    println!("--- Category Breakdown ---");
    let mut categories: Vec<_> = summary.category_stats.iter().collect();
    categories.sort_by(|left, right| left.0.cmp(right.0));
    for (category, stats) in categories {
        println!(
            "  {:<28} selected={:<5} runnable={:<5} pass={:<5} fail={:<5} skip={:<5} panic={:<5} ({}%)",
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

    if summary.options.list_failures && !summary.failures.is_empty() {
        println!();
        println!("--- Failures ---");
        for failure in summary.failures {
            println!("  {failure}");
        }
    }
}
