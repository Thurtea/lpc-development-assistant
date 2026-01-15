use regex::Regex;
use std::env;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

// Analyzer: scan files under given path for efun registrations and return (name, file:line) pairs
pub fn efuns_json(root: &str) -> Result<Vec<(String, String)>, String> {
    let mut results: Vec<(String, String)> = Vec::new();

    let re_add = Regex::new(r#"add_efun\(\s*\"([^\"]+)\""#).map_err(|e| e.to_string())?;
    let re_array = Regex::new(r#"\{\s*\"([^\"]+)\"\s*,\s*[A-Za-z0-9_]+"#).map_err(|e| e.to_string())?;
    let re_reg = Regex::new(r#"register_efun\(\s*\"([^\"]+)\""#).map_err(|e| e.to_string())?;
    let re_fdef = Regex::new(r#"\bf_([a-z0-9_]+)\s*\("#).map_err(|e| e.to_string())?;
    let re_ifdef = Regex::new(r#"#\s*ifdef\s+F_([A-Z0-9_]+)"#).map_err(|e| e.to_string())?;

    let blacklist = vec![
        "if","for","while","return","void","int","float","class","mapping","object",
        "string","switch","case","default","else","do","break","continue","static","private",
        "public","protected","nomask","new","save","const","typedef","struct","union","sizeof",
    ].into_iter().map(|s| s.to_string()).collect::<std::collections::HashSet<String>>();

    for entry in WalkDir::new(root).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        // focus on C/C++ source files and common efun tables
        let consider = match path.extension().and_then(|e| e.to_str()).map(|s| s.to_lowercase()) {
            Some(ext) if ext == "c" || ext == "cc" || ext == "cpp" || ext == "h" => true,
            _ => {
                let n = name.to_lowercase();
                n.starts_with("efun_") || n.contains("efun") || n.contains("funtab") || n.contains("compat")
            }
        };

        if !consider {
            continue;
        }

        let path_s = path.display().to_string().to_lowercase();
        // Skip lexers, parser tables, and tests which contain many unrelated symbol lists
        if path_s.contains("lex.c") || path_s.contains("/packages/") || path_s.contains("\\packages\\") || path_s.contains("testsuite") {
            continue;
        }

        if let Ok(text) = fs::read_to_string(path) {
            for (idx, line) in text.lines().enumerate() {
                if let Some(cap) = re_add.captures(line) {
                    if let Some(m) = cap.get(1) {
                        let nm = m.as_str().to_string();
                        if blacklist.contains(&nm) { continue; }
                        results.push((nm, format!("{}:{}", path.display(), idx + 1)));
                    }
                }
                if let Some(cap) = re_array.captures(line) {
                    if let Some(m) = cap.get(1) {
                        let nm = m.as_str().to_string();
                        if blacklist.contains(&nm) { continue; }
                        results.push((nm, format!("{}:{}", path.display(), idx + 1)));
                    }
                }
                if let Some(cap) = re_reg.captures(line) {
                    if let Some(m) = cap.get(1) {
                        let nm = m.as_str().to_string();
                        if blacklist.contains(&nm) { continue; }
                        results.push((nm, format!("{}:{}", path.display(), idx + 1)));
                    }
                }
                if let Some(cap) = re_fdef.captures(line) {
                    if let Some(m) = cap.get(1) {
                        let nm = m.as_str().to_string();
                        if blacklist.contains(&nm) { continue; }
                        results.push((nm, format!("{}:{}", path.display(), idx + 1)));
                    }
                }
                if let Some(cap) = re_ifdef.captures(line) {
                    if let Some(m) = cap.get(1) {
                        let nm = m.as_str().to_lowercase();
                        if blacklist.contains(&nm) { continue; }
                        results.push((nm, format!("{}:{}", path.display(), idx + 1)));
                    }
                }
            }
        }
    }

    // Deduplicate and sort by name
    results.sort_by(|a, b| a.0.cmp(&b.0));
    results.dedup_by(|a, b| a.0 == b.0 && a.1 == b.1);

    Ok(results)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let root = args.get(1).map(|s| s.as_str()).unwrap_or("E:/Work/AMLP/mud-references/merentha/merentha_fluffos_v2/fluffos-2.9-ds2.03");
    match efuns_json(root) {
        Ok(v) => {
            // Print simple JSON-like output
            print!("{{");
            print!("\"efuns\": [");
            let mut first = true;
            for (name, path) in v {
                if !first { print!(","); } first = false;
                print!("{{\"name\":\"{}\",\"path\":\"{}\"}}", name, path.replace('\\', "/"));
            }
            print!("]}}");
        }
        Err(e) => eprintln!("driver_analyzer error: {}", e),
    }
}

fn visit(path: &Path, re: &Regex, out: &mut Vec<(String, String)>) -> Result<(), std::io::Error> {
    if path.is_file() {
        if let Ok(text) = fs::read_to_string(path) {
            for cap in re.captures_iter(&text) {
                if let Some(name) = cap.get(1) {
                    out.push((name.as_str().to_string(), path.display().to_string()));
                }
            }
        }
    } else if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.filter_map(Result::ok) {
            let p = entry.path();
            let _ = visit(&p, re, out);
        }
    }
    Ok(())
}
