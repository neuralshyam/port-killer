pub mod config;
pub mod detector;
pub mod killer;
pub mod model;
pub mod pattern;
pub mod probe;
pub mod scanner;
pub mod tui;

use std::process::Command;
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;

use crate::killer::terminate_process;
use crate::model::KillMode;
use crate::pattern::resolve_port_patterns;
use crate::probe::{print_probe_result, probe_port};
use crate::scanner::{find_processes_on_port, scan_ports};

#[derive(Parser, Debug)]
#[command(
    name = "kport",
    version = env!("CARGO_PKG_VERSION"),
    about = "⚡ Ultra-fast CLI & TUI to hunt down and kill processes hogging your ports.",
    long_about = "kport (or port-killer) lets you inspect listening network sockets and immediately terminate blocking processes.\nSupports single ports, ranges (3000..3010), wildcards (80*), presets (:dev, :db), orphan/zombie sweeping, probing, and kport.toml project config."
)]
pub struct Cli {
    /// Port(s), ranges (3000..3010), wildcards (80*), or presets (:dev, :db)
    #[arg(value_name = "PORTS")]
    pub ports: Vec<String>,

    /// Force kill immediately (SIGKILL) instead of graceful SIGTERM
    #[arg(short, long)]
    pub force: bool,

    /// List all active listening ports in terminal table format and exit
    #[arg(short, long)]
    pub list: bool,

    /// Output in JSON format (usable with --list)
    #[arg(long)]
    pub json: bool,

    /// Probe a port with HTTP/TCP test and print latency/status/headers
    #[arg(long, value_name = "PORT")]
    pub probe: Option<u16>,

    /// Find and kill orphaned/zombie processes (parent process dead/PPID 1)
    #[arg(short, long)]
    pub zombies: bool,

    /// Execute a shell command after freeing the specified port(s)
    #[arg(long, value_name = "COMMAND")]
    pub exec: Option<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Reset all services defined in local kport.toml
    Reset,
    /// Check status of all services defined in local kport.toml
    Status,
    /// Probe a port (alias for --probe)
    Probe { port: u16 },
    /// Kill zombie / orphan background processes
    Zombies,
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    // Handle Subcommands
    if let Some(cmd) = cli.command {
        match cmd {
            Commands::Reset => return config::handle_config_reset(cli.force),
            Commands::Status => return config::handle_config_status(),
            Commands::Probe { port } => {
                let res = probe_port(port);
                print_probe_result(&res);
                return Ok(());
            }
            Commands::Zombies => return handle_zombies(cli.force),
        }
    }

    // Handle --probe
    if let Some(port) = cli.probe {
        let res = probe_port(port);
        print_probe_result(&res);
        return Ok(());
    }

    // Handle --zombies
    if cli.zombies {
        return handle_zombies(cli.force);
    }

    // Handle --list
    if cli.list {
        return print_ports_list(cli.json);
    }

    // Handle direct port kill with pattern support
    if !cli.ports.is_empty() {
        handle_direct_kill(&cli.ports, cli.force)?;

        // If --exec was specified, run the chained command
        if let Some(ref cmd_str) = cli.exec {
            println!("\n🚀 Executing: {}", cmd_str.cyan().bold());
            let status = Command::new("sh")
                .arg("-c")
                .arg(cmd_str)
                .status()
                .with_context(|| format!("Failed to execute command '{}'", cmd_str))?;
            std::process::exit(status.code().unwrap_or(0));
        }

        return Ok(());
    }

    // Default to interactive TUI
    tui::run_tui()
}

pub fn handle_direct_kill(patterns: &[String], force: bool) -> Result<()> {
    let all_ports = scan_ports();
    let target_ports = resolve_port_patterns(patterns, &all_ports);

    if target_ports.is_empty() {
        println!(
            "{} No ports matched pattern(s): {:?}",
            "ℹ".blue().bold(),
            patterns
        );
        return Ok(());
    }

    let mode = if force { KillMode::Force } else { KillMode::Graceful };

    for port in target_ports {
        let procs = find_processes_on_port(port);
        if procs.is_empty() {
            println!(
                "{} No active process found on port {}",
                "ℹ".blue().bold(),
                port.to_string().yellow().bold()
            );
            continue;
        }

        for proc in procs {
            if proc.pid == 0 {
                println!(
                    "{} Port {} is owned by kernel/system socket (PID 0), skipping.",
                    "⚠".yellow().bold(),
                    port
                );
                continue;
            }

            let label = proc.display_label();
            print!(
                "⚡ Killing '{}' ({}) on port {}... ",
                label.bold(),
                format!("PID {}", proc.pid).cyan(),
                port.to_string().yellow().bold()
            );

            match terminate_process(&proc, mode, force) {
                Ok(_) => {
                    println!("{}", "✓ Done".green().bold());
                }
                Err(e) => {
                    println!("{}", "✗ Failed".red().bold());
                    eprintln!("   {} {}", "Error:".red(), e);
                }
            }
        }
    }

    Ok(())
}

fn handle_zombies(force: bool) -> Result<()> {
    let all_ports = scan_ports();
    let orphans: Vec<_> = all_ports.into_iter().filter(|p| p.is_orphan).collect();

    if orphans.is_empty() {
        println!("{} No zombie/orphan processes found occupying ports!", "✓".green().bold());
        return Ok(());
    }

    println!("🧟 Found {} orphan process(es) occupying ports:\n", orphans.len());
    let mode = if force { KillMode::Force } else { KillMode::Graceful };

    for proc in orphans {
        print!(
            "  Port :{} | PID {} ({}) → Killing... ",
            proc.port.to_string().yellow(),
            proc.pid.to_string().cyan(),
            proc.name.bold()
        );
        match terminate_process(&proc, mode, force) {
            Ok(_) => println!("{}", "✓ Cleaned".green().bold()),
            Err(e) => println!("{} ({})", "✗ Failed".red().bold(), e),
        }
    }

    println!();
    Ok(())
}

pub fn print_ports_list(as_json: bool) -> Result<()> {
    let ports = scan_ports();

    if as_json {
        let json_str = serde_json::to_string_pretty(&ports)?;
        println!("{}", json_str);
        return Ok(());
    }

    if ports.is_empty() {
        println!("{}", "No listening ports found.".yellow());
        return Ok(());
    }

    println!(
        "\n{:<8} {:<8} {:<10} {:<20} {:<24} {}",
        "PORT".green().bold(),
        "PROTO".green().bold(),
        "PID".green().bold(),
        "PROCESS".green().bold(),
        "FRAMEWORK / PROJECT".green().bold(),
        "MEMORY".green().bold()
    );
    println!("{}", "─".repeat(80).dimmed());

    for p in ports {
        let port_str = if p.port < 1024 {
            p.port.to_string().yellow()
        } else {
            p.port.to_string().cyan()
        };

        let badge = if let Some(ref fw) = p.framework {
            format!("[{}]", fw).green().bold().to_string()
        } else if let Some(ref proj) = p.project_name {
            format!("[{}]", proj).cyan().to_string()
        } else {
            "-".dimmed().to_string()
        };

        println!(
            "{:<8} {:<8} {:<10} {:<20} {:<24} {}",
            port_str,
            p.protocol.to_string().dimmed(),
            p.pid.to_string().magenta(),
            p.name.white().bold(),
            badge,
            p.memory_human().blue()
        );
    }
    println!();

    Ok(())
}
