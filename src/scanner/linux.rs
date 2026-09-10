use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use sysinfo::{Pid, System, Users};

use crate::detector::detect_project_info;
use crate::model::{PortProcess, Protocol};

pub fn scan_listening_ports() -> Vec<PortProcess> {
    let mut sys = System::new_all();
    sys.refresh_all();
    let users = Users::new_with_refreshed_list();

    // Map: socket inode -> (port, protocol)
    let mut socket_map: HashMap<u64, (u16, Protocol)> = HashMap::new();

    if let Ok(content) = fs::read_to_string("/proc/net/tcp") {
        parse_proc_net_tcp(&content, Protocol::Tcp, &mut socket_map);
    }
    if let Ok(content) = fs::read_to_string("/proc/net/tcp6") {
        parse_proc_net_tcp(&content, Protocol::Tcp, &mut socket_map);
    }
    if let Ok(content) = fs::read_to_string("/proc/net/udp") {
        parse_proc_net_udp(&content, Protocol::Udp, &mut socket_map);
    }
    if let Ok(content) = fs::read_to_string("/proc/net/udp6") {
        parse_proc_net_udp(&content, Protocol::Udp, &mut socket_map);
    }

    // Map: socket inode -> PID
    let mut inode_to_pid: HashMap<u64, u32> = HashMap::new();
    let proc_dir = Path::new("/proc");

    if let Ok(entries) = fs::read_dir(proc_dir) {
        for entry in entries.flatten() {
            let file_name = entry.file_name();
            let name_str = file_name.to_string_lossy();
            if let Ok(pid) = name_str.parse::<u32>() {
                let fd_dir = entry.path().join("fd");
                if let Ok(fd_entries) = fs::read_dir(fd_dir) {
                    for fd_entry in fd_entries.flatten() {
                        if let Ok(link) = fs::read_link(fd_entry.path()) {
                            let link_str = link.to_string_lossy();
                            if link_str.starts_with("socket:[") && link_str.ends_with(']') {
                                let inode_str = &link_str[8..link_str.len() - 1];
                                if let Ok(inode) = inode_str.parse::<u64>() {
                                    if socket_map.contains_key(&inode) {
                                        inode_to_pid.insert(inode, pid);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let mut results: Vec<PortProcess> = Vec::new();
    let mut seen: HashSet<(u16, Protocol, u32)> = HashSet::new();

    for (inode, (port, protocol)) in socket_map {
        let pid = inode_to_pid.get(&inode).copied().unwrap_or(0);
        if seen.contains(&(port, protocol, pid)) {
            continue;
        }
        seen.insert((port, protocol, pid));

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
                let orphan = parent_pid == Some(1) && !is_protected_process(&proc_name, pid);
                let protected = is_protected_process(&proc_name, pid);
                (proc_name, cmd, mem, cpu_val, user_str, parent_pid, orphan, protected)
            } else {
                ("unknown".to_string(), String::new(), 0, 0.0, None, None, false, false)
            }
        } else {
            ("system/kernel".to_string(), String::new(), 0, 0.0, None, None, false, true)
        };

        let project_meta = if pid > 0 {
            detect_project_info(pid, &name, port)
        } else {
            detect_project_info(0, &name, port)
        };

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

    results.sort_by_key(|p| p.port);
    results
}

fn parse_proc_net_tcp(
    content: &str,
    protocol: Protocol,
    map: &mut HashMap<u64, (u16, Protocol)>,
) {
    for line in content.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 10 {
            continue;
        }
        let state = parts[3];
        if state != "0A" {
            continue;
        }
        let local_addr = parts[1];
        if let Some(port_hex) = local_addr.split(':').nth(1) {
            if let Ok(port) = u16::from_str_radix(port_hex, 16) {
                if let Ok(inode) = parts[9].parse::<u64>() {
                    map.insert(inode, (port, protocol));
                }
            }
        }
    }
}

fn parse_proc_net_udp(
    content: &str,
    protocol: Protocol,
    map: &mut HashMap<u64, (u16, Protocol)>,
) {
    for line in content.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 10 {
            continue;
        }
        let local_addr = parts[1];
        if let Some(port_hex) = local_addr.split(':').nth(1) {
            if let Ok(port) = u16::from_str_radix(port_hex, 16) {
                if let Ok(inode) = parts[9].parse::<u64>() {
                    map.insert(inode, (port, protocol));
                }
            }
        }
    }
}

fn is_protected_process(name: &str, pid: u32) -> bool {
    if pid <= 1 {
        return true;
    }
    let lower = name.to_lowercase();
    lower.contains("systemd")
        || lower.contains("init")
        || lower.contains("sshd")
        || lower.contains("dockerd")
        || lower.contains("containerd")
        || lower.contains("kthreadd")
}
