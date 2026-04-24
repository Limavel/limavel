mod ansible;
mod bootstrap;
mod cli;
mod commands;
mod config;
mod error;
mod hosts;
mod lima;

use clap::Parser;
use cli::{Cli, Commands};
use colored::Colorize;

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Init { name } => commands::init::execute(&name),
        Commands::Start { name, no_hosts } => commands::start::execute(&name, no_hosts),
        Commands::Stop { name, no_hosts } => commands::stop::execute(&name, no_hosts),
        Commands::Restart { name } => commands::restart::execute(&name),
        Commands::Provision { name } => commands::provision::execute(&name),
        Commands::Ssh { name } => commands::ssh::execute(&name),
        Commands::SshDetails { name } => commands::ssh::details(&name),
        Commands::Status { name } => commands::status::execute(&name),
        Commands::Edit { name } => commands::edit::execute(&name),
        Commands::Destroy { name } => commands::destroy::execute(&name),
        Commands::Publish { path } => commands::publish::execute(&path),
    };

    if let Err(e) = result {
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    }
}
