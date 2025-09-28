se anyhow::Result;
use regex::Regex;

pub fn format_code(code: &str) -> Result<String> {
    // Prosty formatter: dodaj spacje, indent
    let mut formatted = String::new();
    let indent_re = Regex::new(r"\{")?;  // Na razie stub
    let mut indent = 0;
    for line in code.lines() {
        let trimmed = line.trim();
        if trimmed.ends_with('}') {
            indent -= 1;
        }
        formatted.push_str(&"    ".repeat(indent));
        formatted.push_str(trimmed);
        formatted.push('\n');
        if trimmed.ends_with('{') {
            indent += 1;
        }
    }
    Ok(formatted)
}
