# LPC Development Assistant

![Rust](https://img.shields.io/badge/rust-1.70%2B-000000.svg) ![License](https://img.shields.io/badge/license-MIT-blue)

A cross-platform desktop application for LPC development, powered by Rust and Ollama. Get contextualized code suggestions and documentation for LPC/MUD development with an integrated AI assistant.

## Features

- Desktop GUI (Windows, macOS, Linux) with Tauri
- Real-time streaming responses from local Ollama models
- Markdown rendering with syntax highlighting
- Adjustable generation parameters (temperature, top-p, top-k, prediction length)
- Built-in LPC reference corpus and search
- First-run setup with automatic Ollama detection

## Prerequisites

### Required
- **Ollama** running locally (default: http://localhost:11434)
- **qwen2.5-coder:3b** model installed
- Rust 1.70+

### Optional
- For building from source: Cargo and standard build tools

## Installation

### Quick Install (Pre-Built)
**Download from [GitHub Releases](https://github.com/Thurtea/lpc-development-assistant/releases):**

#### Windows
Download `LPC Dev Assistant_0.1.0_x64_en-US.msi` and run the installer.

#### macOS
Download `.dmg` file and drag the app to Applications.

Supports both Intel (x86_64) and Apple Silicon (aarch64).

#### Linux
Download `.AppImage` or `.deb` file from releases.

**AppImage:** Make executable and run
```bash
chmod +x LPC_Dev_Assistant_*.AppImage
./LPC_Dev_Assistant_*.AppImage
```

**Debian/Ubuntu:**
```bash
sudo apt install ./lpc-dev-assistant_*.deb
lpc-dev-assistant
```

### Build from Source (Optional)
If you prefer to build locally or releases aren't available yet:
```bash
git clone https://github.com/Thurtea/lpc-development-assistant.git
cd lpc-development-assistant
cargo tauri build
```
Installers will be in `target/release/bundle/`

---

**Note:** The `target/` folder is not in the repository (standard Rust practice). Pre-built installers are distributed via [GitHub Releases](https://github.com/Thurtea/lpc-development-assistant/releases).

## Building from Source

### 1. Clone and enter the repository
```bash
git clone https://github.com/Thurtea/lpc-development-assistant.git
cd lpc-development-assistant
```

### 2. Install Ollama
Visit [ollama.ai](https://ollama.ai) and install for your platform. Start the daemon:
```bash
ollama serve
```

### 3. Pull the required model
The application is optimized for **qwen2.5-coder:3b** based on extensive testing:
```bash
ollama pull qwen2.5-coder:3b
```

This model provides:
- **85% accuracy** on LPC queries
- Fast inference (40.9 tokens/sec)
- Best overall balance for LPC development

### 4. Install Rust toolchain
If you haven't already:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 5. Add build targets (for cross-compilation)

**For Windows builds:**
```bash
rustup target add x86_64-pc-windows-msvc
```

**For Linux builds:**
```bash
rustup target add x86_64-unknown-linux-gnu
# Or use cross-compilation tool:
cargo install cross
```

**For macOS builds (on macOS only):**
```bash
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin
```

### 6. Build the app

**Single platform:**
```bash
# Windows
cargo tauri build

# macOS
cargo tauri build

# Linux
cargo tauri build
```

**All platforms (from Windows with cross-tools):**
```powershell
# Using provided build script (requires cross tools installed)
.\build-all-platforms.ps1 -Platform all
```

**Development mode:**
```bash
cargo tauri dev
```

### Cross-Compilation Notes

**Building for Linux on Windows:**
- Easiest: Use Windows Subsystem for Linux (WSL2) with Linux environment
- Alternative: Install `cross` tool: `cargo install cross`
- Then: `cross tauri build --target x86_64-unknown-linux-gnu`

**Building for macOS:**
- Best option: Build natively on macOS
- macOS to Linux is possible with cross-compilation tools
- Windows to macOS cross-compilation is complex and not recommended

**Linux to Windows:**
- Use `mingw-w64` toolchain
- Install target: `rustup target add x86_64-pc-windows-gnu`
- Build: `cargo tauri build --target x86_64-pc-windows-gnu`

See [Rust Platform Support](https://doc.rust-lang.org/nightly/rustc/platform-support.html) for detailed cross-compilation guidance.

## Model Recommendation

This application is **locked to use qwen2.5-coder:3b exclusively**. This model was selected after comprehensive testing:

| Model | Accuracy | Quality | Speed | Recommendation |
|-------|----------|---------|-------|----------------|
| **qwen2.5-coder:3b** | **85.0%** | 70.3% | 16.6s | ⭐ **Best Choice** |
| qwen2.5-coder:7b | 80.0% | 71.4% | 20.2s | Good but slower |
| qwen2.5-coder:1.5b | 55.0% | 68.7% | 15.1s | Too small |
| llama2:latest | 46.7% | 67.6% | 18.0s | Poor for code |

**Why qwen2.5-coder:3b?**
- Highest accuracy (85%) with LPC-specific queries
- Excellent code generation quality
- Fast inference suitable for interactive development
- Optimal balance of accuracy, quality, and speed

See `model_comparison_results.json` for detailed test results.

## Configuration

Environment variables (optional):
- `OLLAMA_URL`: Ollama server endpoint (default: http://localhost:11434)
- `OLLAMA_TIMEOUT_SECS`: Request timeout in seconds (default: 30)

## Project Structure

- `src/` - Rust library with core logic (ollama_client, prompt_builder, mud_index, context_manager)
- `src-tauri/` - Tauri app integration
- `ui/` - Web frontend (HTML, CSS, JavaScript)
- `icons/` - App icons for all platforms
- `lpc-dev-assistant/templates/` - LPC context templates
- `mud-references/` - Reference documentation archives

## Architecture

```
User Interface (Browser/Tauri Webview)
        |
        | (invoke commands)
        v
Tauri Backend (src-tauri/main.rs)
        |
        +-- Ollama Client (HTTP)
        +-- Prompt Builder (templates + examples)
        +-- Mud Reference Index (full-text search)
        +-- Context Manager (archive extraction)
        |
        v
Local Ollama LLM
```

## License

MIT - see LICENSE file for details