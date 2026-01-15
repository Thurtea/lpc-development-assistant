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
