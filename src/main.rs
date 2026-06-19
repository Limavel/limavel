mod ansible;
mod bootstrap;
mod cli;
mod commands;
mod config;
mod error;
mod hosts;
mod lima;

use clap::{CommandFactory, Parser};
use cli::{Cli, Commands};
use console::style;
use config::limavel_config::LimavelConfig;

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Init { name } => commands::init::execute(&name),
        Commands::Start { name, no_hosts } => {
            LimavelConfig::resolve(name).and_then(|n| commands::start::execute(&n, no_hosts))
        }
        Commands::Stop { name, no_hosts } => {
            LimavelConfig::resolve(name).and_then(|n| commands::stop::execute(&n, no_hosts))
        }
        Commands::Restart { name } => {
            LimavelConfig::resolve(name).and_then(|n| commands::restart::execute(&n))
        }
        Commands::Provision { name, no_edit } => {
            LimavelConfig::resolve(name).and_then(|n| commands::provision::execute(&n, no_edit))
        }
        Commands::Ssh { name } => {
            LimavelConfig::resolve(name).and_then(|n| commands::ssh::execute(&n))
        }
        Commands::Exec { name, c } => {
            LimavelConfig::resolve(name).and_then(|n| commands::exec::execute(&n, &c))
        }
        Commands::SshDetails { name } => {
            LimavelConfig::resolve(name).and_then(|n| commands::ssh::details(&n))
        }
        Commands::Status { name } => {
            LimavelConfig::resolve(name).and_then(|n| commands::status::execute(&n))
        }
        Commands::Edit { name } => {
            LimavelConfig::resolve(name).and_then(|n| commands::edit::execute(&n))
        }
        Commands::Destroy { name } => {
            LimavelConfig::resolve(name).and_then(|n| commands::destroy::execute(&n))
        }
        Commands::Publish { path } => commands::publish::execute(&path),
        Commands::Completions { shell } => {
            shell.generate(&mut Cli::command(), &mut std::io::stdout());
            Ok(())
        }
    };

    if let Err(e) = result {
        eprintln!("{} {}", style("Error:").red().bold(), e);
        std::process::exit(1);
    }
}
