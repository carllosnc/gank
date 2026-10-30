# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.0] - 2026-10-30

### Added
- Multi-platform game scanners for Steam, Epic Games, EA App, Ubisoft Connect, Battle.net, and Emulators/ROMs (RetroArch).
- Native WGPU procedural background with real-time liquid mesh gradient and cosmic dust particles.
- Cinematic game details screen with expanded hero poster, developer metadata, release date, Metacritic score, synopsis, and disk size.
- Custom game cover replacement with drag-and-drop and clipboard paste (Ctrl+V).
- Animated cover support with automatic FFmpeg video loop conversion to 2:3 frames.
- Embedded Windows application icon (`.ico`) and manifest resources via `winres`.
- Windows Shell `SetCurrentProcessExplicitAppUserModelID` registration for immediate taskbar branding and grouping.
- Windows single-instance mutex protection (`CreateMutexW`) that automatically restores and focuses the existing Gank window instead of spawning duplicate instances.
- Official web landing page (`docs/index.html`) featuring a 1:1 WebGL cosmic dust and fluid mesh gradient shader.
- Performance benchmark script (`tests/benchmark.ps1`) for tracking startup time, CPU, and memory footprints.
- Automated packaging script (`scripts/package.ps1`) for generating portable release bundles with SHA256 checksums.
- Inno Setup installer (`scripts/installer.iss` and `scripts/build-installer.ps1`) with custom branded wizard sidebars.
- GitHub Actions CI/CD workflows for automated validation and GitHub Releases publishing.

### Performance
- Cold startup time under 90 milliseconds.
- Working set memory footprint reduced to ~32 MB using Microsoft's `mimalloc` global allocator and native memory trimming (`SetProcessWorkingSetSize`).
- Zero allocations per frame in the main render loop.
- Dynamic focus throttling reducing repaint rate to 15 FPS when unfocused.
- Viewport culling bypassing draw calls for off-screen cards.
- Smart thumbnailing capping decoded textures at 400x600 for a 55%–85% reduction in texture memory.

### Changed
- Filter pills updated to modern borderless translucent capsule styling.
- All documentation, roadmap, and architecture governance translated to English.
