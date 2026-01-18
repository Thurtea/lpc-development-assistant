use std::env;
use std::fs::File;
use std::io::{self, Read, Write, BufWriter};
use std::path::{Path, PathBuf};

use walkdir::WalkDir;
use serde_json::json;

fn is_text_file(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
        matches!(ext.to_lowercase().as_str(), "c" | "h" | "txt" | "md" | "rs" | "json" | "py" | "yaml" | "yml")
    } else {
        false
    }
}

fn chunk_text(s: &str, max_chars: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current = String::new();
    for word in s.split_whitespace() {
        if !current.is_empty() {
            if current.len() + 1 + word.len() > max_chars {
                chunks.push(current);
                current = word.to_string();
                continue;
            } else {
                current.push(' ');
                current.push_str(word);
            }
        } else {
            current.push_str(word);
        }
    }
    if !current.is_empty() {
        chunks.push(current);
    }
    chunks
}

fn process_path(base: &Path, path: &Path, writer: &mut BufWriter<File>) -> io::Result<usize> {
    let mut f = File::open(path)?;
    let mut buf = String::new();
    f.read_to_string(&mut buf)?;
    let rel = path.strip_prefix(base).unwrap_or(path).to_string_lossy().to_string();
    let chunks = chunk_text(&buf, 1000);
    for (i, chunk) in chunks.into_iter().enumerate() {
        let obj = json!({
            "id": format!("{}#{}", rel, i),
            "path": rel,
            "chunk_index": i,
            "text": chunk,
        });
        writeln!(writer, "{}", obj.to_string())?;
    }
    Ok(1)
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    let cwd = env::current_dir()?;
    let base = cwd.join("mud-references");
    if !base.exists() {
        eprintln!("mud-references directory not found at {}", base.display());
        std::process::exit(2);
    }

    let targets: Vec<PathBuf> = if args.is_empty() {
        // index all top-level dirs inside mud-references
        WalkDir::new(&base).max_depth(1).into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_dir())
            .map(|e| e.path().to_path_buf())
            .filter(|p| p != &base)
            .collect()
    } else {
        args.iter().map(|a| base.join(a)).collect()
    };

    let out_path = base.join("index.jsonl");
    let out_file = File::create(&out_path)?;
    let mut writer = BufWriter::new(out_file);

    println!("Indexing {} targets into {}", targets.len(), out_path.display());
    let mut files_indexed = 0usize;

    for t in targets {
        if !t.exists() { eprintln!("Skipping missing target {}", t.display()); continue; }
        for entry in WalkDir::new(&t).into_iter().filter_map(|e| e.ok()) {
            let p = entry.path();
            if entry.file_type().is_file() && is_text_file(p) {
                match process_path(&base, p, &mut writer) {
                    Ok(_) => files_indexed += 1,
                    Err(e) => eprintln!("Failed to index {}: {}", p.display(), e),
                }
            }
        }
    }

    writer.flush()?;
    println!("Indexing complete. Files indexed: {}. Output: {}", files_indexed, out_path.display());
    Ok(())
}
