# LPC Dev Assistant v1.2.0 Release Notes

**Release Date:** January 22, 2026

## What's New

### Ollama Auto-Start Feature
- App now prompts to start Ollama automatically on Windows if not running
- Opens new terminal with `ollama serve` command
- 3-second initialization wait after start
- Graceful fallback if auto-start fails

### Multi-Platform Build Support
- Windows: MSI and NSIS installers
- Linux: DEB and AppImage support (build configuration ready)
- macOS: DMG and APP bundle support (build configuration ready)

### Documentation Organization
- Session documentation moved to `session-notes/` folder
- Cleaner repository structure
- Added comprehensive release process documentation

## Installation

### Windows
**Download one of the installers:**
- `LPC_Dev_Assistant_v1.2.0_x64_en-US.msi` (recommended, 19.7 MB)
- `LPC_Dev_Assistant_v1.2.0_x64-setup.exe` (NSIS, 10.4 MB)

**Run the installer and follow prompts.**

### macOS & Linux
Build from source (native builds coming in future releases):
```bash
git clone https://github.com/Thurtea/lpc-development-assistant.git
cd lpc-development-assistant
git checkout v1.2.0
cargo tauri build
```

## Requirements

- **Windows:** 10 or later (64-bit)
- **Ollama:** Required for LLM functionality ([ollama.ai](https://ollama.ai))
- **Recommended Model:** `qwen2.5-coder:7b` or similar

## Features

- Desktop GUI with real-time streaming responses
- RAG (Retrieval-Augmented Generation) with MUD reference corpus
- Top-15 code search with relevance scoring
- UI validation panel showing code quality metrics
- Score widget (0-10 rating) for generated code
- Copy buttons for prompts and code blocks
- Adjustable generation parameters (temperature, top-p, top-k)
- First-run setup with automatic corpus indexing

## Full Changelog

### Added
- Ollama auto-start command for Windows (`start_ollama`)
- UI prompt for starting Ollama if not detected
- Linux build targets (deb, AppImage) in `tauri.conf.json`
- macOS build targets (dmg, app) in `tauri.conf.json`
- `build-all-platforms.ps1` script for multi-platform builds
- `RELEASE_PROCESS.md` documentation for maintainers
- Cross-compilation guide in README

### Changed
- Moved 19 documentation files to `session-notes/` folder
- Simplified `.gitignore` (13 patterns → 1 folder pattern)
- Updated README with GitHub Releases installation instructions

### Fixed
- Clarified repository structure (target/ folder intentionally excluded)

## Known Issues

- macOS and Linux native builds not yet available (configuration ready, requires native build systems)
- Some non-critical compiler warnings remain (unused code, dead code analysis)

## Documentation

- [README.md](https://github.com/Thurtea/lpc-development-assistant/blob/main/README.md) - Getting started
- [RELEASE_PROCESS.md](https://github.com/Thurtea/lpc-development-assistant/blob/main/RELEASE_PROCESS.md) - For maintainers

## Links

- **Repository:** https://github.com/Thurtea/lpc-development-assistant
- **Issues:** https://github.com/Thurtea/lpc-development-assistant/issues
- **Releases:** https://github.com/Thurtea/lpc-development-assistant/releases

## Feedback

Please report bugs or feature requests via [GitHub Issues](https://github.com/Thurtea/lpc-development-assistant/issues).

---

**Full Commit History:**
```
a3d310a9 - docs: clarify installation from releases and add release process guide
3fedc1da - feat: add Linux and macOS build targets
8c7a8b64 - feat: add ollama auto-start feature and reorganize docs
6aeb1b10 - docs: add phase4 verification and summary
f62710af - Phase 4 complete: object_system template, UI validation, 85%+ benchmark
```
