use std::fs;
use serde_json::Value;

pub fn lookup_efun(name: &str) -> Result<Option<(String,String)>, String> {
    // try json file
    if let Ok(s) = fs::read_to_string("gen\\all_efuns_final.json") {
        match serde_json::from_str::<Value>(&s) {
            Ok(Value::Object(map)) => {
                if let Some(v) = map.get(name) {
                    // if it's a string, treat as docs
                    if v.is_string() {
                        if let Some(s) = v.as_str() {
                            return Ok(Some((name.to_string(), s.to_string())));
                        } else {
                            // unexpected: treat value's stringified form as docs
                            return Ok(Some((name.to_string(), v.to_string())));
                        }
                    }
                    if let Some(sig) = v.get("signature") {
                        let sigs = sig.as_str().unwrap_or("").to_string();
                        let docs = v.get("docs").and_then(|d| d.as_str()).unwrap_or("").to_string();
                        let combined = if docs.is_empty() { sigs.clone() } else { format!("{}\n\n{}", sigs, docs) };
                        return Ok(Some((name.to_string(), combined)));
                    }
                    // fallback to stringified value
                    return Ok(Some((name.to_string(), v.to_string())));
                }
            }
            Ok(_) => {}
            Err(e) => return Err(format!("Failed to parse JSON: {}", e)),
        }
    }

    // fallback: try txt list
    if let Ok(s) = fs::read_to_string("gen\\all_efuns.txt") {
        for line in s.lines() {
            let l = line.trim();
            if l == name {
                return Ok(Some((name.to_string(), "No signature available".to_string())));
            }
        }
    }

    Ok(None)
}
fn main() {
    // Simple CLI interface for efun lookup
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <efun_name>", args[0]);
        std::process::exit(1);
    }

    let efun_name = &args[1];
    match lookup_efun(efun_name) {
        Ok(Some((name, docs))) => {
            println!("Efun: {}\n{}", name, docs);
        }
        Ok(None) => {
            eprintln!("Efun '{}' not found", efun_name);
            std::process::exit(2);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(3);
        }
    }
}