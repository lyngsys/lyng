pub mod cli;
pub mod density;
pub mod runtime;

pub fn run(args: &[String]) -> Result<(), String> {
    match cli::parse_command(args)? {
        cli::Command::Help => {
            println!("{}", cli::help_text());
            Ok(())
        }
        cli::Command::Runtime(command_args) => runtime::run(&command_args),
        cli::Command::Density(command_args) => density::run(&command_args),
    }
}
