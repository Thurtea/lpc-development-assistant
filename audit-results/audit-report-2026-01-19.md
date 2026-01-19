# lpc-dev-assistant — Code Quality Audit (2026-01-19)

Summary
- Project path: `E:/Work/AMLP/lpc-dev-assistant`
- Commands executed (outputs saved in `audit-results/`):
  - `cargo clippy --all-targets -- -W clippy::unwrap_used -W clippy::expect_used` → [audit-results/clippy.txt](audit-results/clippy.txt)
  - `cargo test --workspace` → [audit-results/cargo-test.txt](audit-results/cargo-test.txt)
  - `audit-quality.ps1` run (generated additional artifacts: `unwraps_*.txt`, `todos_*.txt`, `env_*.txt`, `loc_*.txt`) — see `audit-results/`

High-level verdict
- The library (`src/`) builds, but workspace compilation fails due to errors in auxiliary binaries (see `audit-results/cargo-test.txt`). Clippy reports multiple `unwrap()` / `expect()` usages (potential panics) and several lints (unused imports/vars, deprecated APIs). Dependency freshness and vulnerability scans were not available on this environment because `cargo-outdated` and `cargo-audit` are not installed; corresponding placeholder outputs are in `audit-results/`.

LOC (Rust)
- `src/`: 2595 lines (see [audit-results/loc.txt](audit-results/loc.txt))
- `src-tauri/src/`: 217 lines

Table 1 — Current dependencies (from `Cargo.toml`)
- `tauri` = 2.1 (optional, feature `with_tauri`)
- `serde` = 1.0 (features: `derive`)
- `serde_json` = 1.0
- `tokio` = 1.42 (features: `full`)
- `futures` = 0.3
- `syntect` = 5.2
- `eframe` = 0.29
- `egui` = 0.29
- `egui_extras` = 0.29 (features: `syntect`)
- `iced` = 0.9 (optional)
- `tokio-stream` = 0.1
- `reqwest` = 0.12 (features: `json`, `stream`, `blocking`)
- `anyhow` = 1.0
- `async-stream` = 0.3
- `walkdir` = 2.5
- `tantivy` = 0.22 (features: `lz4-compression`)
- `zip` = 2.2
- `tar` = 0.4
- `flate2` = 1.0
- `chrono` = 0.4
- `regex` = 1.9

Table 2 — Freshness & security status (automation needed)
- Automated checks were not available on this runner:
  - `cargo-outdated` is not installed — see `audit-results/cargo-outdated.txt`.
  - `cargo-audit` is not installed — see `audit-results/cargo-audit-*.json`.

To produce a full current-vs-latest dependency table and a vulnerability report, run locally:

```powershell
cargo install cargo-outdated
cargo outdated --depth=1 > audit-results/cargo-outdated.txt


cargo audit --json > audit-results/cargo-audit.json
```

Table 3 — Notable dependency observations
- `reqwest` 0.12 with `blocking` feature increases binary surface area; prefer async API (`reqwest` async client) in async code paths unless blocking is necessary.
- `tokio` 1.42 is specified with `full`; ensure compatibility with `reqwest` and `tokio-stream` features.
- `tantivy` 0.22 appears used by `src/bin/build_tantivy_index.rs`, but build errors indicate API mismatches; either update code to the currently declared `tantivy` API or pin to a compatible version and add tests.
- UI crates (`eframe`/`egui`/`egui_extras`/`iced`) are recent; ensure UI binaries are gated by features to avoid pulling GUI deps for headless CI runs.

Test & build status
- `cargo test --workspace` failed during compilation: see `audit-results/cargo-test.txt` (key errors in `src/bin/efun_loader.rs` and `src/bin/build_tantivy_index.rs`).
- Because workspace compilation failed, no tests were executed.

Unwrap/expect scan (high-priority locations)
- The authoritative clippy run is in `audit-results/clippy.txt`; extracted high-priority `unwrap`/`expect` occurrences (see `audit-results/unwraps_*.txt`):
  - `src/ollama_client.rs:53` — `reqwest::Client::builder()...build().unwrap()` (library; avoid panics during client construction).
  - `src/ollama_client.rs:82` & `:166` — `response.text().await.unwrap_or_default()` (consider propagating errors or adding context rather than defaulting silently).
  - `src/bin/efun_loader.rs:12` — `v.as_str().unwrap()` (binary; can panic on unexpected input).
  - `src/bin/test_get_template.rs:6` — `cwd.parent().unwrap()` (test helper assumes parent exists).
  - `src/bin/egui_gui.rs:185` — `self.selected_model.unwrap()` (UI unwrap; prefer guarded access and graceful error display).

Findings — prioritized recommendations
- Critical (must-fix to get CI/tests passing):
  1. Fix `src/bin/efun_loader.rs` (`main` missing) and `src/bin/build_tantivy_index.rs` (tantivy API/import errors) so the workspace can compile. Optionally gate these binaries behind features to avoid CI breakage.
  2. Replace `build().unwrap()` in `src/ollama_client.rs` with fallible initialization that returns `Result` or logs a clear error; avoid silent panics.

- Important (quality & correctness):
  1. Remediate `unwrap`/`expect` usages in library-critical paths; convert to `?` or explicit error handling.
  2. Apply clippy automatic fixes where safe and address suggested lints (`unused_mut`, `single_char_add_str`, `implicit_saturating_sub`, etc.).

- Advisory (maintenance):
  1. Run `cargo outdated` and `cargo audit` on a machine with network access to fill Table 2 and identify vulnerable crates.
  2. Consider adding unit tests for `PromptBuilder`, `ContextManager`, and `MudReferenceIndex` logic.

Actionable commands (copy/paste)

```powershell
# Re-run local audits (installs only needed once):
cargo install cargo-outdated
cargo install cargo-audit

# Run checks and save outputs:
cargo clippy --all-targets -- -W clippy::unwrap_used -W clippy::expect_used 2>&1 | Tee-Object audit-results/clippy.txt
cargo test --workspace 2>&1 | Tee-Object audit-results/cargo-test.txt
cargo outdated --depth=1 > audit-results/cargo-outdated.txt
cargo audit --json > audit-results/cargo-audit.json

# If you want me to fix compile errors, tell me which to start with (efun_loader or tantivy indexer).
```

Where I saved outputs
- See `audit-results/` for all logs and helper files: `clippy.txt`, `cargo-test.txt`, `cargo-audit-*.json`, `cargo-outdated.txt`, `loc.txt`, `unwraps_*.txt`, `todos_*.txt`, `env_*.txt`.

Next steps I can take (pick one):
- Fix compile errors in `src/bin/efun_loader.rs` and/or `src/bin/build_tantivy_index.rs` so tests can run, or
- Install `cargo-outdated`/`cargo-audit` here and re-run to populate Table 2 with latest-version and vulnerability data, or
- Start remediating `unwrap`/`expect` occurrences in library code and high-risk binaries.

---
Report generated from local runs; attach or paste any log files you want deeper analysis on.
