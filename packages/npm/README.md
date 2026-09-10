# ⚡ @neuralshyam/kport

> Ultra-fast CLI & TUI to hunt down and kill processes hogging your network ports. Written in pure Rust with zero external runtime dependencies.

[![npm](https://img.shields.io/npm/v/@neuralshyam/kport?color=66C2FF&style=flat-square)](https://www.npmjs.com/package/@neuralshyam/kport)
[![Crates.io](https://img.shields.io/crates/v/kport?color=66FFB2&style=flat-square)](https://crates.io/crates/kport)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg?style=flat-square)](https://github.com/neuralshyam/kport)

## ⚡ Quick Run (Zero Install)

```bash
# Using bunx (recommended)
bunx @neuralshyam/kport 3000

# Using npx
npx @neuralshyam/kport 3000
```

## 🚀 Install Globally

```bash
# Bun
bun add -g @neuralshyam/kport

# NPM
npm install -g @neuralshyam/kport

# Cargo
cargo install kport
```

## 🎯 Usage

```bash
# Kill whatever is running on port 3000
kport 3000

# Kill multiple ports & ranges
kport 3000 8080 5000..5010

# Kill by wildcard or preset (:dev, :db)
kport "80*" :dev

# Force kill (SIGKILL)
kport -f 3000

# Interactive TUI dashboard
kport

# List all listening ports (or --json)
kport -l

# Probe port with HTTP/TCP latency benchmark
kport --probe 3000

# Sweep orphaned zombie processes
kport --zombies
```

## 🌐 Links
- **Documentation & Benchmarks:** [https://neuralshyam.github.io/kport/](https://neuralshyam.github.io/kport/)
- **GitHub Repository:** [https://github.com/neuralshyam/kport](https://github.com/neuralshyam/kport)
- **Author:** [Shyam Charan Das](https://shyamcharan.pages.dev/)
