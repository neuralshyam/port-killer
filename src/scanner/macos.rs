use std::process::Command;
use sysinfo::{Pid, System, Users};

use crate::detector::detect_project_info;
use crate::model::{PortProcess, Protocol};

pub fn scan_listening_ports() -> Vec<PortProcess> {
    let mut sys = System::new_all();
    sys.refresh_all();
    let users = Users::new_with_refreshed_list();

    let output = match Command::new("lsof")
        .args(["-iTCP", "-sTCP:LISTEN", "-n", "-P", "-F", "pPn"])
        .output()
    {
        Ok(out) => out,
        Err(_) => return Vec::new(),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut results = Vec::new();
    let mut current_pid: Option<u32> = None;

    for line in stdout.lines() {
        if line.starts_with('p') {
            current_pid = line[1..].parse::<u32>().ok();
        } else if line.starts_with('n') {
            // e.g. "n*:3000" or "n127.0.0.1:8080"
            if let Some(colon_idx) = line.rfind(':') {
                if let Ok(port) = line[colon_idx + 1..].parse::<u16>() {
                    let pid = current_pid.unwrap_or(0);
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
                            let orphan = parent_pid == Some(1);
                            (proc_name, cmd, mem, cpu_val, user_str, parent_pid, orphan, false)
                        } else {
                            ("unknown".to_string(), String::new(), 0, 0.0, None, None, false, false)
                        }
                    } else {
                        ("system".to_string(), String::new(), 0, 0.0, None, None, false, true)
                    };

                    let project_meta = detect_project_info(pid, &name, port);

                    results.push(PortProcess {
                        port,
                        protocol: Protocol::Tcp,
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
    }

    results.sort_by_key(|p| p.port);
    results.dedup_by(|a, b| a.port == b.port && a.pid == b.pid);
    results
}
