use lpc_dev_assistant::driver_analyzer;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let path = args.get(1).map(|s| s.as_str()).unwrap_or(
        "E:/Work/AMLP/mud-references/merentha/merentha_fluffos_v2/fluffos-2.9-ds2.03",
    );

    match driver_analyzer::efuns_json(path) {
        Ok(list) => {
            for (name, p) in &list {
                println!("{} -> {}", name, p);
            }
            println!("Total efuns found: {}", list.len());
        }
        Err(e) => eprintln!("driver_analyzer error: {}", e),
    }
}
