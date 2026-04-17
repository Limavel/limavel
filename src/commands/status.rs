use anyhow::Result;
use colored::Colorize;

use crate::config::limavel_config::LimavelConfig;
use crate::lima::client::LimaClient;

pub fn execute(name: &str) -> Result<()> {
    LimaClient::check_installed()?;

    let config = LimavelConfig::load(name)?;
    let instance = config.instance_name();

    if !LimaClient::instance_exists(instance)? {
        println!("{} No VM instance '{}' found. Run {} to create one.", "ℹ".cyan(), instance, format!("limavel start {}", name).cyan());
        return Ok(());
    }

    let status = LimaClient::instance_status(instance)?;

    println!("{}", "Limavel Instance Status".bold());
    println!("{:<12} {}", "Name:".bold(), instance);

    match status.as_str() {
        "Running" => println!("{:<12} {}", "Status:".bold(), status.green()),
        "Stopped" => println!("{:<12} {}", "Status:".bold(), status.yellow()),
        _ => println!("{:<12} {}", "Status:".bold(), status.red()),
    }

    // Show additional info if running
    if status == "Running" {
        let ip = LimaClient::guest_ip(instance).unwrap_or_else(|_| "N/A".to_string());
        println!("{:<12} {}", "IP:".bold(), ip);
        println!("{:<12} {} MB", "Memory:".bold(), config.memory);
        println!("{:<12} {}", "CPUs:".bold(), config.cpus);

        if !config.sites.is_empty() {
            println!("\n{}", "Sites:".bold());
            for site in &config.sites {
                println!("  {} → {} (PHP {})", site.map.cyan(), site.to, site.php);
            }
        }

        if !config.databases.is_empty() {
            println!("\n{}", "Databases:".bold());
            for db in &config.databases {
                println!("  {} ({})", db.cyan(), config.database.db_type);
            }
        }

        if !config.ports.is_empty() {
            println!("\n{}", "Port Forwards:".bold());
            for port in &config.ports {
                println!("  localhost:{} → guest:{}", port.send, port.to);
            }
        }
    }

    Ok(())
}
