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

pub fn parse_args_from(args: &[String]) -> RunnerConfig {
    let mut filter = None;
    let mut report_path = DEFAULT_REPORT_PATH.to_string();
    let mut manifest_path = DEFAULT_MANIFEST_PATH.to_string();
    let mut no_skip = false;
    let mut list_failures = false;
    let mut timeout_ms = DEFAULT_TIMEOUT_MS;
    let mut proposal_stage = ProposalStage::Stage3;
    let mut worker = false;
    let mut jobs = std::thread::available_parallelism()
        .map(std::num::NonZeroUsize::get)
        .unwrap_or(4);
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--filter" => {
                index += 1;
                filter = args.get(index).cloned();
            }
            "--report" => {
                index += 1;
                report_path = args
                    .get(index)
                    .cloned()
                    .unwrap_or_else(|| DEFAULT_REPORT_PATH.to_string());
            }
            "--manifest" => {
                index += 1;
                manifest_path = args
                    .get(index)
                    .cloned()
                    .unwrap_or_else(|| DEFAULT_MANIFEST_PATH.to_string());
            }
            "--no-skip" => {
                no_skip = true;
            }
            "--list-failures" => {
                list_failures = true;
            }
            "--timeout-ms" => {
                index += 1;
                timeout_ms = args
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
                proposal_stage = parse_proposal_stage_arg(value).unwrap_or_else(|| {
                    eprintln!("Invalid proposal stage: {value}");
                    print_help();
                    std::process::exit(1);
                });
            }
            "--jobs" | "-j" => {
                index += 1;
                jobs = args
                    .get(index)
                    .and_then(|value| value.parse().ok())
                    .unwrap_or(4);
                if jobs == 0 {
                    jobs = 1;
                }
            }
            WORKER_FLAG => {
                worker = true;
            }
            arg if arg.starts_with("-j") && arg.len() > 2 => {
                if let Ok(parsed) = arg[2..].parse::<usize>() {
                    jobs = parsed.max(1);
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
                filter = Some(arg.to_string());
            }
        }
        index += 1;
    }

    RunnerConfig {
        filter,
        report_path,
        manifest_path,
        no_skip,
        list_failures,
        jobs,
        timeout_ms,
        proposal_stage,
        worker,
    }
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
