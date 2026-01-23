# Integration with amlp-driver

## Current Status
- ✅ **amlp-driver**: Complete with working execution pipeline (4/4 tests passing)
- ✅ **lpc-development-assistant**: RAG system and UI ready
- ✅ **WSL Bridge**: Implemented in `src-tauri/src/wsl/` (command_executor, path_mapper)
- ✅ **Driver Pipeline**: Implemented in `src-tauri/src/driver/pipeline.rs`
- 🚧 **Full E2E Testing**: Not yet completed

## Integration Architecture

```
┌─────────────────────────────────────────────────────────────┐
│  lpc-development-assistant (Windows - Rust/Tauri)           │
│  ┌──────────────┐   ┌──────────────┐   ┌─────────────┐     │
│  │   UI Layer   │   │  Tauri IPC   │   │  Rust Core  │     │
│  │ index.html   │──▶│  main.rs     │──▶│  pipeline   │     │
│  └──────────────┘   └──────────────┘   └─────────────┘     │
│                                              │               │
│                                              ▼               │
│                                      ┌─────────────────┐    │
│                                      │  WslExecutor    │    │
│                                      │  command_exec.  │    │
│                                      └─────────────────┘    │
└──────────────────────────────────────────┬──────────────────┘
                                           │ WSL commands
                                           ▼
┌─────────────────────────────────────────────────────────────┐
│  WSL (Linux - FedoraLinux-43)                               │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  /home/thurtea/amlp-driver/build/driver             │   │
│  │  Commands: compile | ast | bytecode | run          │   │
│  └─────────────────────────────────────────────────────┘   │
│                           │                                 │
│                           ▼                                 │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  /home/thurtea/amlp-library/                        │   │
│  │  LPC source files: master.c, domains/, docs/        │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                           │
                           ▼ (compilation results)
┌─────────────────────────────────────────────────────────────┐
│  Back to lpc-development-assistant                          │
│  - Parse stdout/stderr                                      │
│  - Display in UI with color coding                          │
│  - Show bytecode/AST if requested                           │
└─────────────────────────────────────────────────────────────┘
```

## Phase 1: WSL Bridge ✅ COMPLETE

**Status**: Implemented and tested

**Files**:
- `src-tauri/src/wsl/command_executor.rs` - Execute WSL commands with timeout protection
- `src-tauri/src/wsl/path_mapper.rs` - Map Windows paths to WSL paths
- `src-tauri/src/wsl/mod.rs` - Module exports

**Features**:
- ✅ Execute arbitrary WSL commands with working directory
- ✅ Stream stdout/stderr in real-time
- ✅ Timeout protection (5 minute default, configurable)
- ✅ EOF detection to prevent infinite loops
- ✅ Shell escaping to prevent command injection
- ✅ Unit tests for path escaping

**Recent Bug Fixes** (Commit `a75652d9`):
- Fixed infinite loop in command_executor by tracking EOF explicitly
- Added timeout protection (5min default) to prevent hung commands
- Enhanced shell escaping to prevent command injection attacks ($, `, \, ')
- Support both WSL and Windows paths in compilation
- Improved error diagnostics in test_driver_connection
- Changed compile_lpc to return structured data instead of JSON string
- Added unit tests for shell escaping functions

## Phase 2: Driver Client ✅ COMPLETE

**Status**: Implemented and tested

**Files**:
- `src-tauri/src/driver/pipeline.rs` - High-level wrapper for amlp-driver CLI

**Implementation**:
```rust
pub struct DriverPipeline {
    executor: WslExecutor,
    paths: Arc<PathMapper>,
}

impl DriverPipeline {
    pub async fn compile(&self, file_path: &str, ...) -> Result<CompileResult>
    pub async fn ast(&self, file_path: &str, ...) -> Result<CompileResult>
    pub async fn bytecode(&self, file_path: &str, ...) -> Result<CompileResult>
    pub async fn test(&self, ...) -> Result<CompileResult>
}
```

**Features**:
- ✅ Wraps all 4 driver commands (compile, ast, bytecode, run)
- ✅ Path validation for both WSL and Windows paths
- ✅ Returns structured `CompileResult` with success/exit_code/stdout/stderr
- ✅ Shell escaping for safe path handling

## Phase 3: UI Integration ✅ COMPLETE

**Status**: Implemented in Driver tab

**Files**:
- `ui/index.html` - Driver tab UI
- `src-tauri/src/main.rs` - Tauri command handlers

**Tauri Commands**:
```rust
#[tauri::command]
async fn test_driver_connection(state: ...) -> Result<String, String>

#[tauri::command]
async fn compile_lpc(file_path: String, state: ...) -> Result<CompileResult, String>

#[tauri::command]
async fn build_driver_ui(state: ...) -> Result<String, String>
```

**UI Features**:
- ✅ "Test Driver Connection" button with status display
- ✅ "Compile File" button with path input
- ✅ Color-coded output (green=success, red=error)
- ✅ Exit code display
- ✅ Stdout/stderr separation

## Phase 4: E2E Testing 🚧 IN PROGRESS

**Status**: Pre-flight checks passed, app UI needs full testing

**Pre-Flight Results** (Jan 23, 2026):
- ✅ WSL installed and running (FedoraLinux-43)
- ✅ Driver binary built and executable (`./build/driver --help` works)
- ✅ Library directory exists with master.c
- ✅ App compiles successfully with Tauri
- ⏳ Full UI testing pending

**Test Checklist**:
- [ ] Tab switching (Driver ↔ Assistant)
- [ ] Driver connection test button
- [ ] LPC compilation with valid file
- [ ] Error handling for invalid paths
- [ ] Empty path validation
- [ ] Timeout protection verification
- [ ] Shell escaping security test

**Next Steps**:
1. Complete E2E UI testing in Tauri dev mode
2. Test with actual LPC files from amlp-library
3. Verify ANSI output handling
4. Document any issues found
5. Create screenshots for documentation

## Phase 5: Auto-Apply Pipeline 🔮 FUTURE

**Status**: Not yet started

**Goal**: Close the loop from AI prompt → code generation → compilation → results

**Proposed Flow**:
```
User Prompt → Ollama (code generation) → Parse LPC code 
    ↓
Write to /home/thurtea/amlp-library/generated/
    ↓
Trigger compile_lpc command
    ↓
Display results in UI (success/errors)
    ↓
If errors: Feed back to Ollama for fixes
```

**Considerations**:
- Where to store generated files? (`amlp-library/generated/` or `amlp-library/workspace/`?)
- Auto-backup before overwriting?
- Git integration for version control?
- Iterative refinement loop (fix compilation errors automatically)?

## Current Blockers

None - ready to proceed with E2E testing.

## Developer Handoff Notes

**To continue integration work**:

1. **Start dev server**: `cargo tauri dev` (in `e:\Work\lpc-development-assistant`)
2. **Driver location**: `/home/thurtea/amlp-driver/build/driver` (WSL)
3. **Library location**: `/home/thurtea/amlp-library/` (WSL)
4. **Test command**: `wsl.exe -e bash -lc "cd /home/thurtea/amlp-driver && ./build/driver --help"`

**Key Architecture Points**:
- All WSL interaction goes through `WslExecutor` (handles timeouts, EOF, escaping)
- `PathMapper` converts between Windows and WSL paths
- `DriverPipeline` provides high-level API for driver commands
- Tauri IPC handles async communication between Rust and JavaScript

**Testing Resources**:
- Driver test suite: `cd /home/thurtea/amlp-driver && make test`
- LPC test files: `/home/thurtea/amlp-library/domains/` or create in `/tmp/`
- Assistant test: `cargo test --lib` (in Rust codebase)

**Known Issues**:
- None currently - all bugs from previous session fixed and committed

**Documentation Links**:
- [WSL Command Executor](../src-tauri/src/wsl/command_executor.rs)
- [Driver Pipeline](../src-tauri/src/driver/pipeline.rs)
- [Main Tauri Handlers](../src-tauri/src/main.rs)
- [Driver Tab UI](../ui/index.html) (search for "Driver" tab)
