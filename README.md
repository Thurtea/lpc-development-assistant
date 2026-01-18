# lpc-development-assistant

Lightweight tooling to help develop and analyze MUD drivers and mudlibs.

Getting started

- Open this repository in GitHub Codespaces or VS Code.
- Run the dev UI (if using the GUI) with the provided `start-dev.ps1` or start the Rust backend:

```powershell
# frontend (vite)
npx vite --port 1420 --host 0.0.0.0

# backend (Rust)
cargo run
```

Repository layout

- `src/` - Rust library and binaries
- `src-tauri/` - Tauri integration (optional, feature-gated)
- `ui/` - Vite frontend
- `mud-references/` - optional downloaded reference archives (ignored via .gitignore)

Ollama / LLM setup
-------------------

The project expects an Ollama-compatible service by default at `http://localhost:11434`.

- To override the URL set the `OLLAMA_URL` environment variable.
- To adjust request timeout (seconds) set `OLLAMA_TIMEOUT_SECS`.

You can verify connectivity with the included health binary:

```powershell
cd lpc-dev-assistant
cargo run --bin ollama_health
```

If you don't have Ollama, point `OLLAMA_URL` to a compatible API endpoint.
Contributing

Please open issues or pull requests. If you want me to add collaborators or push to your GitHub remote, provide the repository URL and a token or ensure `git`/`gh` are authenticated in this environment.
