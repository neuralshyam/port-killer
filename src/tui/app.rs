use std::collections::HashSet;
use crate::killer::terminate_process;
use crate::model::{KillMode, PortProcess};
use crate::probe::{probe_port, ProbeResult};
use crate::scanner::scan_ports;

pub struct App {
    pub all_ports: Vec<PortProcess>,
    pub filtered_ports: Vec<PortProcess>,
    pub selected_index: usize,
    pub search_query: String,
    pub is_searching: bool,
    pub selected_to_kill: HashSet<u32>, // Multiple selection by PID
    pub status_message: Option<(String, bool)>, // (message, is_error)
    pub probe_result: Option<ProbeResult>,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        let mut app = Self {
            all_ports: Vec::new(),
            filtered_ports: Vec::new(),
            selected_index: 0,
            search_query: String::new(),
            is_searching: false,
            selected_to_kill: HashSet::new(),
            status_message: None,
            probe_result: None,
            should_quit: false,
        };
        app.refresh();
        app
    }

    pub fn refresh(&mut self) {
        self.all_ports = scan_ports();
        self.apply_filter();
        self.probe_result = None;
    }

    pub fn apply_filter(&mut self) {
        let q = self.search_query.trim().to_lowercase();
        if q.is_empty() {
            self.filtered_ports = self.all_ports.clone();
        } else {
            self.filtered_ports = self
                .all_ports
                .iter()
                .filter(|p| {
                    p.port.to_string().contains(&q)
                        || p.name.to_lowercase().contains(&q)
                        || p.pid.to_string().contains(&q)
                        || p.cmdline.to_lowercase().contains(&q)
                        || p.project_name.as_ref().map_or(false, |n| n.to_lowercase().contains(&q))
                        || p.framework.as_ref().map_or(false, |f| f.to_lowercase().contains(&q))
                })
                .cloned()
                .collect();
        }

        if self.filtered_ports.is_empty() {
            self.selected_index = 0;
        } else if self.selected_index >= self.filtered_ports.len() {
            self.selected_index = self.filtered_ports.len() - 1;
        }
    }

    pub fn next(&mut self) {
        if !self.filtered_ports.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.filtered_ports.len();
            self.probe_result = None;
        }
    }

    pub fn previous(&mut self) {
        if !self.filtered_ports.is_empty() {
            if self.selected_index > 0 {
                self.selected_index -= 1;
            } else {
                self.selected_index = self.filtered_ports.len() - 1;
            }
            self.probe_result = None;
        }
    }

    pub fn toggle_selection(&mut self) {
        if let Some(proc) = self.filtered_ports.get(self.selected_index) {
            if proc.pid > 0 {
                if self.selected_to_kill.contains(&proc.pid) {
                    self.selected_to_kill.remove(&proc.pid);
                } else {
                    self.selected_to_kill.insert(proc.pid);
                }
            }
        }
    }

    pub fn probe_current(&mut self) {
        if let Some(proc) = self.filtered_ports.get(self.selected_index) {
            let res = probe_port(proc.port);
            self.status_message = Some((
                format!(
                    "Probed :{} — {}",
                    proc.port,
                    res.http_status.as_deref().unwrap_or("Active TCP")
                ),
                false,
            ));
            self.probe_result = Some(res);
        }
    }

    pub fn kill_current(&mut self, force: bool) {
        if let Some(proc) = self.filtered_ports.get(self.selected_index).cloned() {
            let mode = if force {
                KillMode::Force
            } else {
                KillMode::Graceful
            };
            match terminate_process(&proc, mode, force) {
                Ok(_) => {
                    self.status_message = Some((
                        format!("Killed '{}' (PID {}) on port {}", proc.name, proc.pid, proc.port),
                        false,
                    ));
                    self.refresh();
                }
                Err(e) => {
                    self.status_message = Some((format!("Error: {}", e), true));
                }
            }
        }
    }

    pub fn kill_selected_batch(&mut self, force: bool) {
        if self.selected_to_kill.is_empty() {
            self.kill_current(force);
            return;
        }

        let pids: Vec<u32> = self.selected_to_kill.iter().copied().collect();
        let mut killed = 0;
        let mut errors = Vec::new();

        let mode = if force {
            KillMode::Force
        } else {
            KillMode::Graceful
        };

        for pid in pids {
            if let Some(proc) = self.all_ports.iter().find(|p| p.pid == pid) {
                match terminate_process(proc, mode, force) {
                    Ok(_) => killed += 1,
                    Err(e) => errors.push(format!("PID {}: {}", pid, e)),
                }
            }
        }

        self.selected_to_kill.clear();
        if errors.is_empty() {
            self.status_message = Some((format!("Successfully terminated {} process(es)", killed), false));
        } else {
            self.status_message = Some((
                format!("Terminated {} process(es). Errors: {}", killed, errors.join(", ")),
                true,
            ));
        }
        self.refresh();
    }
}
