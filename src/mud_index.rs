use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct MudReferenceIndex {
    corpus_root: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSnippet {
    pub path: PathBuf,
    pub snippet: String,
}

impl MudReferenceIndex {
    /// Build a lightweight fallback indexer that scans files on demand.
    pub fn open_or_build(_index_dir: &Path, corpus_root: &Path) -> Result<Self> {
        std::fs::create_dir_all(_index_dir)
            .with_context(|| format!("Failed to create index dir {:?}", _index_dir))?;
        Ok(Self {
            corpus_root: corpus_root.to_path_buf(),
        })
    }

    /// Create a new index with the given corpus root (simple fallback constructor).
    pub fn new(corpus_root: PathBuf) -> Self {
        Self { corpus_root }
    }

    pub fn refresh(&self) -> Result<()> {
        // no-op for the simple fallback
        Ok(())
    }

    /// Search the corpus by doing a simple substring match over files.
    /// This is a fallback for environments where the Tantivy index isn't available.
    pub fn search_relevant_code(&self, query: &str, limit: usize) -> Result<Vec<CodeSnippet>> {
        let mut results = Vec::new();
        let q = query.to_lowercase();

        for entry in walkdir::WalkDir::new(&self.corpus_root)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            if !Self::is_code_file(path) {
                continue;
            }
            let content = match std::fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            if content.to_lowercase().contains(&q) {
                let snippet = Self::extract_snippet(&content, &q, 1024);
                results.push(CodeSnippet {
                    path: path.to_path_buf(),
                    snippet,
                });
                if results.len() >= limit {
                    break;
                }
            }
        }

        Ok(results)
    }

    fn is_code_file(path: &Path) -> bool {
        match path.extension().and_then(|s| s.to_str()) {
            Some("c") | Some("h") | Some("lpc") | Some("y") | Some("txt") => true,
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

