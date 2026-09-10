#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "windows")]
pub mod windows;

use crate::model::PortProcess;

pub fn scan_ports() -> Vec<PortProcess> {
    #[cfg(target_os = "linux")]
    {
        linux::scan_listening_ports()
    }

    #[cfg(target_os = "macos")]
    {
        macos::scan_listening_ports()
    }

    #[cfg(target_os = "windows")]
    {
        windows::scan_listening_ports()
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        Vec::new()
    }
}

pub fn find_processes_on_port(port: u16) -> Vec<PortProcess> {
    scan_ports().into_iter().filter(|p| p.port == port).collect()
}
