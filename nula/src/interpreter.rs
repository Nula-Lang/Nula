use crate::cli::{print_error, print_info};
use std::collections::HashMap;

pub fn interpret_ast(ast: &str) {
    print_info("Starting interpretation...");
    let mut variables: HashMap<String, i64> = HashMap::new();

    for line in ast.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('@') {
            continue; // Skip comments
        }
        if trimmed.starts_with("write") {
            let msg = trimmed.trim_start_matches("write ").trim_matches(|c| c == '"' || c == '\'');
            let processed = process_expression(msg, &variables);
            println!("{}", processed);
        } else if trimmed.starts_with("add") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() == 3 {
                let a = process_expression(parts[1], &variables).parse::<i64>().unwrap_or(0);
                let b = process_expression(parts[2], &variables).parse::<i64>().unwrap_or(0);
                println!("{}", a + b);
            }
        } else if trimmed.starts_with("var") {
            let parts: Vec<&str> = trimmed.split('=').collect();
            if parts.len() == 2 {
                let name = parts[0].trim_start_matches("var ").trim().to_string();
                let value_str = process_expression(parts[1].trim(), &variables);
                let value = value_str.parse::<i64>().unwrap_or(0);
                variables.insert(name, value);
            }
        } else if trimmed.starts_with("if") {
            // Simple if handling
            let condition = trimmed.trim_start_matches("if ").split('{').next().unwrap_or("").trim();
            if evaluate_condition(condition, &variables) {
                // Execute block (simplified, assume single stmt)
                let block = trimmed.split('{').nth(1).unwrap_or("").split('}').next().unwrap_or("").trim();
                interpret_block(block, &mut variables);
            }
        }
        // Add more: loops, functions...
    }
    print_info("Interpretation completed");
}

fn process_expression(expr: &str, vars: &HashMap<String, i64>) -> String {
    if let Some(value) = vars.get(expr) {
        value.to_string()
    } else {
        expr.to_string()
    }
}

fn evaluate_condition(cond: &str, vars: &HashMap<String, i64>) -> bool {
    // Simple equality check, e.g., "x == 5"
    let parts: Vec<&str> = cond.split("==").collect();
    if parts.len() == 2 {
        let left = process_expression(parts[0].trim(), vars).parse::<i64>().unwrap_or(0);
        let right = process_expression(parts[1].trim(), vars).parse::<i64>().unwrap_or(0);
        left == right
    } else {
        false
    }
}

fn interpret_block(block: &str, vars: &mut HashMap<String, i64>) {
    // Recursive interpret for blocks
    for stmt in block.split(';') {
        // Simplified
        let trimmed = stmt.trim();
        if !trimmed.is_empty() {
            // Call interpret_ast on stmt, but avoid recursion depth issues
            if trimmed.starts_with("write") {
                let msg = trimmed.trim_start_matches("write ").trim_matches(|c| c == '"' || c == '\'');
                println!("{}", process_expression(msg, vars));
            }
            // Etc.
        }
    }
}
