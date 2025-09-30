use crate::cli::print_info;

pub fn translate_code(lang: &str, code: &str) -> String {
    print_info(&format!("Translating {} code...", lang));
    let lower_lang = lang.to_lowercase();
    match lower_lang.as_str() {
        "python" => translate_python(code),
        "javascript" => translate_javascript(code),
        "rust" => translate_rust(code),
        "c" => translate_c(code),
        "java" => translate_java(code),
        "go" => translate_go(code),
        _ => code.to_string(),
    }
}

fn translate_python(code: &str) -> String {
    code.replace("print(", "write (")
    .replace("def ", "fn ")
    .replace(":", " {")
    .replace("\n    ", "\n")
    .replace("self", "_self")
    .replace("class ", "struct ")
}

fn translate_javascript(code: &str) -> String {
    code.replace("console.log(", "write (")
    .replace("function ", "fn ")
    .replace("let ", "var ")
    .replace("const ", "var ")
    .replace("class ", "struct ")
    .replace("=>", "=>")
}

fn translate_rust(code: &str) -> String {
    code.replace("println!(", "write (")
    .replace("fn ", "fn ")
    .replace("let ", "var ")
    .replace("mut ", "")
    .replace("struct ", "struct ")
    .replace("&", "")
    .replace("::", ".")
}

fn translate_c(code: &str) -> String {
    code.replace("printf(", "write (")
    .replace("int main(", "fn main(")
    .replace(";", ";")
    .replace("#include ", "import ")
}

fn translate_java(code: &str) -> String {
    code.replace("System.out.println(", "write (")
    .replace("public static void ", "fn ")
    .replace("class ", "struct ")
    .replace("int ", "var ")
}

fn translate_go(code: &str) -> String {
    code.replace("fmt.Println(", "write (")
    .replace("func ", "fn ")
    .replace("var ", "var ")
    .replace("package ", "# ")
}
