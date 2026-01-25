# RAG Enhancement & Model Comparison Guide

## Overview

This document describes the new RAG (Retrieval-Augmented Generation) validation pipeline and model comparison framework added to the LPC Development Assistant.

## Features

### 1. RAG Validation Pipeline (`rag_validator.rs`)

**Purpose**: Ensures that LPC-related queries retrieve and validate against ALL relevant examples and documentation from the corpus before generating responses.

**Key Capabilities**:
- Cross-references search results across multiple sources
- Validates answers against known-good examples
- Extracts and identifies efuns from retrieved documents
- Provides confidence scores for each query
- Categorizes sources by validation status:
  - ✓ **Verified**: Content found in 3+ sources
  - ⊕ **Cross-Referenced**: Found in 2+ sources  
  - ○ **Single Source**: Found in only one source
  - ? **Uncertain**: Conflicting information

**Usage**:
```rust
let validator = RAGValidator::new(mud_index, corpus_root)?;
let result = validator.validate_query("How do I use call_other()?")?;

println!("Confidence: {:.1}%", result.confidence_score * 100.0);
println!("Sources: {}", result.sources_consulted);
println!("Efuns found: {:?}", result.efuns_found);
```

### 2. Model Comparison Framework (`model_tester.rs`)

**Purpose**: Systematically test and compare different Ollama models for LPC code generation accuracy.

**Test Models**:
- `deepseek-coder` - Specialized for code generation
- `codellama` - Code-focused LLM
- `llama3` - General purpose, strong reasoning
- `mistral` - Lightweight and fast

**Evaluation Metrics**:
- **Accuracy**: Presence of expected keywords in responses
- **Quality**: Code completeness, LPC syntax awareness
- **Speed**: Response time and tokens/second
- **Confidence**: RAG validation scores

**Usage**:
```bash
# Run model comparison
cargo run --bin model_comparison

# Results saved to: model_comparison_results.json
```

### 3. Enhanced Query CLI (`enhanced_query.rs`)

**Purpose**: Interactive CLI that demonstrates the full RAG pipeline with source citations.

**Features**:
- Real-time source validation
- Confidence scoring
- Citation of consulted sources
- Efun detection
- Code example extraction

**Usage**:
```bash
cargo run --bin enhanced_query

# Example session:
/> How do I use call_other()?

🔍 Searching knowledge base...

📚 Source Validation:
  Confidence Score: 87.3%
  Validation Score: 91.2%
  Sources Consulted: 15
  Efuns Found: call_other, call_out

📄 Top Sources:
  1. [✓] interpret.c (line 1234) - 92%
  2. [✓] efun_defs.c (line 567) - 88%
  3. [⊕] object.c (line 890) - 85%

🤖 Generating response with deepseek-coder model...

📝 Answer:
======================================================================
[Response with code examples and explanations]
======================================================================

Answer based on 15 sources from the MUD reference corpus
```

## New Binaries

### `model_comparison`
Runs comprehensive comparison of all available Ollama models against standard LPC queries.

```bash
cargo run --bin model_comparison
```

**Output**:
- Console report with best model recommendations
- `model_comparison_results.json` with detailed metrics

### `enhanced_query`
Interactive query tool with full RAG validation and source citations.

```bash
cargo run --bin enhanced_query
```

## Test Queries

The model comparison uses these standard queries:

1. **Combat System**: "How do I implement a combat system in LPC?"
   - Tests: Object-oriented design, inheritance, mudlib patterns

2. **Query Functions**: "Show me the correct syntax for query_* functions"
   - Tests: Function naming conventions, return types

3. **Efun Comparison**: "What's the difference between call_other() and call_out()?"
   - Tests: Understanding of core efuns, async concepts

4. **Code Generation**: "Generate a basic room inherit for the std/room.c"
   - Tests: Code synthesis, inheritance patterns

5. **Object System**: "How does the LPC object inheritance system work?"
   - Tests: Conceptual understanding, architecture knowledge

## Integration with Existing Code

### Adding RAG Validation to Your Queries

```rust
use lpc_dev_assistant::{RAGValidator, MudReferenceIndex};

// Initialize
let mut index = MudReferenceIndex::new(corpus_path);
index.build_index()?;

let validator = RAGValidator::new(index, corpus_path)?;

// Validate query
let result = validator.validate_query(user_query)?;

// Check confidence before proceeding
if result.confidence_score < 0.5 {
    println!("Low confidence - may need more context");
}

// Use retrieved documents in prompt
for doc in result.retrieved_documents.iter().take(5) {
    println!("Source: {}", doc.path);
    println!("Relevance: {:.0}%", doc.relevance_score * 100.0);
}
```

### Running Model Tests Programmatically

```rust
use lpc_dev_assistant::ModelTester;

let tester = ModelTester::new(ollama_client, prompt_builder, Some(validator));

// Use default queries or create custom ones
let queries = ModelTester::get_default_test_queries();

// Run comparison
let results = tester.compare_models(vec!["deepseek-coder".to_string()], queries).await?;

// Access metrics
println!("Best model: {}", results.summary.recommended_model);
```

## Corpus Statistics

Current indexed corpus:
- **Total Files**: 25,427
- **Documentation Chunks**: 49,094  
- **Known Efuns**: 166+
- **Driver Sources**: MudOS, FluffOS, DGD, LDMud, Nightmare, Merentha

### Source Priorities

The RAG system prioritizes sources in this order:
1. Driver source files (`.c`, `.h` from mudos/fluffos/dgd)
2. Efun definition files
3. Mudlib implementations
4. Documentation files

## Performance Considerations

### Indexing
- First run: ~30-60 seconds for full corpus
- Subsequent runs: Use cached index (not yet implemented)
- Memory: ~200-500MB for full index

### Query Validation
- Average validation time: 100-300ms
- Searches up to 25 documents per query
- Cross-references results for accuracy

### Model Generation
- Varies by model (500ms - 5000ms)
- Depends on context size and model parameters
- Temperature set to 0.3 for consistency

## Recommended Workflow

### 1. Initial Setup
```bash
# Ensure Ollama is running
ollama serve

# Pull recommended models
ollama pull deepseek-coder
ollama pull codellama
ollama pull llama3
```

### 2. Run Model Comparison (One-Time)
```bash
cargo run --bin model_comparison
# Review model_comparison_results.json
# Note the recommended model
```

### 3. Use Enhanced Query Tool
```bash
cargo run --bin enhanced_query
# Select your preferred model
# Start asking LPC questions
```

### 4. Integration
Use the recommended model and RAG validation in your main application.

## Future Enhancements

**Planned**:
- [ ] Persistent index caching
- [ ] Semantic search with embeddings
- [ ] Multi-query expansion
- [ ] Answer verification against actual code compilation
- [ ] Integration with live AMLP driver for testing
- [ ] Fine-tuning dataset generation from validated Q&A pairs

**In Progress**:
- [x] RAG validation pipeline
- [x] Model comparison framework
- [x] Source citation system

## Troubleshooting

### "No models found"
```bash
# Check Ollama is running
ollama list

# Pull a model
ollama pull deepseek-coder
```

### "Failed to build index"
- Ensure `mud-references/` directory exists
- Check file permissions
- Verify disk space (needs ~1GB free)

### "Low confidence scores"
- Query may be too vague
- Try more specific questions
- Check if efuns are spelled correctly
- Reference mudlib may not have examples for your query

## Contributing

When adding new test queries to model comparison:

1. Choose queries that test different aspects:
   - Efun knowledge
   - Code generation
   - Architectural understanding
   - Mudlib patterns

2. Define expected keywords for accuracy scoring

3. Categorize by `QueryCategory` for analysis

## License

Same as parent project.
