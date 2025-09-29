pub fn optimize_ast(ast: &str) -> String {
    let mut optimized = String::new();
    for line in ast.lines() {
        // Simple optimizations: remove redundant lines, constant folding
        if line.contains("add 0") || line.contains("+ 0") {
            continue; // Fold add 0
        }
        optimized.push_str(line);
        optimized.push('\n');
    }
    optimized
}
