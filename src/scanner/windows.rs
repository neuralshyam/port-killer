use std::process::Command;
use sysinfo::{Pid, System, Users};

use crate::detector::detect_project_info;
use crate::model::{PortProcess, Protocol};

pub fn scan_listening_ports() -> Vec<PortProcess> {
    let mut sys = System::new_all();
    sys.refresh_all();
    let users = Users::new_with_refreshed_list();

    let output = match Command::new("netstat").args(["-ano"]).output() {
        Ok(out) => out,
        Err(_) => return Vec::new(),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut results = Vec::new();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            continue;
        }

        let proto = parts[0].to_uppercase();
        let protocol = if proto == "TCP" {
            Protocol::Tcp
        } else if proto == "UDP" {
            Protocol::Udp
        } else {
            continue;
        };

        let local_addr = parts[1];
        let state = if protocol == Protocol::Tcp && parts.len() >= 4 {
            parts[3]
        } else {
            "LISTENING"
        };

        if protocol == Protocol::Tcp && state != "LISTENING" {
            continue;
        }

        let pid_str = parts.last().unwrap_or(&"0");
        let pid = pid_str.parse::<u32>().unwrap_or(0);

        if let Some(colon_idx) = local_addr.rfind(':') {
            if let Ok(port) = local_addr[colon_idx + 1..].parse::<u16>() {
                let (name, cmdline, memory, cpu, user, ppid, is_orphan, is_protected) = if pid > 0 {
                    if let Some(proc) = sys.process(Pid::from(pid as usize)) {
                        let proc_name = proc.name().to_string_lossy().to_string();
                        let cmd = proc
                            .cmd()
                            .iter()
                            .map(|s| s.to_string_lossy().to_string())
                            .collect::<Vec<_>>()
                            .join(" ");
                        let mem = proc.memory();
                        let cpu_val = proc.cpu_usage();
                        let user_str = proc
                            .user_id()
                            .and_then(|uid| users.get_user_by_id(uid))
                            .map(|u| u.name().to_string());
                        let parent_pid = proc.parent().map(|p| p.as_u32());
                        (proc_name, cmd, mem, cpu_val, user_str, parent_pid, false, false)
                    } else {
                        ("unknown".to_string(), String::new(), 0, 0.0, None, None, false, false)
                    }
                } else {
                    ("System".to_string(), String::new(), 0, 0.0, None, None, false, true)
                };

                let project_meta = detect_project_info(pid, &name, port);

                results.push(PortProcess {
                    port,
                    protocol,
                    pid,
                    ppid,
                    name,
                    cmdline,
                    cwd: project_meta.cwd,
                    project_name: project_meta.project_name,
                    framework: project_meta.framework,
                    memory_bytes: memory,
                    cpu_usage: cpu,
                    user,
                    is_system_protected: is_protected,
                    is_orphan,
                });
            }
        }
    }

    results.sort_by_key(|p| p.port);
    results.dedup_by(|a, b| a.port == b.port && a.pid == b.pid);
    results
}
