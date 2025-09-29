use crate::cli::print_error;
use std::fs;
use std::path::Path;
use toml::Value;

pub fn load_config() -> Result<Value, String> {
    let config_path = Path::new("nula.toml");
    if !config_path.exists() {
        return Err("nula.toml not found".to_string());
    }
    let content = match fs::read_to_string(config_path) {
        Ok(c) => c,
        Err(e) => return Err(format!("Failed to read nula.toml: {}", e)),
    };
    match content.parse::<Value>() {
        Ok(v) => Ok(v),
        Err(e) => Err(format!("Invalid TOML: {}", e)),
    }
}
