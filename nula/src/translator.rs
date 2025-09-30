use crate::cli::print_info;

pub fn translate_code(lang: &str, code: &str) -> String {
    print_info(&format!("Translating {} code...", lang));
    let lower_lang = lang.to_lowercase();
    match lower_lang.as_str() {
        "python" => {
            code.replace("print(", "write ")
            .replace("def ", "fn ")
            .replace("if ", "if ")
            .replace("for ", "for ")
            .replace("while ", "while ")
            .replace(":", " {")
            .replace("\n", ";\n")
        }
        "javascript" => {
            code.replace("console.log(", "write ")
            .replace("function ", "fn ")
            .replace("if(", "if ")
            .replace("for(", "for ")
            .replace("while(", "while ")
        }
        "rust" => {
            code.replace("println!(", "write ")
            .replace("fn ", "fn ")
            .replace("if ", "if ")
            .replace("for ", "for ")
            .replace("while ", "while ")
        }
        _ => code.to_string(),
    }
}
