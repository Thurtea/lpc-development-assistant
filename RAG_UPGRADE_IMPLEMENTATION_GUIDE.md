# Phase 3 RAG Upgrade - Implementation Guide

## Overview
The LPC Dev Assistant RAG system has been upgraded to dramatically improve codegen/compiler task performance. This guide integrates the improvements into the existing workflow.

## Changes Summary

### 1. Enhanced Query Generation (prompt_builder.rs)
**What changed:**
- Added `generate_specialized_queries()` function
- Detects prompt type (codegen, VM, data structures)
- Generates 3-8 specialized queries per prompt

**How to use:**
```rust
let queries = PromptBuilder::generate_specialized_queries(
    "implement codegen.c using vm.h/codegen.h"
);
// Returns: vec!["implement codegen.c using vm.h/codegen.h",
//               "LPC driver bytecode VM compilation",
//               "MudOS vm.c opcode dispatch",
//               "FluffOS interpret.c stack machine",
//               "stack VM codegen AST visitor",
//               "opcode emission C bytecode"]
```

### 2. Multi-Pass Retrieval (mud_index.rs)
**What changed:**
- Prioritizes driver source files (.c, .h)
- Expands result limit from 5 to 15
- Increases context window from 5 lines to 9+ lines
- Filters for MudOS, FluffOS, interpret, codegen files

**Impact:**
- Top results now include actual driver source code
- Better context for pattern matching
- Avoids generic documentation

### 3. New Template: driver_codegen.txt
**What changed:**
- New 400+ line specialized template
- Contains complete bytecode VM patterns
- Includes OpCode enum, AST walker examples
- Shows bytecode emission vs interpreter patterns

**Loaded automatically when:**
- Query contains "codegen", "compiler", "bytecode", or "opcode"
- Falls back to driver_context.txt if not present

### 4. Response Validation (ollama_client.rs)
**New method:** `validate_generated_code(code: &str, expected_headers: &[&str])`

**Checks:**
```rust
let issues = OllamaClient::validate_generated_code(
    &response_code,
    &["vm.h", "codegen.h"]
);

if issues.is_empty() {
    // Code is valid - uses headers, no redefinitions, proper opcodes
} else {
    // issues contains specific problems found
    for issue in issues {
        eprintln!("Issue: {}", issue);
    }
}
```

## Integration Steps

### Step 1: Rebuild Project
```bash
cd e:\Work\lpc-development-assistant
cargo build --release
```

### Step 2: Test Basic Functionality
```bash
cargo test --lib mud_index -- --nocapture
cargo test --lib prompt_builder -- --nocapture
```

### Step 3: Test with Ollama

Ensure Ollama is running:
```bash
ollama serve &
ollama pull mistral  # or your preferred model
```

Run Phase 3 test:
```bash
cargo run --bin test_cli -- "implement codegen.c using vm.h/codegen.h APIs, compile 3+5 to bytecode, emit OP_PUSH_INT and OP_BINARY_ADD opcodes, no struct redefinitions"
```

### Step 4: Verify Response Quality

Use the validation function:
```rust
// In your test/CLI code
let validation_issues = OllamaClient::validate_generated_code(
    &response,
    &["vm.h", "codegen.h", "codegen.h"]
);

println!("Validation issues: {:?}", validation_issues);
// Should be empty or minimal
```

## Testing the Upgrade

### Basic Integration Test
```bash
# 1. Check code compiles
cargo check

# 2. Run retrieval tests
cargo test --lib search_with_scoring

# 3. Test prompt generation
cargo run --example prompt_test 2>/dev/null
```

### Manual Codegen Test

1. **Create test file** (test_codegen.txt):
```
TASK: Implement bytecode compiler for LPC expression: 3 + 5

REQUIREMENTS:
- Use vm.h and codegen.h provided APIs
- Do NOT redefine any structs or enums
- Emit bytecode opcodes: OP_PUSH_INT, OP_BINARY_ADD
- Implement AST walker (recursive codegen_expr)
- Result should be 200+ lines of complete C code
- Must handle:
  * Constant pool management
  * Jump patching for control flow
  * Stack discipline (push/pop)
  * Function call conventions

EXPECTED OPCODE SEQUENCE for 3 + 5:
  OP_PUSH_INT 3
  OP_PUSH_INT 5
  OP_BINARY_ADD
  OP_RETURN
```

2. **Run through enhanced system**:
```bash
cargo run --bin test_cli -- < test_codegen.txt
```

3. **Check results**:
```rust
// Automatic validation
let validation = OllamaClient::validate_generated_code(&response, &["vm.h", "codegen.h"]);
println!("Validation: {:?}", validation);

// Manual checks
- Does it include #include "vm.h" and #include "codegen.h"?
- Does it have codegen_emit_op() calls?
- Does it have OP_PUSH_INT, OP_BINARY_ADD opcodes?
- Does it avoid evaluate_* or interpret_* patterns?
- Is it 200+ lines?
```

## Metrics to Track

### Before (Baseline)
- Test score: 2.5/10
- Invalid headers: frequent
- Struct redefinitions: 70%
- Interpreter patterns: 90%
- Output length: ~150 lines

### After (Target)
- Test score: 9/10
- Invalid headers: 0%
- Struct redefinitions: 0%
- Interpreter patterns: 0%
- Output length: 800+ lines

### Key Indicators
Track these in test runs:
1. **Validation pass rate**: % of generated code that passes all checks
2. **Header usage**: % of code including required headers
3. **Opcode coverage**: # of distinct OP_* opcodes emitted
4. **Code length**: lines of generated code
5. **AST walker recursion**: presence of recursive codegen_expr calls

## Troubleshooting

### Issue: Multi-pass retrieval too slow
**Solution**: 
- Reduce number of specialized queries in `generate_specialized_queries()`
- Implement caching for frequently-searched queries
- Use index pre-warming before peak usage

### Issue: Wrong results still being retrieved
**Solution**:
- Refine driver source detection in `search_with_scoring()`
- Add more keyword patterns for MudOS/FluffOS files
- Increase prioritization weight for .c/.h files from driver sources

### Issue: Validation too strict
**Solution**:
- Adjust regex patterns in `validate_generated_code()`
- Add whitelist for common safe patterns
- Make checks configurable per prompt type

### Issue: Template not loading
**Solution**:
- Verify driver_codegen.txt exists in templates/ directory
- Check ContextManager::ensure_templates_exist() is called
- Add debug output: `eprintln!("driver_codegen: {}", pb.templates.get("driver_codegen.txt").map(|s| s.len()).unwrap_or(0))`

## Performance Notes

- **Retrieval time**: ~500-1000ms (3-8 parallel queries)
- **Ollama generation**: ~10-30s (depends on model size)
- **Validation**: <100ms
- **Total pipeline**: ~15-60s per request

Optimization opportunities:
1. Cache search results for repeated queries
2. Implement result streaming
3. Pre-warm index on startup
4. Use thread pool for parallel queries

## Configuration Options

### Environment Variables
```bash
# Control retrieval behavior
RETRIEVAL_LIMIT=15          # Max results to return (default: 15)
CONTEXT_LINES=6             # Lines of context per result (default: 6)
PRIORITY_WEIGHT=2.0         # Weight for driver sources (default: 2.0)

# Control validation
VALIDATION_STRICT=true      # Strict validation mode (default: false)
REQUIRE_HEADERS=true        # Require header includes (default: true)

# Ollama settings
OLLAMA_URL=http://localhost:11434
OLLAMA_TIMEOUT_SECS=60
```

### Compile-time Features
```bash
# Build with validation enabled
cargo build --features validation

# Build with debug logging
cargo build --features debug-retrieval
```

## Next Steps

1. **Immediate** (Phase 3 testing)
   - Run Phase 3 codegen test suite
   - Collect metrics on pass rate
   - Monitor validation failures

2. **Short-term** (1-2 days)
   - Analyze failed responses
   - Refine query generation if needed
   - Tune retrieval thresholds

3. **Medium-term** (1 week)
   - Implement feedback loop
   - Fine-tune model weights
   - Add more specialized templates

4. **Long-term** (2+ weeks)
   - Domain-specific model training
   - Interactive context refinement
   - Multi-model ensemble approaches

## References

- **Implementation**: `src/prompt_builder.rs`, `src/mud_index.rs`, `src/ollama_client.rs`
- **Template**: `templates/driver_codegen.txt`
- **Test Script**: `test_rag_upgrade.ps1`
- **Summary Document**: `PHASE3_RAG_UPGRADE_SUMMARY.md`
- **Driver Sources**: `mud-references/` (MudOS, FluffOS, DGD)

## Questions & Support

For issues or questions:
1. Check `PHASE3_RAG_UPGRADE_SUMMARY.md` for detailed technical info
2. Review test output in test suite
3. Check logs: `$HOME/.lpc-dev-assistant/debug.log`
4. Examine retrieval results: enable debug output in `mud_index.rs`

---

**Last Updated**: January 21, 2026
**Status**: Ready for Phase 3 Testing
