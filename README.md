<div align="center">

<img src="assets/icon.png" alt="Gank Logo" width="80" height="80" />

# Gank

<p>
  <strong>Fast, lightweight, and modern desktop game shortcut hub and launcher written in Rust</strong><br>
  Built with <code>egui</code> and <code>wgpu</code> with zero allocations per frame and instant sub-100ms startup.
</p>

<p>
  <a href="https://github.com/carllosnc/gank/actions/workflows/ci.yml">
    <img src="https://github.com/carllosnc/gank/actions/workflows/ci.yml/badge.svg" alt="CI Status" />
  </a>
  <a href="https://github.com/carllosnc/gank/releases">
    <img src="https://img.shields.io/github/v/release/carllosnc/gank?color=3b82f6&logo=github" alt="Release" />
  </a>
  <a href="https://www.rust-lang.org/">
    <img src="https://img.shields.io/badge/Language-Rust-orange.svg?logo=rust" alt="Language: Rust" />
  </a>
  <a href="https://github.com/carllosnc/gank">
    <img src="https://img.shields.io/badge/Platform-Windows%2010%20%7C%2011-0078d4?logo=windows" alt="Platform: Windows" />
  </a>
  <a href="LICENSE">
    <img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License: MIT" />
  </a>
</p>

<br>

<img src="assets/preview.jpg" alt="Gank Game Hub Interface" width="100%" />

</div>

A fast, lightweight, and modern desktop game shortcut hub and launcher built in **Rust** with an uncompromising focus on **performance**, **minimal resource footprint**, and **instant startup**.

The goal of Gank is to unify your fragmented game libraries (Steam, Epic Games, EA App, Ubisoft Connect, Battle.net, GOG, Xbox, and Emulators) into a single, cohesive, and cinematic interface without the overhead of keeping multiple sluggish store launchers running in the background.

---

## Performance Principles

- **Zero Overhead at Idle**: Reactive user interface built with `egui` and rendered with `wgpu`. CPU usage drops below 0.4% when idle, and idle RAM stabilizes around ~30–40 MB with `mimalloc` and native working set trimming.
- **Lightweight Game Scanners**: Direct file system and registry manifest scanning (no slow network queries or heavyweight API initialization on startup).
- **Sub-100ms Startup**: Native compiled executable launching in under 90 milliseconds.
- **Direct Launching**: Instant game execution through native protocol URIs or direct executables with automatic background launcher bootstrapping.

---

## Tech Stack

- **Language**: [Rust](https://www.rust-lang.org/) (2021 Edition)
- **GUI & Graphics**: [egui](https://github.com/emilk/egui) / [eframe](https://github.com/emilk/egui/tree/master/crates/eframe) with [wgpu](https://wgpu.rs/) rendering backend
- **Memory Allocator**: [mimalloc](https://github.com/microsoft/mimalloc) (zero fragmentation, high throughput)
- **Platform Integrations**: Windows Registry (`winreg`), Win32 APIs, process utilities
- **Shaders**: Real-time WGSL procedural cosmic dust and organic mesh gradient

---

## Project Structure

```text
gank/
├── assets/              # Inter fonts, high-resolution official brand icons, preview
├── src/
│   ├── main.rs          # Entry point, eframe setup, WGPU pipeline and Windows App ID
│   ├── models.rs        # Core data structures (Game, Platform, LaunchCommand)
│   ├── config.rs        # Application configuration and API keys persisted to disk
│   ├── artwork.rs       # Cover art caching, ffmpeg video conversion and thumbnailing
│   ├── metadata/        # Async metadata enrichment (Steam Store, RAWG, SteamGridDB)
│   ├── scanners/        # Local scanners (Steam, Epic, EA, Ubisoft, Battle.net, Emulators)
│   └── ui/              # Egui components, WGSL shaders, design system and views
├── AGENTS.md            # AI agent guidelines, architecture rules and governance
├── ROADMAP.md           # Milestone roadmap for v0.1.0 and beyond
├── RELEASING.md         # Release procedure, quality gates, and publishing guide
├── CHANGELOG.md         # Documented release history and version notes
├── Cargo.toml
├── LICENSE
└── README.md
```

---

## Roadmap & Guidelines

- For release status and planned milestones, see [`ROADMAP.md`](ROADMAP.md).
- For release procedures and packaging instructions, see [`RELEASING.md`](RELEASING.md).
- For version history and change logs, see [`CHANGELOG.md`](CHANGELOG.md).
- For coding standards, anti-regression performance rules, and architecture governance, see [`AGENTS.md`](AGENTS.md).

---

## Getting Started

### Prerequisites
- [Rust & Cargo](https://rustup.rs/) (stable toolchain)
- Windows 10/11 (or Linux via Proton/Wine)

### Running in Development
```powershell
cargo run
```

### Building the Optimized Release Binary
```powershell
cargo build --release
```

The optimized production binary will be generated at `target/release/Gank.exe`.

---

## License

Distributed under the MIT License. See [`LICENSE`](LICENSE) for details.
