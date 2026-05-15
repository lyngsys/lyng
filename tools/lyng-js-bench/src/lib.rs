pub mod cli;
pub mod compare;
pub mod density;
pub mod runtime;
pub mod test262;
pub mod v8suite;

/// Dispatch the requested benchmark suite.
///
/// # Errors
///
/// Returns an error when CLI parsing fails or when the selected suite fails.
pub fn run(args: &[String]) -> Result<(), String> {
    match cli::parse_command(args)? {
        cli::Command::Help => {
            println!("{}", cli::help_text());
            Ok(())
        }
        cli::Command::Runtime(command_args) => runtime::run(&command_args),
        cli::Command::Density(command_args) => density::run(&command_args),
        cli::Command::Test262(command_args) => test262::run(&command_args),
        cli::Command::Compare(command_args) => compare::run(&command_args),
        cli::Command::V8Suite(command_args) => v8suite::run(&command_args),
    }
}
