# AGENTS.md — AI Agent Guide and Repository Governance for Gank

This document defines the architecture, coding standards, design rules, and roadmap for the **Gank** project. Any AI agent or human developer collaborating on this repository must strictly adhere to these guidelines when proposing, modifying, or refactoring code.

---

## Project Overview

**Gank** is an ultra-fast, minimalist, and elegant desktop game aggregator and launcher developed in **Rust**, utilizing **egui** and **wgpu**.

- **Core Mission:** Unify fragmented game libraries (Steam, Epic Games, EA App, Ubisoft Connect, Battle.net, GOG, Xbox, and Emulators) into a single cinematic, responsive interface without the overhead of multiple store clients running in the background.
- **Competitive Advantage:** Idle CPU usage below 0.4%, cold startup under 90 milliseconds, immersive real-time WGPU procedural shaders, and zero allocations per frame in the main render loop.

---

## Architecture and Directory Structure

```text
gank/
├── assets/
│   ├── fonts/           # Inter-Regular and Inter-SemiBold (.otf)
│   └── icons/           # Official SVG and high-resolution PNG brand icons
├── src/
│   ├── main.rs          # Entry point, eframe setup, WGPU pipeline and Windows App ID
│   ├── models.rs        # Core data structures (Game, Platform, LaunchCommand)
│   ├── config.rs        # Application configuration and API keys persisted as JSON
│   ├── artwork.rs       # Local cover cache, ffmpeg animated loop conversion and thumbnailing
│   ├── metadata/        # Async metadata enrichment
│   │   ├── mod.rs       # Metadata manager and on-disk JSON cache
│   │   ├── rawg.rs      # RAWG API integration
│   │   ├── steam_store.rs # Lightweight Steam store web scraping
│   │   └── steamgrid.rs # SteamGridDB integration
│   ├── scanners/        # Local detection modules for installed game platforms
│   │   ├── mod.rs       # Multi-platform aggregator and popular games scanner
│   │   ├── steam.rs     # Steam VDF (.acf) manifest and libraryfolders parser
│   │   ├── epic.rs      # Epic Games manifest (.item) parser
│   │   ├── ubisoft.rs   # Ubisoft Connect registry and manifest reader
│   │   ├── ea.rs        # EA App / Origin manifest and registry reader
│   │   ├── battlenet.rs # Battle.net database (.db / registry) parser
│   │   ├── emulator.rs  # ROM and emulator scanner (RetroArch, RPCS3, etc.)
│   │   └── process.rs   # Process spawning utilities and safe execution
│   └── ui/              # Presentation layer and egui components
│       ├── mod.rs       # UI module re-exports
│       ├── app.rs       # Main rendering loop (GankApp) and game details view
│       ├── aurora_shader.rs # WGSL shaders for procedural cosmic dust and mesh gradient
│       ├── components.rs# Filter pills, action buttons, and standardized cards
│       ├── icons.rs     # Static images and Phosphor icon glyphs
│       └── theme.rs     # Design tokens (Tailwind Blue, typography, border radii)
├── AGENTS.md            # This governance and architecture guide
├── ROADMAP.md           # Detailed milestone roadmap for v0.1.0 and beyond
├── Cargo.toml           # Dependencies and release optimization profile
└── README.md            # Public repository documentation
```

---

## Performance Rules of Thumb (Anti-Regression)

1. **Zero-Allocation in the Main Render Loop (`update`):**
   - **Never** clone collections of `Game` or allocate dynamic `String`s every frame (16ms).
   - Use slices and borrowed references (`Vec<&Game>`). Cloning a `Game` must only happen when the user explicitly clicks to open the details modal or launch the game executable.
2. **Non-Blocking I/O (Thread Isolation):**
   - **Never** perform network requests (`ureq`), heavy disk scans (`fs::read_dir`), or image decodes directly inside the UI `update` method.
   - All I/O must be dispatched through `std::sync::mpsc` channels to background worker threads.
3. **In-Memory Caches and `OnceLock`:**
   - The cover directory path must be resolved once using `std::sync::OnceLock`.
   - Animated cover identifiers and cached disk mappings should be queried against in-memory `HashSet` / `HashMap` collections, avoiding repeated Windows syscalls every frame.
4. **Focus Throttling & Working Set Trimming:**
   - The app renders at 60 FPS with VSync (`Fifo`) when the window is focused.
   - When the window loses focus (e.g. while the user is in-game or browsing), the frame pacing is throttled to 15 FPS (66ms) to conserve CPU/GPU resources, and unused working set pages are trimmed.
5. **Mandatory Viewport Culling:**
   - Cards and posters outside the visible viewport (`!ui.is_rect_visible(rect)`) must immediately abort paint operations to avoid unnecessary GPU work.
6. **Smart Downsampling on Texture Loading:**
   - Covers must be downsampled to a maximum bounding box of 400x600 pixels upon decoding, reducing raw uncompressed texture memory by 55%–85% while preserving 2x HiDPI clarity.

---

## Design System and Visual Guidelines

- **Color Palette:**
  - Base: Midnight Velvet dark (`#080b12` to `#020408`) via native WGPU Mesh Gradient.
  - Primary Accent: **Tailwind Blue** (`BLUE_500`: `#3b82f6`, `BLUE_600`: `#2563eb`, `BLUE_700`: `#1d4ed8`).
  - Surfaces: Dark translucent glass (`rgba(18, 22, 30, alpha)`).
  - Filter Pills & Selectors: Clean borderless translucent capsule when active; no hard dividing lines (*no separator lines*).
- **Typography:**
  - Proportional Font: **Inter** (Regular for body text, SemiBold for headers and primary buttons).
  - Icons: **Phosphor Icons** integrated natively into egui font families + high-resolution official vector/raster platform logos.
- **Cover Aspect Ratio:**
  - Official 2:3 vertical aspect ratio (standard Steam/GOG box art: 600x900 / 300x450).
  - Smooth corner rounding of 8.0px for library cards and 12.0px for the hero poster in the details modal.

---

## Release Roadmap: Version v0.1.0

### Status (100% Completed):
- [x] Scanners for 6 platforms (Steam, Epic, Ubisoft, EA, Battle.net, Emulators).
- [x] Silent background launcher auto-bootstrapping (Steam, Epic, EA, Ubisoft).
- [x] Native WGSL procedural shader with floating cosmic dust and organic mesh gradient.
- [x] Cinematic game details screen (expanded hero poster, synopsis, developer, genres, disk size).
- [x] Custom cover art replacement with drag-and-drop and clipboard paste (Ctrl+V).
- [x] Animated covers with video loop conversion (ffmpeg) to 2:3 frames.
- [x] Zero-Allocation and focus throttling optimizations with idle CPU < 0.4%.
- [x] Windows executable resource packaging (.ico icon via build.rs/winres, window icon, and explicit AppUserModelID).
- [x] Low-RAM footprint (~32 MB) via `mimalloc` and dynamic memory trimming.
- [x] Remote repository setup on GitHub (https://github.com/carllosnc/gank).
