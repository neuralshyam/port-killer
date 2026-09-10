use std::thread;
use std::time::Duration;
use anyhow::{Context, Result};
use sysinfo::{Pid, System};

use crate::model::{KillMode, PortProcess};

#[cfg(unix)]
use nix::sys::signal::{kill as nix_kill, Signal};
#[cfg(unix)]
use nix::unistd::Pid as NixPid;

pub fn terminate_process(proc: &PortProcess, mode: KillMode, force: bool) -> Result<()> {
    if proc.pid == 0 {
        anyhow::bail!("Cannot terminate kernel/system socket without valid PID");
    }

    if proc.is_system_protected && !force {
        anyhow::bail!(
            "Process '{}' (PID {}) is protected. Use --force to override.",
            proc.name,
            proc.pid
        );
    }

    #[cfg(unix)]
    {
        let nix_pid = NixPid::from_raw(proc.pid as i32);

        match mode {
            KillMode::Force => {
                nix_kill(nix_pid, Signal::SIGKILL)
                    .with_context(|| format!("Failed to send SIGKILL to PID {}", proc.pid))?;
            }
            KillMode::Graceful => {
                // Try SIGTERM first
                if let Err(e) = nix_kill(nix_pid, Signal::SIGTERM) {
                    // If process already dead, success
                    if e != nix::errno::Errno::ESRCH {
                        return Err(e).with_context(|| format!("Failed to send SIGTERM to PID {}", proc.pid));
                    }
                    return Ok(());
                }

                // Wait up to 500ms to see if it exits cleanly
                let mut sys = System::new();
                for _ in 0..10 {
                    thread::sleep(Duration::from_millis(50));
                    sys.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[Pid::from(proc.pid as usize)]), true);
                    if sys.process(Pid::from(proc.pid as usize)).is_none() {
                        return Ok(());
                    }
                }

                // If still running, escalate to SIGKILL
                let _ = nix_kill(nix_pid, Signal::SIGKILL);
            }
        }
    }

    #[cfg(not(unix))]
    {
        let mut sys = System::new();
        sys.refresh_all();
        if let Some(p) = sys.process(Pid::from(proc.pid as usize)) {
            p.kill();
        }
    }

    Ok(())
}
