# Release Guide & Procedure — Gank

This document defines the official, standardized release workflow for **Gank**. All contributors and AI agents must adhere strictly to these procedures when preparing, verifying, and publishing releases.

---

## 1. Release Lifecycle & Architecture

The Gank publishing process follows five sequential phases:

```text
[1. Verification & Tests] ➔ [2. PR & Merge to Master] ➔ [3. Version Bump & Changelog] ➔ [4. Local Packaging] ➔ [5. Git Tag & CI/CD]
```

CI/CD on **GitHub Actions** (`.github/workflows/release.yml`) triggers automatically on any tag matching the semantic pattern `v*` (e.g. `v0.1.0`), compiling and publishing checksummed release bundles for Windows.

---

## 2. Branching & PR Strategy for Releases

1. **Working Branches**: Release preparation work (code refinement, documentation, packaging scripts) should occur on a dedicated branch (e.g., `release/v0.1.0` or `release/vX.Y`).
2. **Merge Requirement**: The branch **MUST** be submitted via Pull Request and merged into `master` **before** creating any release tag.
3. **Never Tag Unmerged Branches**: Official release tags must always point to a commit on `master`. Tagging unmerged feature or release branches is strictly prohibited.

---

## 3. Strict Semantic Versioning (SemVer) & Tag Syntax

The project adheres to [Semantic Versioning (SemVer 2.0.0)](https://semver.org/).

### Tag Format
- **Allowed**: `v<MAJOR>.<MINOR>.<PATCH>` (e.g., `v0.1.0`, `v0.1.1`, `v1.0.0`).
- **Prohibited**: Irregular or shorthand forms such as `v.01`, `v0.1`, `release-0.1`, or `0.1.0`.

> [!IMPORTANT]
> GitHub Actions workflows rely on the exact tag format `vX.Y.Z` to construct file names (`gank-v0.1.0-windows-x86_64.zip`). Using irregular tag names will break package generation or create inconsistent download URLs.

---

## 4. Pre-Release Quality Gates

All checks must pass with zero errors and zero warnings before proceeding:

### 4.1 Automated Tests
```powershell
cargo test
```
*Requirement:* All unit tests must pass (`0 failed`).

### 4.2 Strict Clippy Lints
```powershell
cargo clippy --all-targets -- -D warnings
```
*Requirement:* Zero warnings allowed. All warnings are treated as errors.

### 4.3 Code Formatting
```powershell
cargo fmt --all -- --check
```
*Requirement:* Code must strictly match the Rustfmt standard.

---

## 5. Version Bump & Documentation Standards

When preparing a new release:

1. **`Cargo.toml`**:
   Update `version`:
   ```toml
   [package]
   version = "0.1.0"
   ```

2. **`CHANGELOG.md`**:
   Document changes grouped under standard [Keep a Changelog](https://keepachangelog.com/) sections:
   - `### Added`: New capabilities and features.
   - `### Performance`: Speed, CPU, and memory optimizations.
   - `### Changed`: Changes in existing functionality or UI improvements.
   - `### Fixed`: Bug fixes and stability improvements.

3. **`ROADMAP.md`**:
   Mark completed milestone deliverables as `[x]`.

---

## 6. Local Packaging Verification (Smoke Test)

Before tagging, verify that build scripts produce valid artifacts:

### 6.1 Portable Bundle (.zip)
```powershell
powershell -ExecutionPolicy Bypass -File ./scripts/package.ps1
```
*Output:* `dist/gank-vX.Y.Z-windows-x86_64.zip` and `.sha256`.

### 6.2 Manual Smoke Test Checklist
- [ ] Application launches cleanly from the unpacked portable directory.
- [ ] Official Gank icon appears on the executable and Windows Taskbar.
- [ ] Shaders and library cards render at 60 FPS with VSync.
- [ ] Memory footprint stabilizes at ~32 MB after initial startup.
- [ ] Game details modal opens and shows metadata.
- [ ] Game launch initiates the selected platform.

---

## 7. Official Publishing Workflow

Once `master` is updated and verified:

### 7.1 Create Annotated Git Tag
```bash
git tag -a v0.1.0 -m "Gank v0.1.0 — Genesis Release"
```

### 7.2 Push Tag to GitHub
```bash
git push origin v0.1.0
```

---

## 8. GitHub Actions Automated Pipeline & Asset Verification

Pushing the tag triggers `.github/workflows/release.yml`:

1. **Quality Gate (`validate`)**: Runs `cargo fmt`, `cargo clippy`, and `cargo test` on Windows.
2. **Windows Matrix (`release-windows`)**: Builds optimized release binaries with LTO, bundles resources, compresses the portable `.zip`, and uploads release assets.
3. **Release Verification**:
   Navigate to `https://github.com/carllosnc/gank/releases/tag/vX.Y.Z` and verify assets:
   - `gank-vX.Y.Z-windows-x86_64.zip` + `.sha256`
4. **Publish**: The release notes will be automatically populated, or you can paste the relevant entry from `CHANGELOG.md`.
