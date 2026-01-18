use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::cmp::Reverse;

use serde_json::Value;

fn count_occurrences(haystack: &str, needle: &str) -> usize {
    haystack.match_indices(needle).count()
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: search_reference_db <query> [max_results]");
        std::process::exit(2);
    }
    let query = args[1].to_lowercase();
    let max_results: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(10);

    let cwd = env::current_dir()?;
    let jsonl = cwd.join("mud-references").join("index.jsonl");
    if !jsonl.exists() {
        eprintln!("index.jsonl not found at {}", jsonl.display());
        std::process::exit(2);
    }

    let f = File::open(&jsonl)?;
    let reader = BufReader::new(f);

    let mut hits: Vec<(usize, String, i64, String)> = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() { continue; }
        if let Ok(v) = serde_json::from_str::<Value>(&line) {
            if let Some(text) = v.get("text").and_then(|x| x.as_str()) {
                let text_l = text.to_lowercase();
                let occ = count_occurrences(&text_l, &query);
                if occ > 0 {
                    let path = v.get("path").and_then(|x| x.as_str()).unwrap_or("").to_string();
                    let chunk_index = v.get("chunk_index").and_then(|x| x.as_i64()).unwrap_or(0);
                    hits.push((occ, path, chunk_index, text.trim().to_string()));
                }
            }
        }
    }

    // sort by occurrences desc
    hits.sort_by_key(|h| Reverse(h.0));

    for (i, (occ, path, chunk_index, snippet)) in hits.into_iter().take(max_results).enumerate() {
        println!("#{} score={} path={} chunk={}\n{}\n---", i+1, occ, path, chunk_index, snippet);
    }

    Ok(())
}
