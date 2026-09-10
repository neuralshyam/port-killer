use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Protocol {
    Tcp,
    Udp,
}

impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Protocol::Tcp => write!(f, "TCP"),
            Protocol::Udp => write!(f, "UDP"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortProcess {
    pub port: u16,
    pub protocol: Protocol,
    pub pid: u32,
    pub ppid: Option<u32>,
    pub name: String,
    pub cmdline: String,
    pub cwd: Option<String>,
    pub project_name: Option<String>,
    pub framework: Option<String>,
    pub memory_bytes: u64,
    pub cpu_usage: f32,
    pub user: Option<String>,
    pub is_system_protected: bool,
    pub is_orphan: bool,
}

impl PortProcess {
    pub fn memory_human(&self) -> String {
        let bytes = self.memory_bytes;
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        if bytes >= GB {
            format!("{:.1} GB", bytes as f64 / GB as f64)
        } else if bytes >= MB {
            format!("{:.1} MB", bytes as f64 / MB as f64)
        } else if bytes >= KB {
            format!("{:.1} KB", bytes as f64 / KB as f64)
        } else {
            format!("{} B", bytes)
        }
    }

    pub fn display_label(&self) -> String {
        if let Some(ref fw) = self.framework {
            format!("{} ({})", self.name, fw)
        } else if let Some(ref proj) = self.project_name {
            format!("{} [{}]", self.name, proj)
        } else {
            self.name.clone()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KillMode {
    Graceful, // SIGTERM with fallback to SIGKILL
    Force,    // Immediate SIGKILL
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_human() {
        let mut p = PortProcess {
            port: 3000,
            protocol: Protocol::Tcp,
            pid: 1,
            ppid: None,
            name: "test".to_string(),
            cmdline: "".to_string(),
            cwd: None,
            project_name: None,
            framework: None,
            memory_bytes: 500,
            cpu_usage: 0.0,
            user: None,
            is_system_protected: false,
            is_orphan: false,
        };

        assert_eq!(p.memory_human(), "500 B");
        p.memory_bytes = 2048;
        assert_eq!(p.memory_human(), "2.0 KB");
        p.memory_bytes = 1048576 * 5;
        assert_eq!(p.memory_human(), "5.0 MB");
        p.memory_bytes = 1073741824 * 3;
        assert_eq!(p.memory_human(), "3.0 GB");
    }

    #[test]
    fn test_display_label() {
        let mut p = PortProcess {
            port: 3000,
            protocol: Protocol::Tcp,
            pid: 1,
            ppid: None,
            name: "node".to_string(),
            cmdline: "".to_string(),
            cwd: None,
            project_name: Some("my-app".to_string()),
            framework: Some("Next.js".to_string()),
            memory_bytes: 0,
            cpu_usage: 0.0,
            user: None,
            is_system_protected: false,
            is_orphan: false,
        };

        assert_eq!(p.display_label(), "node (Next.js)");
        p.framework = None;
        assert_eq!(p.display_label(), "node [my-app]");
        p.project_name = None;
        assert_eq!(p.display_label(), "node");
    }
}
