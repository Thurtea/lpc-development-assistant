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
                        return Ok(Some((name.to_string(), v.as_str().unwrap().to_string())));
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
