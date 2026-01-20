# lpc-development-assistant

[![Rust CI](https://github.com/Thurtea/lpc-development-assistant/actions/workflows/rust-ci.yml/badge.svg)](https://github.com/Thurtea/lpc-development-assistant/actions/workflows/rust-ci.yml) ![Rust](https://img.shields.io/badge/rust-1.70%2B-000000.svg) ![License](https://img.shields.io/badge/license-MIT-blue)

Lightweight tooling to help develop and analyze MUD drivers and mudlibs, with a Rust core that can query a local Ollama LLM and provide contextualized LPC guidance.

Getting started

- Open this repository in GitHub Codespaces or VS Code.
- Run the dev UI (if using the GUI) with the provided `start-dev.ps1` or start the Rust backend.

```powershell
# frontend (Vite) — from UI/ or ui/ if present
npx vite --port 1420 --host 0.0.0.0

# backend (Rust)
cargo run --bin lpc-dev-assistant
```

Repository layout

- `src/` - Rust library and binaries (ollama_client, context_manager, prompt_builder, mud_index, driver_analyzer)
- `src-tauri/` - Tauri integration (optional, feature-gated via Cargo feature `with_tauri`)
- `UI/` (or `ui/`) - Vite frontend (webview assets for Tauri)
- `mud-references/` - optional downloaded reference archives (ignored via .gitignore)

Project status

A detailed status report is available: [project-report-01-18.md](./project-report-01-18.md)

Architecture

The application is structured as a Rust library and a (optionally) bundled Tauri binary that orchestrates Ollama calls and local LPC references.

ASCII diagram:

User (Browser/Tauri Webview)
  |
  | <-- invokes (Tauri invoke / HTTP) -->
Tauri binary (src-tauri) -- calls --> lpc_dev_assistant lib (src/)
  |                                         |
  |                                         +--> ContextManager (templates, mud-references)
  |                                         +--> PromptBuilder (assemble prompt + examples)
  |                                         +--> MudReferenceIndex (search / fallback)
  |
  +--> emits events (ollama-token / ollama-error / ollama-done) back to the webview
  |
  +--> Ollama (local) <-- HTTP POST /api/generate, GET /api/tags

Data flow: User -> Frontend -> Tauri -> PromptBuilder + Index -> Ollama -> Streamed tokens -> Frontend

Prerequisites

- Rust 1.70+ (edition 2021 used in Cargo.toml)
- Ollama (local daemon) or an Ollama-compatible HTTP endpoint (default: http://localhost:11434)
- Node.js 18+ and npm/yarn (for the Vite frontend if used)

Installation

1. Clone the repository and change into the project folder.
2. Build the Rust code:

```powershell
# build library and binaries
cargo build --release
```

3. Frontend (if UI folder present):

```powershell
cd UI
npm install
# for development
npm run dev
# build for production (used by Tauri packaging)
npm run build
```

4. Ollama models:
- Ensure Ollama daemon is running: `ollama serve` (default port 11434).
- Pull a model you intend to use, for example:

```powershell
# example: pull a model (replace model/name with a real Ollama model name)
ollama pull qwen2.5-coder:3b
```

Configuration (Environment variables)

- OLLAMA_URL — default: `http://localhost:11434`. Overrides Ollama base URL used by the client.
- OLLAMA_TIMEOUT_SECS — default: `30`. Request timeout in seconds used by reqwest client.

These can be exported in your shell or set in your system environment before running the binary.

Usage examples

- List available models (binary):

```powershell
cargo run --bin lpc-dev-assistant -- list-models
# OR using the included small health binary
cargo run --bin ollama_health
```

- Ask Ollama (via Tauri command / CLI test binaries):

```powershell
# Example using the tauri CLI binary (if built as lpc-dev-assistant)
cargo run --bin lpc-dev-assistant -- --model "qwen2.5-coder:3b" --question "Implement heart_beat efun in LPC" --context driver
```

- From the GUI: press "Ask Ollama" (the UI listens for 'ollama-token' events and streams output).

Development workflow

- Run the library/demo binaries for manual testing:
  - `cargo run --bin test_cli` — runs toy tests against Ollama and context manager.
  - `cargo run --bin run_stream_test` — tests streaming path.
- Build the Tauri app (feature-gated):

```powershell
# Build with Tauri features enabled
cargo build --features with_tauri --release
```

- Debugging tips:
  - Set `OLLAMA_URL` to a mock server or alternate endpoint for reproducible tests.
  - Enable verbose logging by running the binary directly and watching stderr (the app prints DEBUG prompt excerpts).

Testing

- There are several manual/test binaries under `src/bin/` for integration-like checks, but no automated unit test suite is present. Consider adding unit tests for ollama_client, prompt_builder, and context_manager.

Integration / Ollama details

- Ollama client lives in `src/ollama_client.rs`. It implements:
  - generate (sync/non-streaming)
  - list_models (GET /api/tags)
  - generate_stream (streamed generation, spawns a dedicated tokio runtime in a separate thread and forwards tokens via a channel)
- The Tauri layer (`src-tauri/src/main.rs`) wires commands: ask_ollama, ask_ollama_stream, list_models, check_ollama_health, extract_references, etc., and emits streaming events to the UI.

Troubleshooting

- "Cannot connect to Ollama": ensure `ollama serve` is running and `OLLAMA_URL` is correct; check port conflicts on 11434.
- Timeouts: increase `OLLAMA_TIMEOUT_SECS` environment variable.
- UI not loading in Tauri: ensure built frontend assets exist (UI/dist) and `tauri.conf.json` in `src-tauri/` points to them.

Project status & roadmap

- A more detailed status report is available: [project-report-01-18.md](./project-report-01-18.md)
- Short-term roadmap: add CI, unit/integration tests, stream cancellation support, and a RAG ingestion pipeline for mud-references.

Roadmap (high level)

1. Stabilize Ollama client and add integration tests.
2. Add CI and automated dependency/security checks.
3. Implement persistent index / RAG pipeline (Tantivy or vector DB + embeddings).
4. Polish UI and bundle with Tauri for releases.

Contributing

Please open issues or pull requests. For larger changes, open an issue first to discuss design and break down tasks.

License

MIT — see LICENSE file for details.

---

If anything is missing or you want this README adapted to a different packaging (e.g., pure native GUI vs Tauri), tell me which target and I will update it.