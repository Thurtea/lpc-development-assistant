# Iced GUI Status & Immediate Action Plan

## Current Status
✅ **Iced GUI Builds Successfully** (release mode, 6.1 MB)
✅ **Running** without errors (terminal ID: c6299aef-1c3e-4f63-b847-f57fcb8bf1a3)
✅ **Syntax Highlighting** with syntect already integrated in `gui.rs`
✅ **Driver Analysis** framework in place (calls `driver_analyzer::efuns_json()`)

## Critical Issues to Fix (User-Reported)

### 1. **Driver Analyzer Regex Patterns** (HIGHEST PRIORITY)
**Current Issue:** Generic driver context instead of real efuns from merentha_fluffos_v2

**Location:** `src/driver_analyzer.rs` lines 10-26

**Patterns to Update:**
- Primary: `#ifdef F_FUNCTIONNAME` (most reliable for FluffOS)
- Secondary: `void f_functionname()` implementations
- Array tables: `{ "funcname", F_FUNCNAME }`

**Sample Real Efuns Found in merentha:**
```
#ifdef F_ALLOCATE
void f_allocate()
#ifdef F_ALL_INVENTORY  
void f_all_inventory()
#ifdef F_ADD_ACTION
void f_add_action()
```

**Action:** Reorder regex priority: ifdef F_* first → f_* functions → tables

### 2. **Syntax Highlighting Enhancement** (MEDIUM)
Current: Plain `render_highlighted_element()` uses syntect but needs color styling

**Enhancement:** Add proper color mapping via syntect styles → iced rich_text! spans
- Keywords: Blue
- Strings: Green  
- Comments: Gray
- Type names: Cyan

### 3. **GUI Layout Improvement** (MEDIUM)
Current: Linear vertical layout

**Target:** Split left (input + driver analysis) | Right (highlighted response + efuns list)

## Immediate Manual Test Sequence

### Test 1: Ollama Connection
```
Question: "Explain LPC heartbeat"
Expected: 30-60 second response with working code
```

### Test 2: Driver Analysis
```
Driver Path: E:\Work\AMLP\mud-references\merentha\merentha_fluffos_v2\fluffos-2.9-ds2.03
Click "🔧 Analyze Driver"
Expected: List of real efuns (add_allocate, all_inventory, add_action, etc.) with file paths
```

### Test 3: Save Generated Code
```
After response received → Click "💾 Save"
Expected: File saved to gen/mudlib_20260119HHMMSS.c
```

## Next Steps
1. ✋ **PAUSE** - Await test results from Iced GUI
2. **Update driver_analyzer.rs** with improved regex patterns
3. **Enhance syntax highlighting** in gui.rs with color spans
4. **Refactor layout** to split-view left/right
5. **Re-run** and commit

## Files Modified
- `src/bin/gui.rs` - Main Iced application (699 lines)
- `src/driver_analyzer.rs` - Regex pattern matching (116 lines)
- `Cargo.toml` - Dependencies already include syntect (5.2)

## Build Command (if needed)
```powershell
cd E:\Work\AMLP\lpc-dev-assistant
cargo build --bin gui --features iced-gui --release
```

## Run Command
```powershell
E:\Work\AMLP\lpc-dev-assistant\target\release\gui.exe
```

---

**Status:** ✅ Ready for manual testing. Iced GUI is live and waiting for user interaction.
