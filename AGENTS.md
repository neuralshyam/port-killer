# AGENTS.md — Port Killer (`port-killer` / `kport`)

## Project Overview
`port-killer` is an ultra-fast, zero-dependency, cross-platform CLI and interactive TUI written in Rust to instantly find, inspect, and terminate processes occupying network ports.

### Key Objectives
- **Blazing Fast Startup & Execution:** Instant port scan (<5ms) using platform-native socket & process inspection (`procfs` on Linux, `sysctl`/`lsof` / `libproc` on macOS, `iphlpapi` on Windows).
- **Dual Mode Support:**
  1. **Direct Mode:** `port-killer <PORT>` (or `kport 3000`) for instant termination.
  2. **Interactive TUI Mode:** `port-killer` (or `kport`) displaying an interactive fuzzy-searchable list of active listening ports, process names, PIDs, memory usage, and user owners with quick-kill actions (Space/Enter).
- **Graceful + Force Termination:** Attempt graceful `SIGTERM` first with configurable timeout, escalating to `SIGKILL` only when necessary.
- **Developer Safety:** Protected ports safeguard (prevent accidental termination of critical system daemons like sshd, systemd, or docker-proxy unless explicitly forced with `-f`/`--force`).

---

## Architecture & Code Structure

```text
port-killer/
├── AGENTS.md
├── Cargo.toml
├── README.md
└── src/
    ├── main.rs          # CLI entry point, argument parsing (Clap)
    ├── model.rs         # Data structures (PortInfo, ProcessInfo, Protocol, KillResult)
    ├── scanner/         # Cross-platform port & process scanner
    │   ├── mod.rs       # Universal scanner trait & platform dispatcher
    │   ├── linux.rs     # Linux /proc/net/tcp, /proc/net/tcp6 and /proc/<pid>/fd parser
    │   ├── macos.rs     # macOS socket & process lookup
    │   └── fallback.rs  # Cross-platform fallback implementation
    ├── killer.rs        # Safe process termination logic (SIGTERM -> SIGKILL)
    └── tui/             # Interactive Ratatui interface
        ├── mod.rs       # TUI event loop & terminal handling
        ├── app.rs       # TUI state machine & filtering
        └── ui.rs        # Terminal rendering & styling
```

---

## Development & Execution Guidelines
- **Language:** Rust (2021 Edition or later).
- **Build / Run:** Use standard `cargo build`, `cargo test`, and `cargo run`.
- **Package Managers:** If any JS/web tooling is added, strictly use `bun` and `bunx`.
- **Code Quality:** Zero unsafe code unless strictly necessary for OS bindings, clear error messages with `thiserror` / `anyhow`.
