use crate::cli::print_info;

pub fn translate_code(lang: &str, code: &str) -> String {
    print_info(&format!("Translating {} code...", lang));
    let lower_lang = lang.to_lowercase();

    let replacements = match lower_lang.as_str() {
        "python" => vec![
            ("print(", "write ("),
            ("def ", "fn "),
            (":", " {"),
            ("\n    ", "\n"),
            ("self", "_self"),
            ("class ", "struct "),
        ],
        "javascript" => vec![
            ("console.log(", "write ("),
            ("function ", "fn "),
            ("let ", "var "),
            ("const ", "var "),
            ("class ", "struct "),
            ("=>", "=>"),
        ],
        "rust" => vec![
            ("println!(", "write ("),
            ("let ", "var "),
            ("mut ", ""),
            ("&", ""),
            ("::", "."),
        ],
        "c" => vec![
            ("printf(", "write ("),
            ("int main(", "fn main("),
            ("#include ", "import "),
        ],
        "java" => vec![
            ("System.out.println(", "write ("),
            ("public static void ", "fn "),
            ("class ", "struct "),
            ("int ", "var "),
        ],
        "go" => vec![
            ("fmt.Println(", "write ("),
            ("func ", "fn "),
            ("package ", "# "),
        ],
        _ => return code.to_string(),
    };

    // Apply all replacements in sequence
    let mut result = code.to_string();
    for (from, to) in replacements {
        result = result.replace(from, to);
    }
    result
}
