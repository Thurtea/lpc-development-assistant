# Changelog

All notable changes to LPC Development Assistant are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.2.0] - 2026-01-26

### Added
- **Generated Code Tab**: New dedicated interface for managing driver and library code output
- **Save to Driver**: Direct file save functionality to WSL driver directory with atomic writes
- **Save to Library**: Direct file save functionality to WSL library directory
- **Copy-to-Output**: Quick transfer buttons to copy AI responses directly to driver or library output areas
- **Configuration UI**: Visual interface for managing WSL output directory paths
- **Status Messaging**: Real-time feedback with color-coded states (success/error/info)
- **Protected Files**: Safety checks preventing accidental overwrites of critical files (master.c, simul_efun.c, etc.)
- **Tab Navigation**: Seamless switching between Assistant, Driver, Generated Code, and Settings tabs

### Fixed
- **Clippy Warnings**: Resolved all clippy lint warnings across binary targets (8 individual fixes)
  - Fixed 4 redundant closures in error handling (`|e| io::Error::other(e)` → `io::Error::other`)
  - Removed unused imports in 5 files (benchmark.rs, test_get_template.rs, driver modules)
  - Fixed needless borrows in Command::args calls
  - Updated complex type annotations in benchmark.rs
- **Deprecated EGUI APIs**: Updated to use `ComboBox::new()` and `id_salt` instead of deprecated methods
- **Code Quality**: Removed unnecessary string allocations and cleaned up error handling

### Changed
- Enhanced file operation error messages with more diagnostic information
- Improved configuration persistence with serde defaults
- Better WSL path mapping validation and error reporting

### Build Info
- **Release Date**: January 26, 2026
- **MSI Installer**: LPC Dev Assistant_1.2.0_x64_en-US.msi (20.3 MB)
- **SHA256**: 2A9E90BEE4ED03EB584557B6E6179652B2C5480B4D614F1090862E5BD4344AE1
- **Build Time**: ~5 minutes (Rust 1.92.0, Tauri 2.x)
- **Quality Metrics**:
  - Clippy Warnings: 0
  - Compiler Errors: 0
  - Library Target: ✅ CLEAN
  - Binary Targets: ✅ PASSING

---

## [1.1.0] - Previous Release

See [releases page](https://github.com/Thurtea/lpc-development-assistant/releases) for earlier versions.
