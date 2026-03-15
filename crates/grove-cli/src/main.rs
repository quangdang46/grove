use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "grove")]
#[command(about = "Autonomous orchestration for beads-backed Claude work")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    Init,
    Status,
    Inspect,
    Log,
    Retry,
    Run,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(command) => println!("{:?} is not implemented yet in the workspace skeleton.", command_name(&command)),
        None => println!("grove workspace skeleton is wired. Command implementations land in later beads."),
    }
}

fn command_name(command: &Command) -> &'static str {
    match command {
        Command::Init => "init",
        Command::Status => "status",
        Command::Inspect => "inspect",
        Command::Log => "log",
        Command::Retry => "retry",
        Command::Run => "run",
    }
}
