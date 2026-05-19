use anyhow::Result;
use console::style;

use crate::config::limavel_config::LimavelConfig;
use crate::lima::client::LimaClient;

pub fn execute(name: &str) -> Result<()> {
    LimaClient::check_installed()?;

    let config = LimavelConfig::load(name)?;
    let instance = config.instance_name();

    if !LimaClient::instance_exists(instance)? {
        println!("{} No VM instance '{}' found. Run {} to create one.", style("ℹ").cyan(), instance, style(format!("limavel start {}", name)).cyan());
        return Ok(());
    }

    let status = LimaClient::instance_status(instance)?;

    println!("{}", style("Limavel Instance Status").bold());
    println!("{:<12} {}", style("Name:").bold(), instance);

    match status.as_str() {
        "Running" => println!("{:<12} {}", style("Status:").bold(), style(&status).green()),
        "Stopped" => println!("{:<12} {}", style("Status:").bold(), style(&status).yellow()),
        _ => println!("{:<12} {}", style("Status:").bold(), style(&status).red()),
    }

    // Show additional info if running
    if status == "Running" {
        let ip = LimaClient::guest_ip(instance).unwrap_or_else(|_| "N/A".to_string());
        println!("{:<12} {}", style("IP:").bold(), ip);
        println!("{:<12} {} MB", style("Memory:").bold(), config.memory);
        println!("{:<12} {}", style("CPUs:").bold(), config.cpus);

        if !config.sites.is_empty() {
            println!("\n{}", style("Sites:").bold());
            for site in &config.sites {
                println!("  {} → {} (PHP {})", style(&site.map).cyan(), site.to, site.php);
            }
        }

        if !config.databases.is_empty() {
            println!("\n{}", style("Databases:").bold());
            for db in &config.databases {
                println!("  {} ({})", style(db).cyan(), config.database.db_type);
            }
        }

        if !config.ports.is_empty() {
            println!("\n{}", style("Port Forwards:").bold());
            for port in &config.ports {
                println!("  localhost:{} → guest:{}", port.send, port.to);
            }
        }
    }

    Ok(())
}
