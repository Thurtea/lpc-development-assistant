# LPC Dev Assistant - Full Project Analysis
**Date:** January 28, 2026  
**Status:** ⚠️ CRITICAL ISSUES - Ready for Development, NOT Production  
**Current Version:** 1.3.0 (git commit 997aac89)

---

## Executive Summary

The LPC Development Assistant has **critical blocking issues** that prevent it from functioning properly after installation:

1. ✅ **UI renders** - GitHub Dark theme applied, 3-tab layout works
2. ❌ **Setup doesn't execute** - First-run setup is silently failing in background
3. ❌ **Models not detected** - Ollama model detection fails or hangs silently
4. ✅ **Backend code exists** - All Tauri commands are properly exposed
5. ⚠️ **"Coming Soon" placeholders** - Optional Tools section has disabled checkboxes (minor issue)

### Root Causes
- **Async initialization errors caught silently** - All async init errors logged to console only, no user feedback
- **Race conditions** - Multiple concurrent async operations without proper error handling or sequencing
- **Missing error boundaries** - Backend calls fail without UI indication
- **Ollama dependency not validated** - App doesn't verify Ollama is running before attempting operations

---

## Project Structure & All Files Involved

### 📁 **Core Application Files**

#### **Frontend (UI)**
```
ui/index.html (1964 lines)
├── CSS: GitHub Dark theme with --bg, --panel, --text, --accent colors
├── HTML: 3 tabs (Assistant, Staging, Settings)
└── JavaScript: ~1000 lines of business logic
    ├── Event handlers for all buttons
    ├── Tauri IPC calls to backend
    ├── Model management & Ollama integration
    └── File operations & staging
```

**Key Issues in index.html:**
- Line 1481-1520: DOMContentLoaded sets "Ready" status immediately WITHOUT waiting for initialization
- Line 1485-1515: All async operations wrapped in background IIFE - errors only logged to console
- Line 884-904: loadModels() calls `get_available_models` but silently fails if Ollama not running
- Line 630-660: "Coming Soon" UI elements (Optional Tools checkboxes) - DISABLED but visible
- Line 1169-1200: checkOllamaStatus() function works but not properly called during init
- Line 1225-1245: pullSelectedModel() exists and works

#### **Tauri Configuration**
```
tauri.conf.json
├── productName: "LPC Dev Assistant"
├── version: 1.3.0
├── Window: 1400x900, centered
├── Bundle: Includes templates/ and mud-references/ resources
└── ✅ All correctly configured
```

#### **Tauri Main (Rust Backend)**
```
src-tauri/src/main.rs (850 lines)
├── AppState struct: workspace_root, prompt_builder, index, cancel_flag, first_run, path_mapper
├── Tauri Commands: 35+ exposed functions
├── Setup: ✅ Properly initializes all state
├── First Run: Uses Arc<AtomicBool> for tracking
└── Issues:
    ├── get_setup_status(): ❌ Checks for .setup_complete file but doesn't trigger setup
    ├── run_initial_setup(): ✅ Exists but NEVER CALLED from UI automatically
    ├── list_models() vs get_available_models(): Two different functions with overlap
    └── check_ollama_available(): ✅ Works but not called during init sequence
```

**Commands Exposed (from invoke_handler):**
- ask_ollama, ask_ollama_stream (streaming generation) 
- check_ollama_health, check_ollama_available, start_ollama
- list_models, get_available_models (DUPLICATE - see issue)
- get_setup_status, run_initial_setup, mark_setup_complete (setup flow)
- check_ollama_installed, check_ollama_running (status checks)
- start_ollama_server, pull_ollama_model, list_ollama_models (model management)
- list_contexts, search_references, extract_references (RAG)
- compile_lpc, run_lpc, build_driver_ui (LPC testing)
- save_to_driver, save_to_library (file output)
- save_to_staging, list_staged_files, copy_staged_to_project, clear_staging (staging)
- validate_and_diagnose_wsl, test_driver_connection (WSL/Driver validation)
- browse_wsl_directory, get_driver_config, save_driver_config_cmd

#### **Backend Modules**

**Ollama Client (src/ollama_client.rs)**
```
OllamaClient
├── new(): Creates HTTP client, timeout 30s, URL from OLLAMA_URL env var (default: localhost:11434)
├── list_models(): ✅ Fetches from /api/tags endpoint
├── generate(): Standard generation request
├── generate_stream_with_cancel(): Streaming with cancellation support
└── validate_generated_code(): Code validation helper
```

**Configuration (src-tauri/src/config.rs)**
```
DriverConfig
├── wsl_username, wsl_driver_root, wsl_library_root (WSL paths)
├── driver_output_dir, library_output_dir (output destinations)
├── ollama_model: "qwen2.5-coder:7b" (locked model)
├── staging_directory: Optional
└── setup_complete: Boolean flag
```
- Stores in: `%APPDATA%\lpc_dev_assistant_driver_config.json`
- Creates defaults on first run

**Context Manager (src/context_manager.rs)**
```
ContextManager
├── templates_path: Points to workspace/templates/
├── ensure_templates_exist(): ✅ Ensures template files present
├── extract_archives(): ✅ Extracts mud-references
├── search_code_examples(): Searches extracted MUD code
└── load_*_context(): Loads template content for RAG
```

**Prompt Builder (src/prompt_builder.rs)**
- Constructs prompts with context, examples, and questions
- Uses template files to build system prompts

**Mud Reference Index (src/mud_index.rs)**
```
MudReferenceIndex
├── build_index(): Indexes extracted MUD code with Tantivy
├── search_relevant_code(): Full-text search
├── search_with_scoring(): Scored search results
└── Status: ✅ Properly initialized
```

**File Operations (src-tauri/src/commands/file_operations.rs)**
```
save_to_driver(), save_to_library()
├── Path mapping (Windows → WSL)
├── WSL command execution
├── Atomic file writes with backups
├── Protected patterns: .backup, Makefile, README.md, master.c, etc.
└── Status: ✅ Fully implemented
```

**Staging (src-tauri/src/commands/staging.rs)**
```
save_to_staging(), list_staged_files(), copy_staged_to_project(), clear_staging()
├── Stores generated files in .lpc-dev-assistant/staging/
├── Lifecycle: pending → verified → copied
└── Status: ✅ Fully implemented
```

---

## Critical Issues Breakdown

### 🔴 Issue 1: Setup Doesn't Run After Install

**Symptoms:**
- App shows "Ready" immediately
- First-run setup is never triggered
- Index not built
- Archives not extracted

**Root Cause:**
```javascript
// ui/index.html, lines 1481-1520
document.addEventListener('DOMContentLoaded', async () => {
  // ...
  
  // Initialize async operations in background
  (async () => {
    try {
      await checkSetup();  // ← THIS IS WHERE SETUP SHOULD HAPPEN
    } catch (e) {
      console.warn('Setup check failed:', e);  // ← ERROR SILENTLY LOGGED
    }
    // ... other init functions wrapped in try-catch with console.warn
  })();
  
  // Set ready IMMEDIATELY - don't wait
  setStatus('Ready', 'success');  // ← HAPPENS BEFORE SETUP COMPLETES
});
```

**The `checkSetup()` function:**
```javascript
// Lines 1521-1580
async function checkSetup() {
  const status = await invoke('get_setup_status');  // ← Backend call
  
  if (status.first_run && !status.index_built) {
    const runSetup = confirm(
      'First Run Setup\n\n' +
      'The application needs to extract reference archives and build the search index.\n' +
      'This may take a few minutes.\n\n' +
      'Click OK to run setup now.'
    );
    
    if (runSetup) {
      setStatus('Running initial setup...', 'warning');
      try {
        const result = await invoke('run_initial_setup');  // ← BACKEND SETUP
        await invoke('mark_setup_complete');
        setStatus('Setup complete', 'success');
      } catch (e) {
        setStatus('Setup failed: ' + e, 'error');
        toast('Setup failed. Some features may not work.');
      }
    }
  }
}
```

**Why It Breaks:**
1. `get_setup_status()` backend call may fail (Ollama connection issue)
2. Error caught in line 1493 → `console.warn()` only → **No user notification**
3. Confirm dialog never appears because function threw
4. Status set to "Ready" at line 1520 while setup never happened
5. User thinks app is ready, but index is empty

**Backend Issue:**
```rust
// src-tauri/src/main.rs, lines 48-67
#[tauri::command]
fn get_setup_status(state: tauri::State<'_, AppState>) -> Result<serde_json::Value, String> {
    let ollama_ok = match tokio::runtime::Runtime::new() {
        Ok(rt) => {
            match OllamaClient::new() {
                Ok(client) => rt.block_on(async { client.list_models().await.is_ok() }),
                Err(_) => false,  // ← Returns false if Ollama not running
            }
        }
        Err(_) => false,
    };
    // Returns early error if ANY check fails
}
```

---

### 🔴 Issue 2: Models Won't Detect

**Symptoms:**
- Settings tab shows "Loading models..."
- Model list never populates
- Error: "Failed to connect to Ollama" or timeout

**Root Cause - Multiple Problems:**

**Problem A: Two conflicting model-listing functions**
```rust
// Function 1: list_models() - src-tauri/src/main.rs line 227
#[tauri::command]
fn list_models() -> Result<Vec<String>, String> {
    let client = OllamaClient::new()?;
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async { client.list_models().await })
}

// Function 2: list_ollama_models() - src-tauri/src/main.rs line 617
#[tauri::command]
async fn list_ollama_models() -> Result<Vec<String>, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()?;
    
    let response = client
        .get("http://localhost:11434/api/tags")
        .send()
        .await?;
    
    let json = response.json::<serde_json::Value>().await?;
    // ... manual parsing
}
```

**Problem B: JavaScript calls wrong function**
```javascript
// ui/index.html, line 889
// In loadModels() during init:
const models = await invoke('get_available_models');  // ← Uses this

// But in Settings tab, line 1200:
// In checkOllamaStatus():
await loadInstalledModels();  // ← This function uses:
async function loadInstalledModels() {
    const models = await invoke('list_ollama_models');  // ← Different function!
}
```

**Problem C: Silent failures in init sequence**
```javascript
// ui/index.html, lines 1496-1502
try {
  await loadModels();  // ← This calls get_available_models
} catch (e) {
  console.warn('Load models failed:', e);  // ← Only logged, not shown to user
}
```

**Problem D: Ollama must be running but isn't checked properly**
```rust
// OllamaClient::new() tries to create HTTP client but doesn't verify connectivity
pub fn new() -> Result<Self, anyhow::Error> {
    let base = std::env::var("OLLAMA_URL")
        .unwrap_or_else(|_| "http://localhost:11434".to_string());
    
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .context("Failed to create HTTP client")?;
    
    Ok(Self { base_url: base, client })  // ← No actual connectivity test!
}
```

**Result:** If Ollama isn't running:
1. IIFE catches error silently
2. Status set to "Ready" anyway
3. User thinks app is ready
4. First time they try to ask a question: "Error: Failed to connect to Ollama"

---

### 🟡 Issue 3: "Coming Soon" In The Program

**Location:** ui/index.html, lines 641-649

```html
<label style="display:flex; align-items:center; gap:8px; font-size:13px;">
  <input type="checkbox" id="tool-claude" disabled />
  <span style="color:var(--muted);">Claude Code (Coming Soon)</span>
</label>
<label style="display:flex; align-items:center; gap:8px; font-size:13px;">
  <input type="checkbox" id="tool-opencode" disabled />
  <span style="color:var(--muted);">OpenCode (Coming Soon)</span>
</label>
<label style="display:flex; align-items:center; gap:8px; font-size:13px;">
  <input type="checkbox" id="tool-codex" disabled />
  <span style="color:var(--muted);">Codex (Coming Soon)</span>
</label>
```

**Status:** ⚠️ **Minor visual issue**
- Checkboxes are disabled (no interaction possible)
- Just placeholder UI for future features
- Doesn't break functionality
- But looks unfinished to users

**Also:**  
Line 1374: `toast('View full file feature coming soon');` - Search results have a button that says "not ready"

---

## File Dependencies & Data Flow

### Initialization Sequence (What SHOULD Happen)
```
App starts (main.rs, AppState initialized)
    ↓
UI loads (index.html)
    ↓
DOMContentLoaded event fired
    ├─ loadSettings()
    ├─ bindEvents()
    │
    ├─ Background IIFE starts:
    │   ├─ checkSetup() ← Should trigger run_initial_setup()
    │   ├─ checkAndStartOllama()
    │   ├─ loadModels() ← Should populate dropdown
    │   ├─ performWslValidation()
    │   ├─ checkOllamaStatus()
    │   └─ loadProjectPaths()
    │
    └─ setStatus('Ready', 'success') ← PROBLEM: Happens immediately, not after above completes

Expected: Setup runs, models load, status changes to "Ready"
Actual: Status shows "Ready" immediately, setup fails silently
```

### Ollama Integration Points
```
UI checkOllamaStatus()
├─ check_ollama_installed (shell: ollama --version)
├─ check_ollama_running (HTTP: /api/tags)
├─ start_ollama_server (shell: ollama serve)
├─ list_ollama_models (HTTP: /api/tags, parse JSON)
└─ pull_ollama_model (shell: ollama pull MODEL)

Backend get_available_models()
├─ OllamaClient::new() (http timeout: 30s)
├─ client.list_models() (GET /api/tags)
└─ Parse response.models[].name

Issue: Two different code paths for same operation!
```

### Setup Flow
```
checkSetup() in UI
├─ Calls: get_setup_status() (backend)
│   ├─ Checks: ollama_installed (via list_models())
│   ├─ Checks: templates_exist
│   ├─ Checks: corpus_exists (mud-references/)
│   └─ Checks: index_built (.index file)
│
├─ If first_run && !index_built:
│   └─ Show confirm dialog
│       └─ If OK → run_initial_setup()
│           ├─ ContextManager::ensure_templates_exist()
│           ├─ extract_archives() (unzip mud-references)
│           └─ MudReferenceIndex::build_index()
│
└─ mark_setup_complete() (set flag)
```

---

## All Files Affected & Their Roles

### **Windows Installer Build**
```
target/release/bundle/
├── msi/LPC Dev Assistant_1.3.0_x64_en-US.msi
└── nsis/LPC Dev Assistant_1.3.0_x64-setup.exe
```
- Created by cargo tauri build
- Bundles: ui/, templates/, mud-references/index.jsonl, mud-references/*.txt
- **Problem:** Doesn't verify post-install setup

### **Configuration & State**
```
%APPDATA%/lpc_dev_assistant_driver_config.json (created at first run)
└── Stores: wsl_username, wsl_driver_root, wsl_library_root, staging_directory, setup_complete

.setup_complete (project root, not .lpc-dev-assistant/staging)
└── Marker file used to detect first-run (if doesn't exist → first_run=true)
```

### **Runtime Resources**
```
Project Root
├── templates/ (15 files)
│   └── Used to build system prompts for Ollama
├── mud-references/
│   ├── index.jsonl (indexed MUD code snippets)
│   ├── *.txt (reference documentation)
│   └── extracted/ (decompressed archives)
└── .index/ (Tantivy index, created at first run)
```

### **Crate Dependencies**
```
Key dependencies for operation:
├── tauri 2.1 (IPC, windowing)
├── tokio (async runtime)
├── reqwest (HTTP client for Ollama)
├── serde/serde_json (serialization)
├── tantivy 0.22 (full-text search indexing)
├── walkdir (directory traversal)
├── regex (pattern matching)
├── zip, tar, flate2 (archive extraction)
└── dirs (config/cache paths)
```

---

## Summary Table: Every Component

| Component | File | Status | Issue |
|-----------|------|--------|-------|
| **Frontend** | ui/index.html | ✅ Renders | ⚠️ Silent error handling |
| **Theme** | ui/index.html:14-28 | ✅ GitHub Dark | ✅ Looks good |
| **Tab System** | ui/index.html:348+ | ✅ Works | ✅ No issues |
| **Tauri IPC** | ui/index.html:1200+ | ✅ API calls work | ⚠️ Errors not shown |
| **Initialization** | ui/index.html:1481-1520 | ❌ Broken | 🔴 Runs setup in BG, shows "Ready" immediately |
| **Setup Detection** | src-tauri/src/main.rs:48-67 | ⚠️ Checks conditions | 🔴 Fails if Ollama not running |
| **Setup Execution** | src-tauri/src/main.rs:73-90 | ✅ Implemented | 🔴 Never called from UI |
| **Model Detection** | src-tauri/src/main.rs:617-635 | ✅ Implemented | ⚠️ JS calls wrong function |
| **Ollama Client** | src/ollama_client.rs:58+ | ✅ Works | ⚠️ No pre-check for connectivity |
| **Assistant Tab (Ask)** | ui/index.html:800-850 | ✅ Works | ✅ If Ollama running |
| **Staging Tab** | ui/index.html:650-700 | ✅ Works | ✅ No issues |
| **Settings Tab** | ui/index.html:550-650 | ⚠️ Partial | ⚠️ Models don't load at init |
| **Coming Soon UI** | ui/index.html:641-649 | ✅ Disabled | ⚠️ Visual clutter |
| **File Operations** | src-tauri/src/commands/file_operations.rs | ✅ Works | ✅ No issues |
| **Staging Operations** | src-tauri/src/commands/staging.rs | ✅ Works | ✅ No issues |
| **Config System** | src-tauri/src/config.rs | ✅ Works | ✅ No issues |
| **WSL Handling** | src-tauri/src/commands/... | ✅ Works | ✅ No issues |
| **RAG/Search** | src/mud_index.rs | ✅ Works | ✅ After setup complete |

---

## What's Actually Working

✅ **These things DO work:**
1. **UI Rendering** - All tabs visible, GitHub Dark theme applied
2. **File Operations** - save_to_driver, save_to_library fully functional
3. **Staging System** - save_to_staging, list_staged_files, copy_staged_to_project
4. **WSL Integration** - Path mapping, validation, diagnostics
5. **Ollama Generation** - If Ollama running with model installed → generation works
6. **Backend Architecture** - All 35+ commands properly exposed
7. **Configuration** - Config file creation, persistence, loading

✅ **What ALMOST works:**
- Setup (exists but never triggered automatically)
- Model detection (function exists but JS uses wrong one)
- Ollama integration (works if manually started)

---

## Installation Testing Scenarios

### Scenario 1: Fresh Windows Install (Worst Case)
```
1. User downloads installer
2. User runs installer
3. User launches app
4. App shows "LPC Dev Assistant | Ready" ← LIE! Setup not done
5. User clicks "Assistant" tab
6. User types prompt and clicks "Ask"
7. Error: "Failed to connect to Ollama. Is it running?"
8. User is confused - app said "Ready"!
```

### Scenario 2: Ollama Installed But Not Running
```
1. User has Ollama executable installed
2. User runs LPC Dev Assistant
3. App checks: "Is Ollama running?" → NO
4. Error caught silently in console.warn
5. Status shows "Ready" anyway
6. User tries to generate code
7. Error: "Failed to connect to Ollama"
```

### Scenario 3: Ollama Running But No Model
```
1. Ollama running (ollama serve in terminal)
2. No models pulled yet
3. App tries to list models
4. get_available_models() returns []
5. loadModels() silently fails
6. UI never shows "please install qwen2.5-coder:7b"
7. User tries to ask question
8. Error: "Model qwen2.5-coder:7b not found"
```

---

## Next Steps for Development

The project structure is **solid**, but the **initialization flow is broken**. To continue development, you'll need to:

1. **Fix error handling** - Surface errors to UI instead of console
2. **Fix initialization sequence** - Don't show "Ready" until setup verified
3. **Deduplicate model detection** - Choose ONE way to get model list
4. **Add proper pre-flight checks** - Verify Ollama before attempting operations
5. **Remove "Coming Soon" elements** - Or implement them

All backend code is functional and properly exposed via Tauri IPC. The issue is entirely in the **JavaScript initialization and error handling**, not the Rust backend.

---

## File Manifest for Development

**All files that need changes:**

### High Priority (Breaks Functionality)
- [ ] `ui/index.html` - Fix DOMContentLoaded init sequence (lines 1481-1520)
- [ ] `ui/index.html` - Fix error handling (wrap init in proper error boundaries)
- [ ] `ui/index.html` - Fix loadModels function (call correct backend function)
- [ ] `src-tauri/src/main.rs` - Remove `list_models()` OR `list_ollama_models()` (keep one)

### Medium Priority (Improves UX)
- [ ] `ui/index.html` - Remove "Coming Soon" elements (lines 641-649)
- [ ] `ui/index.html` - Add proper model loading UI (spinner, status updates)
- [ ] `src-tauri/src/main.rs` - Add connectivity check to OllamaClient::new()

### Low Priority (Polish)
- [ ] Add post-install setup verification
- [ ] Improve error messages
- [ ] Add logging/diagnostics mode

---

**End of Analysis**
