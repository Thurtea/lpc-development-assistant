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
- Light/dark theme toggle

## Prerequisites

### Required
- Ollama running locally (default: http://localhost:11434)
- Rust 1.70+

### Optional
- For building from source: Cargo and standard build tools

## Installation

### Windows
Download the installer from the [latest release](https://github.com/Thurtea/lpc-development-assistant/releases) and run it. The installer will:
1. Detect if Ollama is installed
2. Extract reference documentation
3. Build the search index
4. Launch the app

### macOS and Linux
Coming soon. For now, build from source.

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

### 3. Pull a model
```bash
ollama pull qwen2.5-coder:7b
```

### 4. Build the app
```bash
# Windows
cargo tauri build --bundles nsis

# macOS
cargo tauri build --bundles dmg

# Linux
cargo tauri build --bundles appimage
```

Or run in development mode:
```bash
cargo tauri dev
```

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