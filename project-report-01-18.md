# Project Report: LPC Development Assistant
Date: 2026-01-18

## Executive Summary
- Goal: Provide a desktop assistant (Rust + Tauri) that queries a local Ollama LLM and supplies contextualized guidance for LPC MUD driver and library development using the `mud-references/` corpus.
- Current state: Core Rust library exists (ollama_client, context_manager, prompt_builder, mud_index, driver_analyzer). The Tauri bridge and streaming plumbing are implemented; frontend assets and a stable RAG ingestion pipeline are still missing.
- Completion estimate: ~50% (core interactions implemented; major missing pieces: UI assets, RAG pipeline, CI/tests).
- Critical blockers: absence of frontend build assets / tauri.conf.json in repo for packaging; lack of embeddings/vector store and chunking for reliable RAG; no CI or automated tests.

## Architecture Overview
System components (ASCII diagram):

User (Browser / Tauri Webview)
  |
  | <-- invoke / send question -->
Tauri binary (src-tauri/src/main.rs)
  ├─ manages AppState (PromptBuilder, MudReferenceIndex)
  ├─ exposes commands: ask_ollama, ask_ollama_stream, list_models, check_ollama_health, extract_references, etc.
  └─ emits events to UI: ollama-token, ollama-error, ollama-done
      |
      +--> lpc_dev_assistant library (src/)
            ├─ ollama_client.rs  (HTTP client + streaming)
            ├─ context_manager.rs (templates, archive extraction)
            ├─ prompt_builder.rs (compose prompts & truncate)
            ├─ mud_index.rs (fallback index; planned Tantivy/Vector DB)
            └─ driver_analyzer.rs (efun discovery)

Data flow summary: User -> Frontend (asks) -> Tauri -> PromptBuilder + Index -> Ollama (HTTP API) -> streamed tokens -> Frontend

Technology stack:
- Rust (tokio, reqwest, anyhow, serde)
- Tauri for desktop shell (optional feature)
- Optional native GUI toolkits: egui/eframe, iced
- Local Ollama HTTP API for LLM inference
- mud-references as corpus archives (ZIP/TGZ)

## Component Status Matrix
| Component | Location | Status | Quality | Notes |
|-----------|----------|--------|---------|-------|
| Rust Backend | src/ | Mostly complete | Good; uses anyhow/context; some unwraps remain | Exports OllamaClient and context utilities
| Tauri Integration | src-tauri/ | Mostly complete | Adequate; converts errors to String at boundary | Commands & streaming implemented; tauri.conf.json/assets missing
| Vite Frontend | UI/ | Missing / external | N/A | UI assets not present in repository root (frontend wiring incomplete)
| Ollama Client | src/ollama_client.rs | Complete | Good (stream + sync); small unwrap at client build | Supports generate, list_models, generate_stream
| LPC References | mud-references/ | Present (archives & source trees) | N/A (corpus data) | ContextManager extracts archives; MudReferenceIndex is a fallback file-scan



## Dependencies
### Rust (Cargo.toml)
Production dependencies (from Cargo.toml):
- tauri = 2.1 (optional) — Tauri desktop/webview integration
- serde = 1.0 (derive) — serialization/deserialization
- serde_json = 1.0 — JSON handling
- tokio = 1.42 (full) — async runtime
- futures = 0.3 — async utilities
- syntect = 5.2 — syntax highlighting
- eframe = 0.29, egui = 0.29, egui_extras = 0.29 — optional/native GUI stack
- iced = 0.9 (optional) — alternative GUI
- tokio-stream = 0.1 — stream wrappers
- reqwest = 0.12 (json,stream,blocking) — HTTP client (Ollama)
- anyhow = 1.0 — error handling
- async-stream = 0.3 — async stream helpers
- walkdir = 2.5 — directory traversal for corpus
- tantivy = 0.22 (lz4-compression) — planned full-text index engine
- zip = 2.2, tar = 0.4, flate2 = 1.0 — archive extraction
- chrono = 0.4 — date/time utilities
- regex = 1.9 — pattern matching

Dev/build dependencies:
- tauri-build = 2.1 — build-time Tauri asset compilation

Feature flags:
- with_tauri — enables `tauri` dependency and the Tauri binary
- custom-protocol — enables `tauri/custom-protocol`
- iced-gui — enables `iced` GUI binary

### Frontend (package.json)
- No UI package.json found in repository root (UI/ missing). Frontend dependencies could not be listed. Please provide UI/package.json or add UI assets to repo.

## File Inventory
- Total Rust files: ~25 source files (src/ + src/bin/), several example binaries under src/bin/ (see list in repository). Exact LOC: placeholder (run `cloc` or `wc -l` locally to compute).
- Configuration files present: Cargo.toml, build.rs, .gitignore, README.md, ROADMAP.md, project-report-01-18.md. `src-tauri/tauri.conf.json` and UI/package.json appear missing from repo.
- Documentation files present: README.md, summary-01-18.md, TAURI_FIX_INSTRUCTIONS.md, ROADMAP.md, various docs in src/bin/ and mud-references/.
- Tests: no #[cfg(test)] unit test modules discovered in src/. Several test/demo binaries exist (test_cli, run_stream_test, ollama_health) but no automated test suite or GitHub Actions workflows.

## Integration Status
- Ollama connectivity: partially tested (manual test binaries exist: ollama_health, test_cli). No automated integration tests in CI.
- Frontend-backend communication: implemented in Tauri layer (ask_ollama_stream, events), but frontend assets not bundled — end-to-end GUI unverified in repo.
- LPC reference ingestion: extraction and simple search implemented; no semantic/RAG ingestion yet.
- Training pipeline: not implemented (no embeddings, no vector store, no chunking).

## Quality Assessment
- Error handling maturity: 3/5 (uses anyhow and context across library; some unwraps remain and errors are stringified at Tauri boundary).
- Test coverage: 0-5% (no unit tests found; manual test binaries exist). Placeholder: run `cargo test` and add unit tests.
- Documentation quality: 3/5 (README and docs present; more detailed developer docs and API docs would help).
- Code organization: 4/5 (modules are well-separated: ollama_client, context_manager, prompt_builder, mud_index, driver_analyzer).

## Identified Issues
1. Missing frontend assets (UI/ and src-tauri/tauri.conf.json) in the repository prevents end-to-end GUI validation and packaging.
2. No RAG ingestion pipeline: no chunking, embeddings, or vector store — retrieval is lexical fallback only.
3. No CI or automated tests; only manual/demo binaries exist.
4. Some unwraps/expect calls in code and demo binaries reduce robustness (e.g., reqwest client build unwrap in src/ollama_client.rs).
5. Errors are converted to Strings at Tauri boundary, losing structured context for richer UI diagnostics.

## Recommendations (priority-ordered)
1. Add CI with tests and cargo-audit (High) — implement unit tests for OllamaClient (HTTP mock), PromptBuilder, ContextManager; add a GitHub Actions workflow.
2. Include or reference UI assets and tauri.conf.json (High) — ensure Tauri packaging can be run and frontend listeners for events are present.
3. Implement RAG ingestion pipeline (Medium) — chunking, embeddings service (start with local Python), vector store (Qdrant or HNSW), Retriever integration into MudReferenceIndex.
4. Harden error handling (Medium) — replace unwraps, propagate typed errors to UI boundary, and add telemetry/logging.
5. Implement stream cancellation and timeout controls (Medium) — allow UI to stop active requests.
6. Consolidate GUI dependencies and feature flags to reduce compile-time and binary size (Low).

## Next Steps
- [ ] Immediate actions
  - [ ] Add GitHub Actions CI skeleton to run cargo fmt/clippy/tests and cargo-audit.
  - [ ] Add a smoke integration test or test binary that verifies Ollama connectivity when OLLAMA_URL is set.
- [ ] Short-term (1-2 weeks)
  - [ ] Add frontend assets or document where UI is stored; add tauri.conf.json.
  - [ ] Implement simple chunker + Tantivy hybrid index for lexical retrieval.
- [ ] Medium-term (1 month)
  - [ ] Implement embeddings + vector store (Qdrant/HNSW) and integrate RAG into the ask_ollama flow.
  - [ ] Add structured logging and improve error propagation.
