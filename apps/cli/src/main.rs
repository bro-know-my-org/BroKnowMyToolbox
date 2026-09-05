mod args;
mod commands;
mod error;
mod runtime;

use clap::Parser;

#[tokio::main]
async fn main() {
    let arguments: Vec<_> = std::env::args_os().collect();
    let cli = match args::Cli::try_parse_from(&arguments) {
        Ok(cli) => cli,
        Err(error) => {
            let json = arguments
                .iter()
                .skip(1)
                .take_while(|arg| *arg != "--")
                .any(|arg| arg == "--json");
            if error.use_stderr() && json {
                eprintln!(
                    "{}",
                    error::CliError::input("invalid_arguments", error.to_string(), true).render()
                );
                std::process::exit(2);
            }
            error.exit();
        }
    };
    if let Err(error) = commands::run(cli).await {
        eprintln!("{}", error.render());
        std::process::exit(error.exit_code);
    }
}
