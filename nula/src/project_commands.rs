use crate::ast::AstNode;
use crate::cli::{print_error, print_info, print_note, print_success, print_warning, print_verbose};
use crate::interpreter::interpret_ast;
use crate::utils::{get_nula_go_path, is_in_project, get_lib_dir};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::process::Command;
use toml::Value;
use reqwest::blocking::Client;
use serde_json::Value as JsonValue;

pub fn create_project(args: &[String]) {
    if args.len() < 3 {
        print_warning("Usage: nula create <project_name>");
        return;
    }

    let name = &args[2];
    let path = Path::new(name);
    if path.exists() {
        print_error(&format!("Project '{}' already exists", name));
        return;
    }

    print_info(&format!("Creating project '{}'...", name));
    if let Err(e) = fs::create_dir_all(path) {
        print_error(&format!("Failed to create directory: {}", e));
        return;
    }

    let main_file = path.join("main.nula");
    let mut file = match File::create(&main_file) {
        Ok(f) => f,
        Err(e) => {
            print_error(&format!("Failed to create main.nula: {}", e));
            return;
        }
    };

    let hello_code = r#"write "Hello, Nula!"
    var x = 42
    write x
    if x == 42 {
        write "It's the answer!"
}
"#;

if let Err(e) = file.write_all(hello_code.as_bytes()) {
    print_error(&format!("Failed to write to main.nula: {}", e));
    return;
}

let config_file = path.join("nula.toml");
let mut config = match File::create(&config_file) {
    Ok(f) => f,
    Err(e) => {
        print_error(&format!("Failed to create nula.toml: {}", e));
        return;
    }
};

let config_content = format!("[project]\nname = \"{}\"\nversion = \"0.1.0\"\n[dependencies]\n", name);
if let Err(e) = config.write_all(config_content.as_bytes()) {
    print_error(&format!("Failed to write to nula.toml: {}", e));
    return;
}

print_success(&format!("Created project '{}'", name));
print_note("Run 'nula build --windows' or 'nula build --linux' in the project directory to compile.");
}

pub fn init_project() {
    if is_in_project() {
        print_warning("Already in a Nula project directory");
        return;
    }

    print_info("Initializing project in current directory...");
    let name = std::env::current_dir()
    .unwrap()
    .file_name()
    .unwrap()
    .to_str()
    .unwrap()
    .to_string();

    let main_file = Path::new("main.nula");
    let mut file = match File::create(main_file) {
        Ok(f) => f,
        Err(e) => {
            print_error(&format!("Failed to create main.nula: {}", e));
            return;
        }
    };

    let hello_code = r#"write "Hello, Nula!"
    var x = 42
    write x
    if x == 42 {
        write "It's the answer!"
}
"#;

if let Err(e) = file.write_all(hello_code.as_bytes()) {
    print_error(&format!("Failed to write to main.nula: {}", e));
    return;
}

let config_file = Path::new("nula.toml");
let mut config = match File::create(config_file) {
    Ok(f) => f,
    Err(e) => {
        print_error(&format!("Failed to create nula.toml: {}", e));
        return;
    }
};

let config_content = format!("[project]\nname = \"{}\"\nversion = \"0.1.0\"\n[dependencies]\n", name);
if let Err(e) = config.write_all(config_content.as_bytes()) {
    print_error(&format!("Failed to write to nula.toml: {}", e));
    return;
}

print_success("Initialized project in current directory");
print_note("Run 'nula build --windows' or 'nula build --linux' to compile.");
}

pub fn install_dependency(args: &[String]) {
    if args.len() < 3 {
        print_warning("Usage: nula install <dependency>");
        return;
    }

    let dep = &args[2];
    let nula_go = get_nula_go_path();
    print_info(&format!("Installing dependency '{}' using nula-go...", dep));

    let output = Command::new(nula_go)
    .arg("install")
    .arg(dep)
    .output()
    .expect("Failed to execute nula-go");

    if output.status.success() {
        print_success(&String::from_utf8_lossy(&output.stdout).to_string());
    } else {
        print_error(&String::from_utf8_lossy(&output.stderr).to_string());
    }
}

pub fn remove_dependency(args: &[String]) {
    if args.len() < 3 {
        print_warning("Usage: nula remove <dependency>");
        return;
    }

    let dep = &args[2];
    let install_dir = get_lib_dir().join(dep);
    if !install_dir.exists() {
        print_warning(&format!("Dependency '{}' not installed", dep));
        return;
    }

    print_info(&format!("Removing dependency '{}'...", dep));
    if let Err(e) = fs::remove_dir_all(&install_dir) {
        print_error(&format!("Failed to remove directory: {}", e));
        return;
    }

    print_success(&format!("Removed dependency '{}'", dep));
}

pub fn update_dependencies() {
    let nula_go = get_nula_go_path();
    print_info("Updating installed dependencies with nula-go...");

    let output = Command::new(nula_go)
    .arg("update")
    .output()
    .expect("Failed to execute nula-go for update");

    if output.status.success() {
        print_success(&String::from_utf8_lossy(&output.stdout).to_string());
    } else {
        print_error(&String::from_utf8_lossy(&output.stderr).to_string());
    }
}

pub fn update_nula() {
    let home = directories::UserDirs::new().unwrap().home_dir().to_path_buf();
    let version_path = home.join(".nula/release/version.toml");
    let current_version = match fs::read_to_string(&version_path) {
        Ok(content) => {
            match content.parse::<Value>() {
                Ok(toml_value) => toml_value
                .get("Nula Lang")
                .and_then(|v| v.get("Version"))
                .and_then(|v| v.as_str())
                .unwrap_or("v0.0")
                .to_string(),
                Err(e) => {
                    print_error(&format!("Failed to parse version.toml: {}", e));
                    return;
                }
            }
        }
        Err(e) => {
            print_error(&format!("Failed to read version.toml: {}", e));
            return;
        }
    };

    print_info(&format!("Current version: {}", current_version));

    let client = Client::new();
    let response = match client
    .get("https://api.github.com/repos/Nula-Lang/Nula/releases/latest")
    .header("User-Agent", "nula-updater")
    .send()
    {
        Ok(r) => r,
        Err(e) => {
            print_error(&format!("Failed to fetch latest release: {}", e));
            return;
        }
    };

    let json: JsonValue = match response.json() {
        Ok(j) => j,
        Err(e) => {
            print_error(&format!("Failed to parse JSON: {}", e));
            return;
        }
    };

    let latest_version = json["tag_name"].as_str().unwrap_or("v0.0").to_string();
    if latest_version == current_version {
        print_info("Already up to date.");
        return;
    }

    print_info(&format!("Updating to {}", latest_version));

    let cmds = vec![
        "sudo rm -rf ~/.nula/lib/nula-go",
        "sudo rm -rf ~/.nula/lib/nula-zig",
        "sudo rm -rf /usr/bin/nula",
        "sudo rm -rf ~/.local/bin/nula",
        "curl -L -o /tmp/install.sh https://raw.githubusercontent.com/Nula-Lang/Nula/main/install/install.sh",
        "cd /tmp && sudo chmod +x ./install.sh && ./install.sh",
    ];

    for cmd in cmds {
        let output = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .expect("Failed to execute update command");

        if !output.status.success() {
            print_error(&String::from_utf8_lossy(&output.stderr).to_string());
            return;
        }
    }

    print_success("Updated Nula successfully.");
}

pub fn resolve_dependencies() {
    let nula_go = get_nula_go_path();
    print_info("Resolving dependencies with nula-go...");

    let output = match Command::new(&nula_go).arg("resolve").output() {
        Ok(o) => o,
        Err(e) => {
            print_error(&format!("Failed to execute nula-go: {}", e));
            return;
        }
    };

    if output.status.success() {
        print_success("All dependencies resolved");
        print_verbose(&String::from_utf8_lossy(&output.stdout).to_string());
    } else {
        print_error(&String::from_utf8_lossy(&output.stderr).to_string());
    }
}

pub fn format_project(args: &[String]) {
    if args.len() < 3 {
        print_warning("Usage: nula fmt <file>");
        return;
    }

    let file = &args[2];
    let path = Path::new(file);
    if !path.exists() {
        print_error(&format!("File '{}' does not exist", file));
        return;
    }

    print_info(&format!("Formatting file '{}'...", file));
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            print_error(&format!("Failed to read file: {}", e));
            return;
        }
    };

    let formatted = format_code(&content);
    if let Err(e) = fs::write(path, formatted) {
        print_error(&format!("Failed to write formatted file: {}", e));
        return;
    }

    print_success(&format!("Formatted '{}'", file));
}

pub fn format_code(content: &str) -> String {
    let lines: Vec<String> = content.lines().map(|s| s.trim_end().to_string()).collect();
    let mut formatted = Vec::new();
    let mut indent_level: i32 = 0;

    for line in lines {
        let trimmed = line.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('@') {
            formatted.push(line);
            continue;
        }

        let indent = if trimmed.starts_with('}') || trimmed.starts_with("end") || trimmed.starts_with("else") {
            indent_level = indent_level.saturating_sub(1);
            " ".repeat(indent_level as usize)
        } else {
            " ".repeat(indent_level as usize)
        };

        if trimmed.ends_with('{') || trimmed.contains(" fn ") || trimmed.contains(" for ") ||
            trimmed.contains(" while ") || trimmed.contains(" if ") || trimmed.ends_with("do") {
                formatted.push(format!("{}{}", indent, trimmed));
                indent_level += 1;
            } else {
                formatted.push(format!("{}{}", indent, trimmed));
            }
    }

    formatted.join("\n") + "\n"
}

pub fn interpret_test(ast: &AstNode) -> bool {
    let result = interpret_ast(ast);
    result == 0.0
}
