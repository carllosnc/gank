# Gank Roadmap

This document outlines the development roadmap for **Gank**, detailing the milestones for the **v0.1.0** initial release and future planned evolutions.

---

## Version v0.1.0 — "Genesis Release" (Focus: Stability & Core Capabilities)

The goal of version **v0.1.0** is to deliver a high-performance, visually immersive game aggregator and launcher ready for everyday desktop use on Windows.

### Delivery Checklist

#### 1. Core & Scanners (Completed)
- [x] **Steam** game detection (direct parsing of `.acf` manifests and additional `libraryfolders.vdf`).
- [x] **Epic Games Store** detection (parsing of local `.item` manifests).
- [x] **Ubisoft Connect** detection (Windows registry and game manifests).
- [x] **EA App / Origin** detection (local manifests and registry detection).
- [x] **Blizzard / Battle.net** detection (local product database and registry entries).
- [x] **Emulators & ROMs** detection (RetroArch integration, title cleaning, core association).
- [x] Silent background launcher auto-bootstrapping (Steam, Epic, EA, Ubisoft).
- [x] Game installation disk size resolution and human-readable formatting (GB/MB).

#### 2. User Interface & Experience (Completed)
- [x] Dynamic, responsive grid layout with official 2:3 vertical box art aspect ratio.
- [x] Native WGPU background with real-time liquid mesh gradient and procedural cosmic dust.
- [x] Smooth staggered cascade animation when revealing library cards.
- [x] Navigation filter pills with official high-resolution platform brand logos.
- [x] Cinematic game details screen (expanded hero poster, metadata, synopsis, developer, genres, disk size).
- [x] Custom cover replacement via drag-and-drop and clipboard paste (Ctrl+V).
- [x] Animated covers with video loop conversion (ffmpeg) and streaming playback.
- [x] Unified Design System with centralized tokens (Tailwind Blue, Inter Font, Phosphor Icons).

#### 3. Performance & Resource Efficiency (Completed)
- [x] Idle CPU usage < 0.4% at 60 FPS with VSync (`Fifo`).
- [x] Working set memory footprint reduced to ~32 MB via `mimalloc` and dynamic memory trimming.
- [x] *Zero-Allocation* rendering loop (slice filtering via borrowed references).
- [x] *Focus Throttling*: repaint rate throttled down to ~15 FPS when the window is unfocused.
- [x] *Viewport Culling*: off-screen cards bypass draw operations completely.
- [x] Downsampling on cover decoding (max 400x600) saving 55%–85% texture memory on HiDPI screens.
- [x] Persistent cover and metadata cache in local JSON and JPEGs.
- [x] Automated unit test suite (14 passing tests).

#### 4. Packaging & Release Infrastructure (Completed)
- [x] **Windows Application Resource Packaging:**
  - Embedded official `.ico` icon and Windows resource metadata into the `.exe` via `build.rs` / `winres`.
  - Windows Shell `SetCurrentProcessExplicitAppUserModelID` registration for immediate taskbar identity.
- [x] **Remote Repository:**
  - Published to GitHub: [https://github.com/carllosnc/gank](https://github.com/carllosnc/gank).

---

## Future Releases

### Version v0.2.0 — "Deep Integration"
- **GOG Galaxy & Xbox Game Pass (PC) Support:**
  - Parsing GOG's local SQLite database and Microsoft Store AppX package manifests.
- **Native Playtime Tracking:**
  - Background process monitoring to log and track hours played natively within Gank.
- **Custom Tags & Collections:**
  - User-defined collections ("Favorites", "Finish This Month", "Multiplayer", "Backlog").

### Version v0.3.0 — "Deck & Controller Experience"
- **Big Picture / Gamepad Mode:**
  - Full controller navigation (Xbox, PlayStation DualSense) with focused spatial input navigation.
- **Cross-Platform Support (Linux / Steam Deck):**
  - Adapting system calls, proton prefixes, and paths for native execution on SteamOS and Linux desktop environments.
