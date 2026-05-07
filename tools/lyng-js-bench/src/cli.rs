#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Command {
    Help,
    Runtime(Vec<String>),
    Density(Vec<String>),
    Test262(Vec<String>),
}

/// Parse the top-level benchmark suite selector.
///
/// # Errors
///
/// Returns an error when the requested suite is unknown.
pub fn parse_command(args: &[String]) -> Result<Command, String> {
    match args.get(1).map(String::as_str) {
        None => Ok(Command::Runtime(Vec::new())),
        Some("--help" | "-h" | "help") => Ok(Command::Help),
        Some("runtime") => Ok(Command::Runtime(args[2..].to_vec())),
        Some("density") => Ok(Command::Density(args[2..].to_vec())),
        Some("test262") => Ok(Command::Test262(args[2..].to_vec())),
        Some(other) => Err(format!(
            "Unknown benchmark suite: {other}\n\n{}",
            help_text()
        )),
    }
}

#[must_use]
pub fn help_text() -> String {
    [
        "Usage: lyng-js-bench [runtime|density|test262] [suite-options]",
        "",
        "Suites:",
        "  runtime  Lyng JS runtime, frontend, and memory benchmark report",
        "  density  Lyng JS bytecode-density and instruction-cache proxy report",
        "  test262  Test262 performance diagnostics for agents",
        "",
        "Run `lyng-js-bench <suite> --help` for suite-specific options.",
    ]
    .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|part| (*part).to_string()).collect()
    }

    #[test]
    fn defaults_to_runtime_suite_when_no_suite_is_provided() {
        assert_eq!(
            parse_command(&args(&["lyng-js-bench"])).unwrap(),
            Command::Runtime(vec![])
        );
    }

    #[test]
    fn parses_density_suite_without_phase_names() {
        assert_eq!(
            parse_command(&args(&["lyng-js-bench", "density", "--samples", "3"])).unwrap(),
            Command::Density(vec!["--samples".to_string(), "3".to_string()])
        );
    }

    #[test]
    fn parses_test262_suite_for_agent_diagnostics() {
        assert_eq!(
            parse_command(&args(&[
                "lyng-js-bench",
                "test262",
                "--filter",
                "built-ins/Date"
            ]))
            .unwrap(),
            Command::Test262(vec!["--filter".to_string(), "built-ins/Date".to_string()])
        );
    }

    #[test]
    fn top_level_help_uses_single_runner_language() {
        let help = help_text();
        assert!(help.contains("Usage: lyng-js-bench [runtime|density|test262]"));
        assert!(help.contains("test262  Test262 performance diagnostics for agents"));
        assert!(!help.contains("phase"));
    }
}
