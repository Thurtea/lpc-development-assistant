# Iced Native GUI - Quick Start

## Overview

This is the **pure Rust native GUI** using the `iced` framework. It provides the same functionality as the Tauri GUI but without requiring web technologies.

## Prerequisites

- Rust 1.70+ installed
- Ollama running locally (`ollama serve`)
- Models downloaded (`ollama pull qwen2.5-coder:7b`)

## Running the GUI

```powershell
# From project root
cd E:\Work\AMLP\lpc-dev-assistant

# Run in development mode
cargo run --bin gui

# Or build release version
cargo build --release --bin gui
.\target\release\gui.exe
```

## Features

✅ **Model Selection** - Choose from available Ollama models  
✅ **Context Loading** - Automatically loads MUD development context  
✅ **Code Generation** - Ask questions, get C/LPC code  
✅ **Syntax Highlighting** - Basic highlighting (ANSI colors)  
✅ **File Saving** - Save responses to `gen/` directory  
✅ **Reference Search** - Search extracted MUD sources  

## Usage

### 1. Select Model

Choose `qwen2.5-coder:7b` for code generation or `qwen2.5:7b-instruct` for explanations.

### 2. Select Context

- **Driver Development** - C driver code (lexer, parser, VM, efuns)
- **Efuns Implementation** - Built-in function implementation
- **MudLib/LPC Code** - LPC mudlib features (rooms, NPCs, combat)
- **Reference Libraries** - Guide to extracted MUD sources

### 3. Ask Question

Examples:
```
Write the complete LPC lexer in C with all keywords

Implement the object manager with reference counting

Create a combat system based on Nightmare 4
```

### 4. Save Response

Click **💾 Save** to save the response to `gen/[context]_[timestamp].c`

## Architecture

```
src/bin/gui.rs
├── Application trait implementation (iced)
├── Message enum (UI events)
├── async helpers
│   ├── load_ollama_models()
│   ├── generate_response()
│   ├── call_ollama_http()
│   └── call_ollama_cli() (fallback)
└── view rendering
```

## Differences from Tauri GUI

| Feature | Iced GUI | Tauri GUI |
|---------|----------|-----------|
| **Technology** | Pure Rust | HTML/CSS/JS |
| **Binary Size** | ~15MB | ~8MB |
| **Startup Time** | ~1s | ~2s |
| **Streaming** | No (async batch) | Yes (possible) |
| **Styling** | Rust code | CSS |
| **Development** | Rust expertise | Web expertise |

## Troubleshooting

### "Failed to connect to Ollama"

**Solution:**
```powershell
# Start Ollama server
ollama serve

# Verify models are available
ollama list
```

### "Failed to load context template"

**Solution:**
```powershell
# Check templates exist
dir E:\Work\AMLP\lpc-dev-assistant\templates

# Should see:
# - driver_context.txt
# - efuns_context.txt
# - mudlib_context.txt
# - reference_sources.txt
```

### Compilation Errors

**Solution:**
```powershell
# Update Rust
rustup update

# Clean build
cargo clean
cargo build --bin gui
```

### Syntax Highlighting Not Working

The current implementation uses ANSI terminal escape codes which may not render perfectly in the GUI. This is a known limitation.

**Workaround:** Use the Tauri GUI for better syntax highlighting, or save the file and view in VS Code.

## Known Limitations

1. ❌ **No streaming** - Responses arrive as complete blocks
2. ❌ **Basic highlighting** - Uses ANSI escapes (terminal-style)
3. ❌ **No chat history** - Each query is independent
4. ⚠️ **Windows-specific paths** - Hardcoded to `E:\Work\AMLP\`

## Future Improvements

- [ ] Implement proper streaming with progress indicator
- [ ] Native syntax highlighting (no ANSI escapes)
- [ ] Chat history / conversation persistence
- [ ] Cross-platform path handling
- [ ] Rich text rendering for code blocks
- [ ] Copy-to-clipboard button
- [ ] Model download progress indicator

## Comparison: When to Use Which GUI

### Use **Iced GUI** When:
- ✅ You prefer native Rust code
- ✅ You want faster startup
- ✅ You're comfortable with Rust

### Use **Tauri GUI** When:
- ✅ You want better syntax highlighting
- ✅ You prefer HTML/CSS styling
- ✅ You need streaming responses
- ✅ You want a more polished UI

**Recommendation for learning C:** Either GUI works fine! Pick the one you're more comfortable maintaining.

## Testing

```powershell
# Build and run
cargo run --bin gui

# Test checklist:
# □ Models load from ollama
# □ Context selection changes
# □ Prompt input works
# □ Ask button generates response
# □ Save button creates file in gen/
# □ Search references finds files
```

## Contributing

When modifying `gui.rs`:

1. Keep async functions separate from UI code
2. Use `Command::perform` for async operations
3. Handle all error cases gracefully
4. Test with and without Ollama running
5. Verify file saves work correctly

## Support

For issues, check:
1. This README
2. Main project documentation
3. Iced framework docs: https://docs.rs/iced/
4. Ollama docs: https://github.com/ollama/ollama

---

**Note:** This is a learning tool for C/MUD development. The GUI is secondary to your actual goal of learning C!
