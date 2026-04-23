use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

struct FileResult {
    name: String,
    passed: usize,
    total: usize,
}

struct SuiteResult {
    suite: Suite,
    files: Vec<FileResult>,
}

impl SuiteResult {
    fn passed(&self) -> usize {
        self.files.iter().map(|file| file.passed).sum()
    }

    fn total(&self) -> usize {
        self.files.iter().map(|file| file.total).sum()
    }
}

#[allow(dead_code)]
mod tokenizer_suite {
    use super::{FileResult, Suite, SuiteResult};

    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../crates/html_parser/tests/tokenizer_tests.rs"
    ));

    pub fn run(dir: &Path, filter: Option<&str>) -> SuiteResult {
        let mut entries: Vec<_> = fs::read_dir(dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "test"))
            .collect();
        entries.sort_by_key(|e| e.file_name());

        let mut files = Vec::new();
        for entry in entries {
            let path = entry.path();
            let filename = path.file_name().unwrap().to_string_lossy().into_owned();
            if filter.is_some_and(|needle| !filename.contains(needle)) {
                continue;
            }
            let (passed, total) = run_test_file(&path);
            files.push(FileResult {
                name: filename,
                passed,
                total,
            });
        }

        SuiteResult {
            suite: Suite::Tokenizer,
            files,
        }
    }
}

#[allow(dead_code)]
mod tree_suite {
    use super::{FileResult, Suite, SuiteResult};

    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../crates/html_parser/tests/tree_tests.rs"
    ));

    pub fn run(dir: &Path, filter: Option<&str>) -> SuiteResult {
        let mut files = Vec::new();
        for path in collect_tree_test_files(dir) {
            let filename = path.strip_prefix(dir).unwrap().display().to_string();
            if filter.is_some_and(|needle| !filename.contains(needle)) {
                continue;
            }
            let (passed, total) = run_test_file(&path);
            files.push(FileResult {
                name: filename,
                passed,
                total,
            });
        }

        SuiteResult {
            suite: Suite::Tree,
            files,
        }
    }
}

#[allow(dead_code)]
mod serializer_suite {
    use super::{FileResult, Suite, SuiteResult};

    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../crates/html_parser/tests/serializer_tests.rs"
    ));

    pub fn run(dir: &Path, filter: Option<&str>) -> SuiteResult {
        let mut entries: Vec<_> = fs::read_dir(dir)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "test"))
            .collect();
        entries.sort_by_key(|entry| entry.file_name());

        let mut files = Vec::new();
        for entry in entries {
            let path = entry.path();
            let filename = path.file_name().unwrap().to_string_lossy().into_owned();
            if filter.is_some_and(|needle| !filename.contains(needle)) {
                continue;
            }
            let (passed, total) = run_serializer_file(&path);
            files.push(FileResult {
                name: filename,
                passed,
                total,
            });
        }

        SuiteResult {
            suite: Suite::Serializer,
            files,
        }
    }
}

#[allow(dead_code)]
mod encoding_suite {
    use super::{FileResult, Suite, SuiteResult};

    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../crates/html_parser/tests/encoding_tests.rs"
    ));

    pub fn run(dir: &Path, filter: Option<&str>) -> SuiteResult {
        let mut files = Vec::new();
        for path in collect_encoding_files(dir) {
            let filename = path.strip_prefix(dir).unwrap().display().to_string();
            if filter.is_some_and(|needle| !filename.contains(needle)) {
                continue;
            }
            let (passed, total) = run_encoding_file(&path);
            files.push(FileResult {
                name: filename,
                passed,
                total,
            });
        }

        SuiteResult {
            suite: Suite::Encoding,
            files,
        }
    }
}

#[derive(Clone, Copy)]
enum Suite {
    Tokenizer,
    Tree,
    Serializer,
    Encoding,
}

impl Suite {
    fn name(self) -> &'static str {
        match self {
            Self::Tokenizer => "tokenizer",
            Self::Tree => "tree",
            Self::Serializer => "serializer",
            Self::Encoding => "encoding",
        }
    }

    fn all() -> Vec<Self> {
        vec![
            Self::Tokenizer,
            Self::Tree,
            Self::Serializer,
            Self::Encoding,
        ]
    }

    fn parse(value: &str) -> Option<Self> {
        match value {
            "tokenizer" => Some(Self::Tokenizer),
            "tree" | "tree-construction" => Some(Self::Tree),
            "serializer" => Some(Self::Serializer),
            "encoding" => Some(Self::Encoding),
            _ => None,
        }
    }
}

fn suite_dir(root: &Path, suite: Suite) -> PathBuf {
    let base = root.join("testdata/html5lib");
    match suite {
        Suite::Tokenizer => base.join("tokenizer"),
        Suite::Tree => base.join("tree-construction"),
        Suite::Serializer => base.join("serializer"),
        Suite::Encoding => base.join("encoding"),
    }
}

fn run_suite(root: &Path, suite: Suite, filter: Option<&str>) -> SuiteResult {
    let dir = suite_dir(root, suite);
    if !dir.exists() {
        eprintln!("suite data not found: {}", dir.display());
        return SuiteResult {
            suite,
            files: Vec::new(),
        };
    }

    match suite {
        Suite::Tokenizer => tokenizer_suite::run(&dir, filter),
        Suite::Tree => tree_suite::run(&dir, filter),
        Suite::Serializer => serializer_suite::run(&dir, filter),
        Suite::Encoding => encoding_suite::run(&dir, filter),
    }
}

fn usage(program: &str) {
    eprintln!(
        "Usage: {program} [--suite <tokenizer|tree|serializer|encoding>] [--filter <substring>] [--report <path>]\n\
         Repeat --suite to run multiple suites. Defaults to all suites."
    );
}

fn timestamp_string() -> String {
    let output = Command::new("date")
        .arg("+%Y-%m-%d %H:%M:%S")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|value| value.trim().to_string());

    output.unwrap_or_else(|| "unknown-time".to_string())
}

fn default_report_path(workspace_root: &Path, timestamp: &str) -> PathBuf {
    let slug = timestamp.replace([' ', ':'], "-");
    workspace_root
        .join("reports/html")
        .join(format!("html5lib-{slug}.md"))
}

fn print_suite_result(result: &SuiteResult) {
    println!("{}:", result.suite.name());
    for file in &result.files {
        println!("  {}: {}/{}", file.name, file.passed, file.total);
    }
    println!();
}

fn write_report(
    report_path: &Path,
    timestamp: &str,
    filter: Option<&str>,
    results: &[SuiteResult],
) -> std::io::Result<()> {
    if let Some(parent) = report_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let total_passed: usize = results.iter().map(SuiteResult::passed).sum();
    let total_tests: usize = results.iter().map(SuiteResult::total).sum();
    let total_failed = total_tests.saturating_sub(total_passed);
    let total_rate = if total_tests == 0 {
        0.0
    } else {
        total_passed as f64 / total_tests as f64 * 100.0
    };

    let mut report = String::new();
    report.push_str(&format!("# html5lib report generated {timestamp}\n\n"));
    if let Some(filter) = filter {
        report.push_str(&format!("Filter: `{filter}`\n\n"));
    }
    report.push_str("| Suite | Pass | Fail | Rate |\n");
    report.push_str("|-------|-----:|-----:|-----:|\n");
    for result in results {
        let passed = result.passed();
        let total = result.total();
        let failed = total.saturating_sub(passed);
        let rate = if total == 0 {
            0.0
        } else {
            passed as f64 / total as f64 * 100.0
        };
        report.push_str(&format!(
            "| {} | {} | {} | {:.1}% |\n",
            result.suite.name(),
            passed,
            failed,
            rate
        ));
    }
    report.push_str(&format!(
        "| total | {} | {} | {:.1}% |\n\n",
        total_passed, total_failed, total_rate
    ));

    for result in results {
        report.push_str(&format!("## {}\n\n", result.suite.name()));
        report.push_str("| File | Pass | Fail | Rate |\n");
        report.push_str("|------|-----:|-----:|-----:|\n");
        for file in &result.files {
            let failed = file.total.saturating_sub(file.passed);
            let rate = if file.total == 0 {
                0.0
            } else {
                file.passed as f64 / file.total as f64 * 100.0
            };
            report.push_str(&format!(
                "| {} | {} | {} | {:.1}% |\n",
                file.name, file.passed, failed, rate
            ));
        }
        report.push('\n');
    }

    fs::write(report_path, report)
}

fn main() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root should exist");

    let mut suites = Vec::new();
    let mut filter = None;
    let mut report_path = None;
    let mut args = env::args();
    let program = args.next().unwrap_or_else(|| "html5lib_runner".to_string());

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--suite" => {
                let Some(value) = args.next() else {
                    usage(&program);
                    std::process::exit(2);
                };
                let Some(suite) = Suite::parse(&value) else {
                    eprintln!("unknown suite: {value}");
                    usage(&program);
                    std::process::exit(2);
                };
                suites.push(suite);
            }
            "--filter" => {
                let Some(value) = args.next() else {
                    usage(&program);
                    std::process::exit(2);
                };
                filter = Some(value);
            }
            "--report" => {
                let Some(value) = args.next() else {
                    usage(&program);
                    std::process::exit(2);
                };
                report_path = Some(PathBuf::from(value));
            }
            "--help" | "-h" => {
                usage(&program);
                return;
            }
            other => {
                eprintln!("unknown argument: {other}");
                usage(&program);
                std::process::exit(2);
            }
        }
    }

    if suites.is_empty() {
        suites = Suite::all();
    }

    let filter_ref = filter.as_deref();
    let timestamp = timestamp_string();
    let report_path =
        report_path.unwrap_or_else(|| default_report_path(&workspace_root, &timestamp));
    let mut results = Vec::new();

    for suite in suites {
        let result = run_suite(&workspace_root, suite, filter_ref);
        print_suite_result(&result);
        results.push(result);
    }

    if let Err(error) = write_report(&report_path, &timestamp, filter_ref, &results) {
        eprintln!("failed to write report {}: {error}", report_path.display());
        std::process::exit(1);
    }

    let total_passed: usize = results.iter().map(SuiteResult::passed).sum();
    let total_tests: usize = results.iter().map(SuiteResult::total).sum();
    let pass_rate = if total_tests == 0 {
        0.0
    } else {
        total_passed as f64 / total_tests as f64 * 100.0
    };

    println!("Summary: {total_passed}/{total_tests} ({pass_rate:.1}%)");
    println!("Report: {}", report_path.display());

    if total_passed != total_tests {
        std::process::exit(1);
    }
}
