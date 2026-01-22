use std::{fs, path::Path};

struct BenchmarkResult {
    name: &'static str,
    score: f32,
    pass: bool,
}

const PASS_THRESHOLD: f32 = 70.0;
const TARGET_AVERAGE: f32 = 85.0;

fn main() {
    let tests: Vec<(&'static str, fn() -> f32)> = vec![
        ("Codegen core opcodes", test_codegen_core),
        ("Codegen control flow", test_codegen_control_flow),
        ("Codegen VM patterns", test_codegen_vm_patterns),
        ("Object CALL_METHOD", test_object_call_method),
        ("Object ref + cleanup", test_object_ref_cleanup),
        ("Object env + move", test_object_env_move),
        ("Efun catalog present", test_efun_catalog),
        ("Efun context template", test_efun_context),
        ("UI validation widgets", test_ui_validation),
        ("RAG top15 retrieval", test_rag_top15),
    ];

    let mut results = Vec::new();
    for (name, func) in tests {
        let score = func().clamp(0.0, 100.0);
        let pass = score >= PASS_THRESHOLD;
        results.push(BenchmarkResult { name, score, pass });
    }

    println!("{:<3} | {:<40} | {:<4} | {:>6}", "#", "Test", "Res", "Score");
    println!("{}", "-".repeat(65));

    let mut total = 0.0;
    for (idx, r) in results.iter().enumerate() {
        total += r.score;
        let status = if r.pass { "PASS" } else { "FAIL" };
        println!("{:<3} | {:<40} | {:<4} | {:>5.1}%", idx + 1, r.name, status, r.score);
    }

    let avg = total / results.len() as f32;
    println!("{}", "-".repeat(65));
    println!("Avg | {:<40} | {:<4} | {:>5.1}%", "--", if avg >= TARGET_AVERAGE { "PASS" } else { "FAIL" }, avg);

    if avg < TARGET_AVERAGE {
        std::process::exit(1);
    }
}

fn score_patterns(path: &str, patterns: &[&str]) -> f32 {
    let text = match fs::read_to_string(path) {
        Ok(s) => s.to_lowercase(),
        Err(_) => return 0.0,
    };

    if patterns.is_empty() {
        return 100.0;
    }

    let mut hits = 0;
    for p in patterns {
        if text.contains(&p.to_lowercase()) {
            hits += 1;
        }
    }
    (hits as f32 / patterns.len() as f32) * 100.0
}

fn score_line_count(path: &str, min_lines: usize) -> f32 {
    let text = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(_) => return 0.0,
    };
    let lines = text.lines().count();
    if lines == 0 {
        return 0.0;
    }
    let ratio = lines as f32 / min_lines as f32;
    (ratio.min(1.0)) * 100.0
}

fn test_codegen_core() -> f32 {
    score_patterns(
        "templates/driver_codegen.txt",
        &["opcode", "op_push_int", "codegen_emit_op", "codegen_emit_arg", "codegen_expr"],
    )
}

fn test_codegen_control_flow() -> f32 {
    score_patterns(
        "templates/driver_codegen.txt",
        &["op_jmp", "op_jmp_if_false", "op_return", "op_call_func"],
    )
}

fn test_codegen_vm_patterns() -> f32 {
    score_patterns(
        "templates/driver_codegen.txt",
        &["vm execution loop", "bytecode"],
    )
}

fn test_object_call_method() -> f32 {
    score_patterns(
        "templates/object_system.txt",
        &["call_method", "op_call_method", "op_super_call", "inherit", "call_other"],
    )
}

fn test_object_ref_cleanup() -> f32 {
    score_patterns(
        "templates/object_system.txt",
        &["refcount", "add_ref", "free_object", "clean_up", "heart_beat"],
    )
}

fn test_object_env_move() -> f32 {
    score_patterns(
        "templates/object_system.txt",
        &["environment", "move_object", "inventory", "deep_inventory"],
    )
}

fn test_efun_catalog() -> f32 {
    score_line_count("mud-references/all_efuns.txt", 200)
}

fn test_efun_context() -> f32 {
    let content = score_patterns(
        "templates/efuns_context.txt",
        &["efun", "call_out"],
    );
    let size_score = score_line_count("templates/efuns_context.txt", 10);
    (content * 0.5) + (size_score * 0.5)
}

fn test_ui_validation() -> f32 {
    score_patterns(
        "ui/index.html",
        &["header-badge", "quality-score", "score-pill", "issue-list", "context-list"],
    )
}

fn test_rag_top15() -> f32 {
    score_patterns(
        "src/prompt_builder.rs",
        &["Top 15", "CRITICAL REFERENCES", "object_system", "call_method"],
    )
}
