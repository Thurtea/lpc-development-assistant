use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
// PathBuf not needed explicitly

use serde_json::Value;
use tantivy::directory::MmapDirectory;
use tantivy::schema::*;
use tantivy::Index;
use tantivy::doc;

fn main() -> io::Result<()> {
    let cwd = env::current_dir()?;
    let base = cwd.join("mud-references");
    let jsonl = base.join("index.jsonl");
    if !jsonl.exists() {
        eprintln!("index.jsonl not found at {}", jsonl.display());
        std::process::exit(2);
    }

    println!("Building Tantivy index from {}", jsonl.display());

    // Schema
    let mut schema_builder = Schema::builder();
    let id_field = schema_builder.add_text_field("id", STRING | STORED);
    let path_field = schema_builder.add_text_field("path", STRING | STORED);
    let chunk_field = schema_builder.add_i64_field("chunk_index", STORED);
    let text_field = schema_builder.add_text_field("text", TEXT | STORED);
    let schema = schema_builder.build();

    let index_path = base.join("index_tantivy");
    std::fs::create_dir_all(&index_path)?;
    let mmap_dir = MmapDirectory::open(&index_path).map_err(io::Error::other)?;
    let index = Index::open_or_create(mmap_dir, schema.clone()).map_err(io::Error::other)?;

    let mut writer = index.writer(50_000_000).map_err(io::Error::other)?;

    let f = File::open(&jsonl)?;
    let reader = BufReader::new(f);
    let mut count: usize = 0;

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() { continue; }
        match serde_json::from_str::<Value>(&line) {
            Ok(v) => {
                // Use doc! macro to create document safely
                let mut doc = doc!(
                    id_field => "",
                    path_field => "",
                    chunk_field => 0i64,
                    text_field => ""
                );
                
                if let Some(id) = v.get("id").and_then(|x| x.as_str()) {
                    doc.add_text(id_field, id);
                }
                if let Some(path) = v.get("path").and_then(|x| x.as_str()) {
                    doc.add_text(path_field, path);
                }
                if let Some(ci) = v.get("chunk_index").and_then(|x| x.as_i64()) {
                    doc.add_i64(chunk_field, ci);
                }
                if let Some(text) = v.get("text").and_then(|x| x.as_str()) {
                    doc.add_text(text_field, text);
                }
                let _ = writer.add_document(doc);
                count += 1;
                if count.is_multiple_of(5000) {
                    println!("Indexed {} documents...", count);
                }
            }
            Err(e) => eprintln!("Failed to parse line as JSON: {}", e),
        }
    }

    writer.commit().map_err(io::Error::other)?;
    println!("Tantivy index built: {} documents indexed", count);
    Ok(())
}

