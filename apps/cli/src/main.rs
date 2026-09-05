mod args;
mod commands;
mod error;
mod runtime;

use clap::Parser;

#[tokio::main]
async fn main() {
    if let Err(error) = commands::run(args::Cli::parse()).await {
        eprintln!("{}", error.render());
        std::process::exit(error.exit_code);
    }
}
