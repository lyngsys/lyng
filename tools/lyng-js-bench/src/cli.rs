#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Command {
    Help,
    Runtime(Vec<String>),
    Density(Vec<String>),
}

pub fn parse_command(args: &[String]) -> Result<Command, String> {
    match args.get(1).map(String::as_str) {
        None => Ok(Command::Runtime(Vec::new())),
        Some("--help" | "-h" | "help") => Ok(Command::Help),
        Some("runtime") => Ok(Command::Runtime(args[2..].to_vec())),
        Some("density") => Ok(Command::Density(args[2..].to_vec())),
        Some(other) => Err(format!(
            "Unknown benchmark suite: {other}\n\n{}",
            help_text()
        )),
    }
}

pub fn help_text() -> String {
    [
        "Usage: lyng-js-bench [runtime|density] [suite-options]",
        "",
        "Suites:",
        "  runtime  Lyng JS runtime, frontend, and memory benchmark report",
        "  density  Lyng JS bytecode-density and instruction-cache proxy report",
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
    fn top_level_help_uses_single_runner_language() {
        let help = help_text();
        assert!(help.contains("Usage: lyng-js-bench [runtime|density]"));
        assert!(!help.contains("phase"));
    }
}
