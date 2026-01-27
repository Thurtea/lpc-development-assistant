use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use crate::mud_index::MudReferenceIndex;

pub struct PromptBuilder {
    _templates_dir: PathBuf,
    templates: HashMap<String, String>,
    reference_index: Option<MudReferenceIndex>,
}

impl PromptBuilder {
    /// Generate specialized retrieval queries based on prompt type
    fn generate_specialized_queries(query: &str) -> Vec<String> {
        let query_lower = query.to_lowercase();
        let mut queries = vec![query.to_string()];

        let is_codegen = query_lower.contains("codegen") || query_lower.contains("compiler") ||
            query_lower.contains("bytecode") || query_lower.contains("opcode");
        let is_vm = query_lower.contains("vm") || query_lower.contains("virtual machine") ||
            query_lower.contains("interpreter") || query_lower.contains("execute");
        let is_struct = query_lower.contains("struct") || query_lower.contains("typedef") ||
            query_lower.contains("data structure");
        let is_object = query_lower.contains("object") || query_lower.contains("inherit") ||
            query_lower.contains("call_other") || query_lower.contains("call method") ||
            query_lower.contains("shadow");
        let is_efun = query_lower.contains("efun") || query_lower.contains("call_out") ||
            query_lower.contains("simul") || query_lower.contains("apply") ||
            query_lower.contains("function_exists");

        if is_codegen {
            queries.extend(vec![
                "LPC driver bytecode VM compilation".to_string(),
                "MudOS vm.c opcode dispatch".to_string(),
                "FluffOS interpret.c stack machine".to_string(),
                "stack VM codegen AST visitor".to_string(),
                "opcode emission C bytecode".to_string(),
            ]);
        }

        if is_vm {
            queries.extend(vec![
                "stack-based virtual machine implementation".to_string(),
                "LPC driver VM execution loop".to_string(),
                "bytecode interpreter instruction dispatch".to_string(),
                "call stack frame management".to_string(),
            ]);
        }

        if is_struct {
            queries.extend(vec![
                "tagged union VMValue LPC".to_string(),
                "OpCode enum C definition".to_string(),
                "symbol table codegen".to_string(),
            ]);
        }

        if is_object {
            queries.extend(vec![
                "LPC object inheritance chain resolution".to_string(),
                "MudOS object.c call_other implementation".to_string(),
                "FluffOS object.c CALL_METHOD opcode".to_string(),
                "LPC object variables inherit offsets".to_string(),
                "environment inventory traversal driver".to_string(),
            ]);
        }

        if is_efun {
            queries.extend(vec![
                "LPC efun binding table".to_string(),
                "FluffOS efun_defs.c call_other".to_string(),
                "simul_efun dispatch master object".to_string(),
                "call_out scheduler driver".to_string(),
                "children() and living() driver code".to_string(),
            ]);
        }

        // Always add general LPC driver context
        if !query_lower.contains("driver") {
            queries.push("LPC driver C implementation".to_string());
        }

        queries
    }

    pub fn new(templates_dir: PathBuf) -> Result<Self, String> {
        let mut pb = PromptBuilder {
            _templates_dir: templates_dir.clone(),
            templates: HashMap::new(),
            reference_index: None,
        };

        // Attempt to load known templates; missing files become empty strings
        let names = [
            "driver_context.txt",
            "mudlib_context.txt",
            "efuns_context.txt",
            "reference_sources.txt",
            "driver_codegen.txt",
            "object_system.txt",
        ];

        for name in &names {
            let path = templates_dir.join(name);
            let content = fs::read_to_string(&path).unwrap_or_else(|_| String::new());
            pb.templates.insert(name.to_string(), content);
        }

        Ok(pb)
    }

    /// Fallback constructor that never fails; templates map is empty
    pub fn new_empty(templates_dir: PathBuf) -> Self {
        PromptBuilder {
            _templates_dir: templates_dir,
            templates: HashMap::new(),
            reference_index: None,
        }
    }

    pub fn with_reference_index(mut self, index: MudReferenceIndex) -> Self {
        self.reference_index = Some(index);
        self
    }

    pub fn build_prompt(&self, user_query: &str, _model: &str, examples: Vec<String>) -> Result<String, String> {
        // System header with enhanced role
        let mut out = String::new();
        out.push_str("You are LPC Driver Architect. IMPLEMENT EXACTLY using provided headers and APIs.\n");
        out.push_str("NO redefinitions. Emit bytecode only.\n");
        out.push_str("Traverse AST recursively. Match MudOS/FluffOS patterns from context.\n\n");

        // Detect if this is a specialized query and load appropriate template
        let query_lower = user_query.to_lowercase();
        let is_codegen = query_lower.contains("codegen") || query_lower.contains("compiler") || 
                        query_lower.contains("bytecode") || query_lower.contains("opcode");
        let is_object = query_lower.contains("object") || query_lower.contains("inherit") ||
                        query_lower.contains("call_other") || query_lower.contains("call method") ||
                        query_lower.contains("shadow");
        let is_efun = query_lower.contains("efun") || query_lower.contains("call_out") ||
                      query_lower.contains("simul") || query_lower.contains("function_exists");
        
        let driver = if is_codegen {
            self.templates.get("driver_codegen.txt")
                .filter(|s| !s.is_empty())
                .map(|s| s.as_str())
                .unwrap_or_else(|| self.templates.get("driver_context.txt").map(|s| s.as_str()).unwrap_or(""))
        } else {
            self.templates.get("driver_context.txt").map(|s| s.as_str()).unwrap_or("")
        };

        let object_system = self.templates.get("object_system.txt").map(|s| s.as_str()).unwrap_or("");
        
        let mudlib = self.templates.get("mudlib_context.txt").map(|s| s.as_str()).unwrap_or("");
        let efuns = self.templates.get("efuns_context.txt").map(|s| s.as_str()).unwrap_or("");
        let refs = self.templates.get("reference_sources.txt").map(|s| s.as_str()).unwrap_or("");

        out.push_str(driver);
        out.push_str("\n\n");

        if (is_object || is_efun) && !object_system.is_empty() {
            out.push_str(object_system);
            out.push_str("\n\n");
        }

        // Multi-pass retrieval: generate specialized queries
        let all_queries = Self::generate_specialized_queries(user_query);
        let mut all_results = Vec::new();
        
        if let Some(ref index) = self.reference_index {
            // Collect results from all specialized queries
            for query in &all_queries {
                if let Ok(search_results) = index.search_with_scoring(query, 8) {
                    all_results.extend(search_results);
                }
            }
            
            // Deduplicate and sort by relevance
            let mut unique_results = HashMap::new();
            for result in all_results {
                let key = result.path.clone();
                unique_results.entry(key)
                    .and_modify(|r: &mut crate::mud_index::SearchResult| {
                        if result.relevance_score > r.relevance_score {
                            *r = result.clone();
                        }
                    })
                    .or_insert(result);
            }
            
            let mut sorted_results: Vec<_> = unique_results.into_values().collect();
            sorted_results.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap_or(std::cmp::Ordering::Equal));
            
            // Include top 15 results (not just 5)
            if !sorted_results.is_empty() {
                out.push_str("CRITICAL REFERENCES (Top 15 chunks, prioritize MudOS/FluffOS patterns):\n\n");
                for (idx, result) in sorted_results.iter().take(15).enumerate() {
                    out.push_str(&format!("[{}] From: {}\n", idx + 1, result.path.display()));
                    out.push_str(&format!("Match: {:.1}%\n", result.relevance_score * 100.0));
                    out.push_str(&result.snippet);
                    out.push_str("\n\n");
                }
            }
        }

        // Examples placeholder
        if !examples.is_empty() {
            out.push_str("Relevant examples:\n");
            for ex in &examples {
                out.push_str(ex);
                out.push_str("\n----\n");
            }
            out.push('\n');
        }

        // Add supporting templates
        if !mudlib.is_empty() {
            out.push_str(mudlib);
            out.push_str("\n\n");
        }
        if !efuns.is_empty() {
            out.push_str(efuns);
            out.push_str("\n\n");
        }
        if !refs.is_empty() {
            out.push_str(refs);
            out.push_str("\n\n");
        }

        // User query with clear instructions
        out.push_str("User Query: ");
        out.push_str(user_query);
        out.push_str("\n\nInstructions:\n");
        out.push_str("1. Follow MudOS/FluffOS patterns from references; avoid redefining platform APIs\n");
        out.push_str("2. Keep outputs self-contained: include only what is referenced in retrieved snippets\n");
        out.push_str("3. For codegen: emit bytecode opcodes only, traverse AST recursively\n");
        out.push_str("4. For object/efun tasks: use CALL_METHOD/OP_SUPER_CALL, respect master hooks, avoid redefinitions\n");
        out.push_str("5. Return complete, compilable C code with clear headers and no placeholder TODOs\n");

        // Trim to fit max tokens (8000 tokens ~= 32k chars)
        let max_tokens = 8000usize;
        let mut final_text = out.clone();
        let tokens = Self::estimate_tokens(&final_text);
        if tokens > max_tokens {
            // Order of trimming: examples first, then supporting templates, keep driver + references + query
            let mut reduced = String::new();
            reduced.push_str("You are LPC Driver Architect. Use provided APIs, no redefinitions.\n\n");
            reduced.push_str(driver);
            reduced.push_str("\n\n");
            
            // Include specialized references only
            if let Some(ref index) = self.reference_index {
                if let Ok(search_results) = index.search_with_scoring(user_query, 5) {
                    if !search_results.is_empty() {
                        reduced.push_str("Top references:\n");
                        for (idx, result) in search_results.iter().take(5).enumerate() {
                            reduced.push_str(&format!("[{}] {}\n", idx + 1, result.path.display()));
                            reduced.push_str(&result.snippet);
                            reduced.push_str("\n\n");
                        }
                    }
                }
            }
            
            reduced.push_str("User Query: ");
            reduced.push_str(user_query);
            reduced.push_str("\n\nProvide complete C code following patterns shown above.");

            // If still too large, truncate final_text to char limit
            let mut final_chars = max_tokens * 4;
            if final_chars > reduced.len() { final_chars = reduced.len(); }
            final_text = reduced.chars().take(final_chars).collect();
            // ensure user query remains intact
            if !final_text.contains(user_query) {
                final_text.push_str("\n\nUser Query: ");
                final_text.push_str(user_query);
            }
        }

        Ok(final_text)
    }

    pub fn estimate_tokens(text: &str) -> usize {
        (text.chars().count() / 4) + 1
    }

    pub fn trim_to_fit(&self, parts: Vec<&str>, max_tokens: usize) -> String {
        // Simple concatenation then naive trimming
        let s = parts.join("\n\n");
        let tokens = Self::estimate_tokens(&s);
        if tokens <= max_tokens { return s; }
        let max_chars = max_tokens * 4;
        if max_chars == 0 { return String::new(); }
        s.chars().take(max_chars).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimate_tokens() {
        // 4 chars ≈ 1 token, so 16 chars should be ~5 tokens
        let text = "This is a test string for token estimation.";
        let tokens = PromptBuilder::estimate_tokens(text);
        assert!(tokens > 0, "Token count should be positive");
    }

    #[test]
    fn test_estimate_tokens_empty() {
        let tokens = PromptBuilder::estimate_tokens("");
        assert_eq!(tokens, 1, "Empty string should return at least 1 token");
    }

    #[test]
    fn test_trim_to_fit_under_limit() {
        let pb = PromptBuilder::new_empty(PathBuf::from("/tmp"));
        let parts = vec!["hello", "world"];
        let result = pb.trim_to_fit(parts, 100);
        assert!(result.contains("hello"), "Result should contain 'hello'");
        assert!(result.contains("world"), "Result should contain 'world'");
    }

    #[test]
    fn test_trim_to_fit_over_limit() {
        let pb = PromptBuilder::new_empty(PathBuf::from("/tmp"));
        let long_text = "x".repeat(1000);
        let parts = vec![long_text.as_str()];
        let result = pb.trim_to_fit(parts, 10);
        assert!(result.len() < 1000, "Result should be shorter than input");
    }

    #[test]
    fn test_new_empty() {
        let pb = PromptBuilder::new_empty(PathBuf::from("/tmp"));
        assert!(pb.templates.is_empty(), "Empty constructor should have no templates");
    }
}

