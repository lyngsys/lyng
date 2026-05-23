use crate::selection::ProposalStage;

use std::env;

pub const DEFAULT_REPORT_PATH: &str = "reports/js/lyng-js/test262.md";
pub const DEFAULT_MANIFEST_PATH: &str = "reports/js/lyng-js/test262-exclusions.txt";
pub const DEFAULT_TIMEOUT_MS: u64 = 1_000;
pub const WORKER_FLAG: &str = "--worker";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunnerConfig {
    pub(crate) filter: Option<String>,
    pub(crate) report_path: String,
    pub(crate) manifest_path: String,
    pub(crate) no_skip: bool,
    pub(crate) list_failures: bool,
    pub(crate) jobs: usize,
    pub(crate) timeout_ms: u64,
    pub(crate) proposal_stage: ProposalStage,
    pub(crate) worker: bool,
}

pub fn parse_args() -> RunnerConfig {
    let args: Vec<String> = env::args().skip(1).collect();
    parse_args_from(&args)
}

struct ParsedArgs {
    filter: Option<String>,
    report_path: String,
    manifest_path: String,
    no_skip: bool,
    list_failures: bool,
    timeout_ms: u64,
    proposal_stage: ProposalStage,
    worker: bool,
    jobs: usize,
}

impl ParsedArgs {
    fn new() -> Self {
        Self {
            filter: None,
            report_path: DEFAULT_REPORT_PATH.to_string(),
            manifest_path: DEFAULT_MANIFEST_PATH.to_string(),
            no_skip: false,
            list_failures: false,
            timeout_ms: DEFAULT_TIMEOUT_MS,
            proposal_stage: ProposalStage::Stage3,
            worker: false,
            jobs: std::thread::available_parallelism()
                .map(std::num::NonZeroUsize::get)
                .unwrap_or(4),
        }
    }

    fn into_config(self) -> RunnerConfig {
        RunnerConfig {
            filter: self.filter,
            report_path: self.report_path,
            manifest_path: self.manifest_path,
            no_skip: self.no_skip,
            list_failures: self.list_failures,
            jobs: self.jobs,
            timeout_ms: self.timeout_ms,
            proposal_stage: self.proposal_stage,
            worker: self.worker,
        }
    }
}

pub fn parse_args_from(args: &[String]) -> RunnerConfig {
    let mut parsed = ParsedArgs::new();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--filter" => {
                index += 1;
                parsed.filter = args.get(index).cloned();
            }
            "--report" => {
                index += 1;
                parsed.report_path = args
                    .get(index)
                    .cloned()
                    .unwrap_or_else(|| DEFAULT_REPORT_PATH.to_string());
            }
            "--manifest" => {
                index += 1;
                parsed.manifest_path = args
                    .get(index)
                    .cloned()
                    .unwrap_or_else(|| DEFAULT_MANIFEST_PATH.to_string());
            }
            "--no-skip" => {
                parsed.no_skip = true;
            }
            "--list-failures" => {
                parsed.list_failures = true;
            }
            "--timeout-ms" => {
                index += 1;
                parsed.timeout_ms = args
                    .get(index)
                    .and_then(|value| value.parse().ok())
                    .unwrap_or(DEFAULT_TIMEOUT_MS)
                    .max(1);
            }
            "--proposal-stage" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    eprintln!("Missing value for --proposal-stage");
                    print_help();
                    std::process::exit(1);
                };
                parsed.proposal_stage = parse_proposal_stage_arg(value).unwrap_or_else(|| {
                    eprintln!("Invalid proposal stage: {value}");
                    print_help();
                    std::process::exit(1);
                });
            }
            "--jobs" | "-j" => {
                index += 1;
                parsed.jobs = args
                    .get(index)
                    .and_then(|value| value.parse().ok())
                    .unwrap_or(4);
                if parsed.jobs == 0 {
                    parsed.jobs = 1;
                }
            }
            WORKER_FLAG => {
                parsed.worker = true;
            }
            arg if arg.starts_with("-j") && arg.len() > 2 => {
                if let Ok(job_count) = arg[2..].parse::<usize>() {
                    parsed.jobs = job_count.max(1);
                }
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            arg if arg.starts_with("--") => {
                eprintln!("Unknown flag: {arg}");
                print_help();
                std::process::exit(1);
            }
            arg => {
                parsed.filter = Some(arg.to_string());
            }
        }
        index += 1;
    }

    parsed.into_config()
}

pub fn parse_proposal_stage_arg(value: &str) -> Option<ProposalStage> {
    match value {
        "4" => Some(ProposalStage::Stage4),
        "3" => Some(ProposalStage::Stage3),
        "2.7" => Some(ProposalStage::Stage2_7),
        _ => None,
    }
}

pub fn print_help() {
    eprintln!(
        "Usage: lyng-js-test262 [--filter <path-or-fragment>] [--report <path>] [--manifest <path>] [--proposal-stage <4|3|2.7>] [--no-skip] [--list-failures] [--timeout-ms <N>] [-j <N>]"
    );
}

#[cfg(test)]
mod tests {
    use crate::selection::ProposalStage;

    use super::{
        parse_args_from, parse_proposal_stage_arg, RunnerConfig, DEFAULT_MANIFEST_PATH,
        DEFAULT_TIMEOUT_MS, WORKER_FLAG,
    };

    #[test]
    fn parse_args_supports_whole_suite_runner_flags() {
        let options = parse_args_from(&[
            "--filter".to_string(),
            "built-ins/Array".to_string(),
            "--report".to_string(),
            "/tmp/test262.md".to_string(),
            "--manifest".to_string(),
            "/tmp/manifest.txt".to_string(),
            "--no-skip".to_string(),
            "--list-failures".to_string(),
            "--timeout-ms".to_string(),
            "250".to_string(),
            "--proposal-stage".to_string(),
            "2.7".to_string(),
            "-j8".to_string(),
        ]);

        assert_eq!(
            options,
            RunnerConfig {
                filter: Some("built-ins/Array".to_string()),
                report_path: "/tmp/test262.md".to_string(),
                manifest_path: "/tmp/manifest.txt".to_string(),
                no_skip: true,
                list_failures: true,
                jobs: 8,
                timeout_ms: 250,
                proposal_stage: ProposalStage::Stage2_7,
                worker: false,
            }
        );
    }

    #[test]
    fn parse_args_supports_worker_mode() {
        let options = parse_args_from(&[WORKER_FLAG.to_string()]);

        assert!(options.worker);
        assert_eq!(options.timeout_ms, DEFAULT_TIMEOUT_MS);
    }

    #[test]
    fn parse_args_defaults_to_stage_3_proposal_policy() {
        let options = parse_args_from(&[]);

        assert_eq!(options.proposal_stage, ProposalStage::Stage3);
    }

    #[test]
    fn parse_proposal_stage_arg_accepts_supported_policy_values() {
        assert_eq!(parse_proposal_stage_arg("4"), Some(ProposalStage::Stage4));
        assert_eq!(parse_proposal_stage_arg("3"), Some(ProposalStage::Stage3));
        assert_eq!(
            parse_proposal_stage_arg("2.7"),
            Some(ProposalStage::Stage2_7)
        );
    }

    #[test]
    fn parse_proposal_stage_arg_rejects_invalid_policy_values() {
        assert_eq!(parse_proposal_stage_arg("2"), None);
        assert_eq!(parse_proposal_stage_arg("stage3"), None);
    }

    #[test]
    fn parse_args_defaults_to_active_test262_manifest_path() {
        let options = parse_args_from(&[]);

        assert_eq!(options.manifest_path, DEFAULT_MANIFEST_PATH);
    }

    #[test]
    fn parse_args_defaults_timeout_to_one_second() {
        let options = parse_args_from(&[]);

        assert_eq!(options.timeout_ms, 1_000);
    }
}
