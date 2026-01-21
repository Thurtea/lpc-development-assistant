use anyhow::{Context as AnyhowContext, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct SearchResult {
    pub path: PathBuf,
    pub line_number: usize,
    pub snippet: String,
    pub relevance_score: f32,
    pub file_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSnippet {
    pub path: PathBuf,
    pub snippet: String,
}

pub struct MudReferenceIndex {
    corpus_root: PathBuf,
    term_index: HashMap<String, Vec<(PathBuf, usize)>>,
    indexed: bool,
}

impl MudReferenceIndex {
    pub fn new(corpus_root: PathBuf) -> Self {
        Self {
            corpus_root,
            term_index: HashMap::new(),
            indexed: false,
        }
    }

    pub fn open_or_build(_index_dir: &Path, corpus_root: &Path) -> Result<Self> {
        std::fs::create_dir_all(_index_dir)
            .with_context(|| format!("Failed to create index dir {:?}", _index_dir))?;
        Ok(Self::new(corpus_root.to_path_buf()))
    }

    pub fn build_index(&mut self) -> Result<usize> {
        if self.indexed {
            return Ok(self.term_index.len());
        }

        let mut indexed_files = 0;

        for entry in walkdir::WalkDir::new(&self.corpus_root)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            if !Self::is_code_file(path) {
                continue;
            }

            match std::fs::read_to_string(path) {
                Ok(content) => {
                    Self::index_file(&mut self.term_index, path, &content);
                    indexed_files += 1;
                }
                Err(_) => continue,
            }
        }

        self.indexed = true;
        Ok(indexed_files)
    }

    fn index_file(term_index: &mut HashMap<String, Vec<(PathBuf, usize)>>, path: &Path, content: &str) {
        for (line_idx, line) in content.lines().enumerate() {
            let terms = Self::extract_terms(line);
            for term in terms {
                term_index
                    .entry(term)
                    .or_insert_with(Vec::new)
                    .push((path.to_path_buf(), line_idx));
            }
        }
    }

    fn extract_terms(line: &str) -> HashSet<String> {
        let mut terms = HashSet::new();

        let words: Vec<&str> = line
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .filter(|w| !w.is_empty() && w.len() > 2)
            .collect();

        for word in words {
            terms.insert(word.to_lowercase());
        }

        terms
    }

    pub fn search_relevant_code(&self, query: &str, limit: usize) -> Result<Vec<CodeSnippet>> {
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        for entry in walkdir::WalkDir::new(&self.corpus_root)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            if !Self::is_code_file(path) {
                continue;
            }

            match std::fs::read_to_string(path) {
                Ok(content) => {
                    if content.to_lowercase().contains(&query_lower) {
                        let snippet = Self::extract_snippet(&content, &query_lower, 1024);
                        results.push(CodeSnippet {
                            path: path.to_path_buf(),
                            snippet,
                        });
                        if results.len() >= limit {
                            break;
                        }
                    }
                }
                Err(_) => continue,
            }
        }

        Ok(results)
    }

    pub fn search_with_scoring(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let query_terms: Vec<&str> = query
            .split_whitespace()
            .filter(|w| w.len() > 2)
            .collect();

        let mut candidates: Vec<(PathBuf, usize, f32)> = Vec::new();

        for entry in walkdir::WalkDir::new(&self.corpus_root)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            if !Self::is_code_file(path) {
                continue;
            }

            match std::fs::read_to_string(path) {
                Ok(content) => {
                    for (line_idx, line) in content.lines().enumerate() {
                        let score = Self::calculate_relevance(&query_terms, line);
                        if score > 0.0 {
                            candidates.push((path.to_path_buf(), line_idx, score));
                        }
                    }
                }
                Err(_) => continue,
            }
        }

        candidates.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

        let mut results = Vec::new();
        let mut seen_files = HashSet::new();

        for (path, line_idx, score) in candidates.iter().take(limit * 3) {
            if seen_files.contains(path) && results.len() >= limit {
                continue;
            }
            seen_files.insert(path.clone());

            if let Ok(content) = std::fs::read_to_string(path) {
                let lines: Vec<&str> = content.lines().collect();
                if *line_idx < lines.len() {
                    let context_start = if *line_idx > 2 { *line_idx - 2 } else { 0 };
                    let context_end = std::cmp::min(*line_idx + 3, lines.len());
                    let snippet = lines[context_start..context_end].join("\n");

                    results.push(SearchResult {
                        path: path.clone(),
                        line_number: *line_idx,
                        snippet,
                        relevance_score: *score,
                        file_type: path
                            .extension()
                            .and_then(|e| e.to_str())
                            .unwrap_or("unknown")
                            .to_string(),
                    });

                    if results.len() >= limit {
                        break;
                    }
                }
            }
        }

        Ok(results)
    }

    fn calculate_relevance(query_terms: &[&str], line: &str) -> f32 {
        let line_lower = line.to_lowercase();
        let mut matches = 0;

        for term in query_terms {
            if line_lower.contains(term) {
                matches += 1;
            }
        }

        if matches == 0 {
            return 0.0;
        }

        (matches as f32) / (query_terms.len() as f32)
    }

    pub fn refresh(&self) -> Result<()> {
        Ok(())
    }

    fn is_code_file(path: &Path) -> bool {
        match path.extension().and_then(|s| s.to_str()) {
            Some("c") | Some("h") | Some("lpc") | Some("y") | Some("txt") | Some("md") => true,
            _ => false,
        }
    }

    fn extract_snippet(content: &str, q: &str, max_len: usize) -> String {
        let lc = content.to_lowercase();
        if let Some(idx) = lc.find(q) {
            let start = if idx > max_len / 4 { idx - max_len / 4 } else { 0 };
            let end = std::cmp::min(start + max_len, content.len());
            let mut s = content[start..end].to_string();
            if end < content.len() {
                s.push_str("\n...");
            }
            return s;
        }
        if content.len() <= max_len {
            content.to_string()
        } else {
            let mut s = content[..max_len].to_string();
            s.push_str("\n...");
            s
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_code_file_valid() {
        assert!(MudReferenceIndex::is_code_file(Path::new("test.c")));
        assert!(MudReferenceIndex::is_code_file(Path::new("test.h")));
        assert!(MudReferenceIndex::is_code_file(Path::new("test.lpc")));
        assert!(MudReferenceIndex::is_code_file(Path::new("test.y")));
        assert!(MudReferenceIndex::is_code_file(Path::new("test.txt")));
    }

    #[test]
    fn test_is_code_file_invalid() {
        assert!(!MudReferenceIndex::is_code_file(Path::new("test.rs")));
        assert!(!MudReferenceIndex::is_code_file(Path::new("test.json")));
        assert!(!MudReferenceIndex::is_code_file(Path::new("test")));
    }

    #[test]
    fn test_extract_snippet_with_match() {
        let content = "This is a test string with some content";
        let snippet = MudReferenceIndex::extract_snippet(content, "test", 50);
        assert!(snippet.contains("test"), "Snippet should contain search term");
    }

    #[test]
    fn test_extract_snippet_no_match() {
        let content = "This is a test string";
        let snippet = MudReferenceIndex::extract_snippet(content, "notfound", 50);
        assert!(snippet.contains("test"), "Should return fallback snippet");
    }

    #[test]
    fn test_extract_snippet_truncation() {
        let content = "a".repeat(1000);
        let snippet = MudReferenceIndex::extract_snippet(&content, "z", 100);
        assert!(snippet.len() < 200, "Snippet should be truncated");
    }

    #[test]
    fn test_new() {
        let idx = MudReferenceIndex::new(PathBuf::from("/tmp"));
        assert_eq!(idx.corpus_root, PathBuf::from("/tmp"));
    }

    #[test]
    fn test_refresh() {
        let idx = MudReferenceIndex::new(PathBuf::from("/tmp"));
        assert!(idx.refresh().is_ok(), "Refresh should succeed");
    }
}

