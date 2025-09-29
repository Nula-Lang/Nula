use anyhow::Result;
use std::fs;

pub fn translate_from(code: &str, from_lang: &str) -> Result<String> {
    match from_lang {
        "python" => {
            // Proste mapowanie: print -> write
            let translated = code.replace("print(", "write(");
            Ok(translated)
        }
        _ => Ok(code.to_string()),
    }
}

// Użyj w parser: jeśli kod ma # = lang = {code}, parse i translate
