# ROADMAP: LPC Development Assistant

Goal: Rust client -> connects to local Ollama -> trained on mud-references/ (MUD drivers and libraries)

Current state (summary)
- Core Rust library implemented (Ollama client, ContextManager, PromptBuilder, fallback MudReferenceIndex, driver analyzer).
- Ollama integration is implemented (generate, list_models, streaming) and wired to Tauri commands.
- mud-references extraction & simple file-scan retrieval exist, but no embeddings/vector store or chunking pipeline.
- UI assets and tauri.conf.json are not bundled in this repo; multiple GUI backends are included but feature-gated.
- No automated CI/tests for integration or ingestion pipeline.

Phases

## Phase 1: Foundation (Current)
- [ ] Core Rust client architecture — S (already implemented)
  - Prereq: None
- [ ] Ollama API integration — S (implemented)
  - Prereq: Local Ollama or compatible endpoint
- [ ] Basic UI for queries — M (partial; UI assets missing/need wiring)
  - Prereq: Vite UI assets or Tauri web assets
- [ ] Configuration management (OLLAMA_URL, timeouts) — S (implemented via env vars)
  - Prereq: Document env var usage in README

## Phase 2: LPC Reference Integration
- [ ] mud-references/ extraction scripts — S (ContextManager.extract_archives implemented)
  - Effort: S
  - Prereq: Ensure mud archives present
- [ ] LPC/C code parser (structured parsing beyond regex) — M
  - Effort: M
  - Prereq: Evaluate use of tree-sitter or custom parser libraries
- [ ] Document chunking and preprocessing (HTML->text, code separation) — M
  - Effort: M
  - Prereq: Tokenization strategy (tokenizers crate) and chunk size decision
- [ ] RAG vector store setup (Qdrant / HNSW / Tantivy hybrid) — M-L
  - Effort: M-L (depending on choice and infra)
  - Prereq: Embedding service (local or cloud) and vector DB choice

## Phase 3: Training/Context System
- [ ] Ollama model selection and configuration — S-M
  - Effort: S-M
  - Prereq: Available local models and testing harness
- [ ] LPC context injection strategy (prompt templates, citation format) — M
  - Effort: M
  - Prereq: Retrieval results and prompt budget decisions
- [ ] Prompt engineering for MUD/LPC queries (templates, system prompt) — M
  - Effort: M
  - Prereq: Test queries and feedback loop
- [ ] Testing with sample LPC code questions (integration tests + evaluation) — M
  - Effort: M
  - Prereq: Test corpus and scoring/rubric

## Phase 4: Polish and UX
- [ ] Improved UI for code browsing and snippets — M
  - Effort: M
  - Prereq: Frontend scaffolding, index APIs from backend
- [ ] Syntax highlighting for LPC (syntect integration or web-based) — M
  - Effort: M
  - Prereq: Template rendering and frontend support
- [ ] Example queries and tutorials — S-M
  - Effort: S-M
  - Prereq: Documentation and example corpus
- [ ] Documentation and developer guides — S
  - Effort: S
  - Prereq: project-report and README updates

Milestones
- M1: Basic RAG pipeline (chunker + embeddings service + vector store + retriever) — 2-3 weeks
- M2: Integrated prompt injection + streamable UI — 1 week
- M3: CI, tests, and documentation — 1 week

Notes
- Prefer RAG over immediate fine-tuning; fine-tuning to be considered after stable RAG performance or if local model training capability exists.
- Start with an external/local embedding service (Python sentence-transformers) to iterate fast, then consider Rust-native embedding once stable.

If you want, create issues/PR stubs for top 3 Phase 2 tasks and I will scaffold the code and CI tasks.