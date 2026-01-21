use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use crate::mud_index::MudReferenceIndex;

pub struct PromptBuilder {
    templates_dir: PathBuf,
    templates: HashMap<String, String>,
    reference_index: Option<MudReferenceIndex>,
}

impl PromptBuilder {
    pub fn new(templates_dir: PathBuf) -> Result<Self, String> {
        let mut pb = PromptBuilder {
            templates_dir: templates_dir.clone(),
            templates: HashMap::new(),
            reference_index: None,
        };

        // Attempt to load known templates; missing files become empty strings
        let names = [
            "driver_context.txt",
            "mudlib_context.txt",
            "efuns_context.txt",
            "reference_sources.txt",
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
            templates_dir,
            templates: HashMap::new(),
            reference_index: None,
        }
    }

    pub fn with_reference_index(mut self, index: MudReferenceIndex) -> Self {
        self.reference_index = Some(index);
        self
    }

    pub fn build_prompt(&self, user_query: &str, _model: &str, examples: Vec<String>) -> Result<String, String> {
        // System header
        let mut out = String::new();
        out.push_str("You are an LPC MUD development assistant.\n\n");

        // Core template: driver_context preferred
        let driver = self.templates.get("driver_context.txt").map(|s| s.as_str()).unwrap_or("");
        let mudlib = self.templates.get("mudlib_context.txt").map(|s| s.as_str()).unwrap_or("");
        let efuns = self.templates.get("efuns_context.txt").map(|s| s.as_str()).unwrap_or("");
        let refs = self.templates.get("reference_sources.txt").map(|s| s.as_str()).unwrap_or("");

        out.push_str(driver);
        out.push_str("\n\n");

        // Search for relevant documentation
        if let Some(ref index) = self.reference_index {
            if let Ok(search_results) = index.search_with_scoring(user_query, 3) {
                if !search_results.is_empty() {
                    out.push_str("Relevant documentation:\n\n");
                    for result in search_results {
                        out.push_str(&format!("From: {}\n", result.path.display()));
                        out.push_str(&format!("Relevance: {:.1}%\n", result.relevance_score * 100.0));
                        out.push_str(&result.snippet);
                        out.push_str("\n\n");
                    }
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
            out.push_str("\n");
        }

        // Add supporting templates
        out.push_str(mudlib);
        out.push_str("\n\n");
        out.push_str(efuns);
        out.push_str("\n\n");
        out.push_str(refs);
        out.push_str("\n\n");

        // User query
        out.push_str("User Query: ");
        out.push_str(user_query);
        out.push_str("\n\nProvide LPC code following the patterns shown above.");

        // Trim to fit max tokens (8000 tokens ~= 32k chars)
        let max_tokens = 8000usize;
        let mut final_text = out.clone();
        let mut tokens = Self::estimate_tokens(&final_text);
        if tokens > max_tokens {
            // Order of trimming: examples first, then supporting templates, keep driver + references + query
            let mut reduced = String::new();
            reduced.push_str("You are an LPC MUD development assistant.\n\n");
            reduced.push_str(driver);
            reduced.push_str("\n\n");
            
            // Include top reference if available
            if let Some(ref index) = self.reference_index {
                if let Ok(search_results) = index.search_with_scoring(user_query, 1) {
                    if !search_results.is_empty() {
                        let result = &search_results[0];
                        reduced.push_str("Top reference:\n");
                        reduced.push_str(&result.snippet);
                        reduced.push_str("\n\n");
                    }
                }
            }
            
            reduced.push_str("User Query: ");
            reduced.push_str(user_query);
            reduced.push_str("\n\nProvide LPC code following the patterns shown above.");

            // If still too large, truncate final_text to char limit
            let mut final_chars = (max_tokens * 4) as usize;
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

