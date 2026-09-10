use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use anyhow::{Context, Result};
use colored::Colorize;
use serde::Deserialize;

use crate::killer::terminate_process;
use crate::model::KillMode;
use crate::scanner::scan_ports;

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct KportConfig {
    #[serde(default)]
    pub services: BTreeMap<String, u16>,
}

pub fn find_config_file() -> Option<PathBuf> {
    let mut curr = env::current_dir().ok()?;
    loop {
        let candidate = curr.join("kport.toml");
        if candidate.exists() {
            return Some(candidate);
        }
        let dot_candidate = curr.join(".kport.toml");
        if dot_candidate.exists() {
            return Some(dot_candidate);
        }
        if !curr.pop() {
            break;
        }
    }
    None
}

pub fn load_config() -> Result<Option<(PathBuf, KportConfig)>> {
    if let Some(path) = find_config_file() {
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        let config: KportConfig = toml::from_str(&content)
            .with_context(|| format!("Failed to parse TOML in {}", path.display()))?;
        Ok(Some((path, config)))
    } else {
        Ok(None)
    }
}

pub fn handle_config_reset(force: bool) -> Result<()> {
    match load_config()? {
        Some((path, config)) => {
            println!(
                "📁 Using config: {}",
                path.display().to_string().cyan().bold()
            );
            println!("⚡ Resetting configured services...\n");

            let ports = scan_ports();
            let mode = if force { KillMode::Force } else { KillMode::Graceful };

            for (service_name, port) in config.services {
                let matches: Vec<_> = ports.iter().filter(|p| p.port == port).collect();
                if matches.is_empty() {
                    println!(
                        "  {:<15} :{} → {}",
                        service_name.bold(),
                        port,
                        "Already Free".dimmed()
                    );
                } else {
                    for proc in matches {
                        print!(
                            "  {:<15} :{} (PID {}) → Killing... ",
                            service_name.bold(),
                            port,
                            proc.pid.to_string().cyan()
                        );
                        match terminate_process(proc, mode, force) {
                            Ok(_) => println!("{}", "✓ Freed".green().bold()),
                            Err(e) => println!("{} ({})", "✗ Failed".red().bold(), e),
                        }
                    }
                }
            }
            println!();
        }
        None => {
            println!(
                "{} No `kport.toml` found in current or parent directory.\nCreate one like:\n\n[services]\nfrontend = 3000\nbackend = 8000\ndatabase = 5432\n",
                "ℹ".blue().bold()
            );
        }
    }
    Ok(())
}

pub fn handle_config_status() -> Result<()> {
    match load_config()? {
        Some((path, config)) => {
            println!(
                "📁 Config: {}",
                path.display().to_string().cyan().bold()
            );
            println!(
                "\n{:<15} {:<8} {:<10} {:<15} {}",
                "SERVICE".green().bold(),
                "PORT".green().bold(),
                "STATUS".green().bold(),
                "PID".green().bold(),
                "DETAILS".green().bold()
            );
            println!("{}", "─".repeat(60).dimmed());

            let ports = scan_ports();
            for (service, port) in config.services {
                let found = ports.iter().find(|p| p.port == port);
                match found {
                    Some(proc) => {
                        let label = proc.display_label();
                        println!(
                            "{:<15} {:<8} {:<10} {:<15} {}",
                            service.white().bold(),
                            port.to_string().cyan(),
                            "ACTIVE".green().bold(),
                            proc.pid.to_string().magenta(),
                            label.dimmed()
                        );
                    }
                    None => {
                        println!(
                            "{:<15} {:<8} {:<10} {:<15} {}",
                            service.white().bold(),
                            port.to_string().yellow(),
                            "DOWN".dimmed(),
                            "-",
                            "-"
                        );
                    }
                }
            }
            println!();
        }
        None => {
            println!(
                "{} No `kport.toml` found in current or parent directory.",
                "ℹ".blue().bold()
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_kport_toml() {
        let sample = r#"
[services]
frontend = 3000
backend = 8080
postgres = 5432
"#;
        let config: KportConfig = toml::from_str(sample).unwrap();
        assert_eq!(config.services.get("frontend"), Some(&3000));
        assert_eq!(config.services.get("backend"), Some(&8080));
        assert_eq!(config.services.get("postgres"), Some(&5432));
    }
}
