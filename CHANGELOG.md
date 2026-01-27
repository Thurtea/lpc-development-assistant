# Changelog

All notable changes to LPC Development Assistant are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.3.0] - 2026-01-27

### Added
- **Ollama Setup Wizard**: Complete configuration interface for Ollama installation, models, and status
  - Check Ollama installation and running status
  - Start Ollama server directly from UI
  - Pull models from dropdown selection (qwen2.5-coder:7b, glm-4.7-flash, glm-4.7:cloud)
  - View installed models list
  - Save Ollama configuration to persistent storage
- **Staging Workflow**: Safe code review before deployment to AMLP projects
  - New Staging tab for reviewing generated code
  - Save generated code to staging directory first
  - Preview file contents (200 chars) with full view option
  - Copy individual files to driver or library after review
  - Clear staging directory with confirmation
  - Auto-initialize staging at ~/.lpc-dev-assistant/staging/
- **Project Path Configuration**: Centralized management of AMLP paths
  - Configure AMLP Driver Root path (WSL)
  - Configure AMLP Library Root path (WSL)
  - Test path connectivity before saving
  - Display staging directory location (read-only)
- **Automatic File Backups**: Timestamp-based backup before overwriting
  - Format: filename.c.bak-YYYYMMDD-HHMMSS
  - Preserves original files when copying from staging
- **Enhanced Config Structure**: Extended DriverConfig with new fields
  - ollama_model: Selected model name
  - ollama_provider: "local" or "cloud"
  - ollama_enabled_tools: Future expansion for tool integration
  - staging_directory: Path to staging location
  - setup_complete: Track wizard completion status

### Changed
- **Code Generation Flow**: Generated code now saves to staging first instead of direct deployment
- **Copy Buttons**: "To Driver" and "To Library" buttons now route through staging workflow
- **Tab Navigation**: Added Staging tab between Generated Code and Settings
- **Settings Tab**: Reorganized with new Ollama Setup and Project Paths sections

### Fixed
- Removed all non-ASCII characters (emojis, special arrows, bullets) from UI
- Replaced Unicode arrows with "To Driver"/"To Library" text
- Converted ellipsis (...) to ASCII equivalent
- Replaced bullet characters with hyphens

### Technical
- New Rust module: src-tauri/src/commands/staging.rs
- New Tauri commands: check_ollama_installed, check_ollama_running, start_ollama_server, pull_ollama_model, list_ollama_models, setup_staging_directory
- New staging commands: save_to_staging, list_staged_files, copy_staged_to_project, clear_staging
- WSL integration for safe file copying with parent directory creation

---

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
