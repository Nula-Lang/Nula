use crate::cli::print_debug;

pub fn optimize_ast(ast: &str) -> String {
    let mut optimized = String::new();
    for line in ast.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("//") {
            continue;
        }
        if line.contains("add 0") || line.contains("+ 0") || line.contains("mul 1") || line.contains("* 1") {
            continue; // Fold add 0, mul 1
        }
        optimized.push_str(line);
        optimized.push('\n');
    }
    print_debug(&format!("Optimized AST:\n{}", optimized));
    optimized
}
