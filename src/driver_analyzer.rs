use regex::Regex;
use std::env;
use std::fs;
use walkdir::WalkDir;

// Analyzer: scan files under given path for efun registrations and return (name, file) pairs
pub fn efuns_json(root: &str) -> Result<Vec<(String, String)>, String> {
    let mut results: Vec<(String, String)> = Vec::new();

    // Primary pattern: #ifdef F_FUNCTIONNAME (most reliable for FluffOS)
    let re_ifdef = Regex::new(r#"#ifdef\s+F_([A-Za-z_][A-Za-z0-9_]*)"#).map_err(|e| e.to_string())?;
    
    // Secondary patterns
    let re_fdef = Regex::new(r#"void\s+f_([A-Za-z_][A-Za-z0-9_]*)\s*\("#).map_err(|e| e.to_string())?;
    let re_add = Regex::new(r#"ADD_EFUN\s*\(\s*\"([^\"]+)\""#).map_err(|e| e.to_string())?;
    let re_array = Regex::new(r#"array_init\s*\(\s*\"([A-Za-z_][A-Za-z0-9_]*)\""#).map_err(|e| e.to_string())?;
    let re_reg = Regex::new(r#"register_efun\s*\(\s*\"([^\"]+)\""#).map_err(|e| e.to_string())?;

    let blacklist = vec![
        "if","for","while","return","void","int","float","class","mapping","object",
        "string","switch","case","default","else","do","break","continue","static","private",
        "public","protected","nomask","new","save","const","typedef","struct","union","sizeof",
        "standard","generic","all","lpc","extensions","optional","major","minor","debug",
    ].into_iter().map(|s| s.to_string()).collect::<std::collections::HashSet<String>>();

    for entry in WalkDir::new(root).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let path_s = path.display().to_string();
        let path_lower = path_s.to_lowercase();
        
        // ONLY scan src/**/*.c and src/**/*.h
        if !path_lower.contains("\\src\\") && !path_lower.contains("/src/") {
            continue;
        }
        
        match path.extension().and_then(|e| e.to_str()).map(|s| s.to_lowercase()) {
            Some(ext) if ext == "c" || ext == "h" => {},
            _ => continue,
        }
        
        // Skip known noise sources
        if path_lower.contains("lex.c") || path_lower.contains("test") || path_lower.contains("parse.c") {
            continue;
        }

        if let Ok(text) = fs::read_to_string(path) {
            for line in text.lines() {
                // PRIMARY: #ifdef F_FUNCTIONNAME (priority 1)
                if let Some(cap) = re_ifdef.captures(line) {
                    if let Some(m) = cap.get(1) {
                        let nm = m.as_str().to_lowercase();
                        if !blacklist.contains(&nm) && !nm.is_empty() {
                            results.push((nm, path_s.replace("\\", "/")));
                        }
                    }
                }
                // SECONDARY: void f_functionname (priority 2)
                if let Some(cap) = re_fdef.captures(line) {
                    if let Some(m) = cap.get(1) {
                        let nm = m.as_str().to_string();
                        if !blacklist.contains(&nm) && !nm.is_empty() {
                            results.push((nm, path_s.replace("\\", "/")));
                        }
                    }
                }
                // ADD_EFUN macro
                if let Some(cap) = re_add.captures(line) {
                    if let Some(m) = cap.get(1) {
                        let nm = m.as_str().to_string();
                        if !blacklist.contains(&nm) && !nm.is_empty() {
                            results.push((nm, path_s.replace("\\", "/")));
                        }
                    }
                }
                // register_efun calls
                if let Some(cap) = re_reg.captures(line) {
                    if let Some(m) = cap.get(1) {
                        let nm = m.as_str().to_string();
                        if !blacklist.contains(&nm) && !nm.is_empty() {
                            results.push((nm, path_s.replace("\\", "/")));
                        }
                    }
                }
                // array_init arrays
                if let Some(cap) = re_array.captures(line) {
                    if let Some(m) = cap.get(1) {
                        let nm = m.as_str().to_string();
                        if !blacklist.contains(&nm) && !nm.is_empty() {
                            results.push((nm, path_s.replace("\\", "/")));
                        }
                    }
                }
            }
        }
    }

    // Deduplicate by name only (keep first occurrence of each efun)
    results.sort_by(|a, b| a.0.cmp(&b.0));
    results.dedup_by(|a, b| a.0 == b.0);
    results.sort_by(|a, b| a.0.cmp(&b.0));

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

// Unused helper function removed
