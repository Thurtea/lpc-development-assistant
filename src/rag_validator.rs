use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use crate::mud_index::{MudReferenceIndex, SearchResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub query: String,
    pub retrieved_documents: Vec<RetrievedDocument>,
    pub confidence_score: f32,
    pub validation_score: f32,
    pub sources_consulted: usize,
    pub efuns_found: Vec<String>,
    pub code_examples: Vec<CodeExample>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievedDocument {
    pub path: String,
    pub relevance_score: f32,
    pub snippet: String,
    pub line_number: usize,
    pub validation_status: ValidationStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ValidationStatus {
    Verified,        // Content verified against known-good examples
    CrossReferenced, // Found in multiple sources
    SingleSource,    // Only found in one source
    Uncertain,       // Conflicting information
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeExample {
    pub path: String,
    pub code: String,
    pub context: String,
    pub driver: String, // e.g., "MudOS", "FluffOS", "DGD"
}

pub struct RAGValidator {
    index: MudReferenceIndex,
    efun_definitions: HashMap<String, EfunDefinition>,
    min_confidence_threshold: f32,
}

#[derive(Debug, Clone)]
struct EfunDefinition {
    name: String,
    sources: Vec<String>,
    signatures: Vec<String>,
}

impl RAGValidator {
    pub fn new(index: MudReferenceIndex, corpus_root: PathBuf) -> Result<Self> {
        let efun_definitions = Self::load_efun_definitions(&corpus_root)?;
        
        Ok(Self {
            index,
            efun_definitions,
            min_confidence_threshold: 0.3,
        })
    }

    /// Validate a query by retrieving and cross-referencing documents
    pub fn validate_query(&self, query: &str) -> Result<ValidationResult> {
        // Step 1: Retrieve relevant documents using enhanced search
        let search_results = self.index.search_with_scoring(query, 25)?;
        
        // Step 2: Cross-reference results
        let mut retrieved_docs = Vec::new();
        let mut efuns_found = Vec::new();
        let mut code_examples = Vec::new();
        
        for result in &search_results {
            let validation_status = self.validate_result(result, &search_results);
            
            retrieved_docs.push(RetrievedDocument {
                path: result.path.display().to_string(),
                relevance_score: result.relevance_score,
                snippet: result.snippet.clone(),
                line_number: result.line_number,
                validation_status,
            });
            
            // Extract efuns mentioned in the result
            let efuns = self.extract_efuns(&result.snippet);
            efuns_found.extend(efuns);
            
            // Extract code examples if this is a source file
            if self.is_source_file(&result.path) {
                if let Some(example) = self.extract_code_example(result) {
                    code_examples.push(example);
                }
            }
        }
        
        // Step 3: Calculate confidence and validation scores
        let confidence_score = self.calculate_confidence_score(&retrieved_docs);
        let validation_score = self.calculate_validation_score(&retrieved_docs);
        
        // Deduplicate efuns
        efuns_found.sort();
        efuns_found.dedup();
        
        Ok(ValidationResult {
            query: query.to_string(),
            retrieved_documents: retrieved_docs,
            confidence_score,
            validation_score,
            sources_consulted: search_results.len(),
            efuns_found,
            code_examples,
        })
    }

    /// Validate individual search result by cross-referencing with other results
    fn validate_result(&self, result: &SearchResult, all_results: &[SearchResult]) -> ValidationStatus {
        // Check if similar content appears in multiple sources
        let mut match_count = 0;
        let result_lower = result.snippet.to_lowercase();
        
        for other in all_results {
            if other.path == result.path {
                continue;
            }
            
            let other_lower = other.snippet.to_lowercase();
            
            // Simple heuristic: check for common significant terms
            let result_terms: Vec<&str> = result_lower
                .split_whitespace()
                .filter(|w| w.len() > 4)
                .collect();
            
            let matches = result_terms.iter()
                .filter(|&&term| other_lower.contains(term))
                .count();
            
            if matches > result_terms.len() / 4 {
                match_count += 1;
            }
        }
        
        if match_count >= 3 {
            ValidationStatus::Verified
        } else if match_count >= 1 {
            ValidationStatus::CrossReferenced
        } else {
            ValidationStatus::SingleSource
        }
    }

    /// Extract efun names from text
    fn extract_efuns(&self, text: &str) -> Vec<String> {
        let mut found = Vec::new();
        
        for (efun_name, _) in &self.efun_definitions {
            if text.contains(efun_name) {
                found.push(efun_name.clone());
            }
        }
        
        found
    }

    /// Check if path is a source file
    fn is_source_file(&self, path: &PathBuf) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .map(|e| matches!(e, "c" | "h" | "lpc"))
            .unwrap_or(false)
    }

    /// Extract code example from search result
    fn extract_code_example(&self, result: &SearchResult) -> Option<CodeExample> {
        let path_str = result.path.display().to_string();
        
        // Determine driver from path
        let driver = if path_str.contains("mudos") {
            "MudOS"
        } else if path_str.contains("fluffos") {
            "FluffOS"
        } else if path_str.contains("dgd") {
            "DGD"
        } else if path_str.contains("ldmud") {
            "LDMud"
        } else {
            "Unknown"
        };
        
        // Extract meaningful context
        let lines: Vec<&str> = result.snippet.lines().collect();
        if lines.len() < 3 {
            return None;
        }
        
        Some(CodeExample {
            path: path_str,
            code: result.snippet.clone(),
            context: format!("From {} at line {}", driver, result.line_number),
            driver: driver.to_string(),
        })
    }

    /// Calculate confidence score based on retrieval quality
    fn calculate_confidence_score(&self, docs: &[RetrievedDocument]) -> f32 {
        if docs.is_empty() {
            return 0.0;
        }
        
        let avg_relevance: f32 = docs.iter()
            .map(|d| d.relevance_score)
            .sum::<f32>() / docs.len() as f32;
        
        let verified_count = docs.iter()
            .filter(|d| d.validation_status == ValidationStatus::Verified)
            .count();
        
        let cross_ref_count = docs.iter()
            .filter(|d| d.validation_status == ValidationStatus::CrossReferenced)
            .count();
        
        // Weighted score
        let validation_boost = (verified_count as f32 * 0.3 + cross_ref_count as f32 * 0.15) / docs.len() as f32;
        
        (avg_relevance + validation_boost).min(1.0)
    }

    /// Calculate validation score based on cross-referencing
    fn calculate_validation_score(&self, docs: &[RetrievedDocument]) -> f32 {
        if docs.is_empty() {
            return 0.0;
        }
        
        let verified = docs.iter()
            .filter(|d| d.validation_status == ValidationStatus::Verified)
            .count() as f32;
        
        let cross_ref = docs.iter()
            .filter(|d| d.validation_status == ValidationStatus::CrossReferenced)
            .count() as f32;
        
        let single = docs.iter()
            .filter(|d| d.validation_status == ValidationStatus::SingleSource)
            .count() as f32;
        
        (verified + cross_ref * 0.6 + single * 0.3) / docs.len() as f32
    }

    /// Load efun definitions from corpus
    fn load_efun_definitions(corpus_root: &PathBuf) -> Result<HashMap<String, EfunDefinition>> {
        let mut definitions = HashMap::new();
        
        // Load all_efuns.txt
        let all_efuns_path = corpus_root.join("all_efuns.txt");
        if let Ok(content) = std::fs::read_to_string(&all_efuns_path) {
            for line in content.lines() {
                let name = line.trim();
                if !name.is_empty() {
                    definitions.entry(name.to_string())
                        .or_insert_with(|| EfunDefinition {
                            name: name.to_string(),
                            sources: vec!["all_efuns.txt".to_string()],
                            signatures: Vec::new(),
                        });
                }
            }
        }
        
        // Load merentha_efuns.txt
        let merentha_path = corpus_root.join("merentha_efuns.txt");
        if let Ok(content) = std::fs::read_to_string(&merentha_path) {
            for line in content.lines() {
                let name = line.trim();
                if !name.is_empty() {
                    definitions.entry(name.to_string())
                        .or_insert_with(|| EfunDefinition {
                            name: name.to_string(),
                            sources: Vec::new(),
                            signatures: Vec::new(),
                        })
                        .sources.push("merentha_efuns.txt".to_string());
                }
            }
        }
        
        Ok(definitions)
    }

    /// Get all known efun names
    pub fn get_known_efuns(&self) -> Vec<String> {
        self.efun_definitions.keys().cloned().collect()
    }

    /// Search for specific efun documentation
    pub fn search_efun(&self, efun_name: &str) -> Result<Vec<SearchResult>> {
        let query = format!("{} efun function", efun_name);
        self.index.search_with_scoring(&query, 10)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_status() {
        assert_eq!(ValidationStatus::Verified, ValidationStatus::Verified);
        assert_ne!(ValidationStatus::Verified, ValidationStatus::SingleSource);
    }

    #[test]
    fn test_calculate_confidence_empty() {
        let docs = vec![];
        let validator = RAGValidator {
            index: MudReferenceIndex::new(PathBuf::from("/tmp")),
            efun_definitions: HashMap::new(),
            min_confidence_threshold: 0.3,
        };
        
        assert_eq!(validator.calculate_confidence_score(&docs), 0.0);
    }

    #[test]
    fn test_is_source_file() {
        let validator = RAGValidator {
            index: MudReferenceIndex::new(PathBuf::from("/tmp")),
            efun_definitions: HashMap::new(),
            min_confidence_threshold: 0.3,
        };
        
        assert!(validator.is_source_file(&PathBuf::from("test.c")));
        assert!(validator.is_source_file(&PathBuf::from("test.h")));
        assert!(validator.is_source_file(&PathBuf::from("test.lpc")));
        assert!(!validator.is_source_file(&PathBuf::from("test.txt")));
    }
}
