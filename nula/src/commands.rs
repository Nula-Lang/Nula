use crate::ast::AstNode;
use crate::cli::{print_error, print_info, print_note, print_success, print_warning, print_compiling, print_parsing, print_optimizing, print_finished, print_verbose, print_parse_error};
use crate::cranelift_generator::generate_cranelift;
use crate::interpreter::interpret_ast;
use crate::optimizer::optimize_ast;
use crate::parser::parse_nula_file;
use crate::utils::{get_nula_go_path, is_in_project, get_lib_dir};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;
use walkdir::WalkDir;
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
    let name = std::env::current_dir().unwrap().file_name().unwrap().to_str().unwrap().to_string();
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
                Err(_) => {
                    print_error("Failed to parse version.toml");
                    return;
                }
            }
        }
        Err(_) => {
            print_error("Failed to read version.toml");
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

pub fn build_project(args: &[String], config: &Value) {
    if !is_in_project() {
        print_error("Must be in a Nula project directory (missing nula.toml or main.nula)");
        return;
    }

    let windows = args.iter().any(|a| a == "--windows");
    let linux = args.iter().any(|a| a == "--linux");
    if !windows && !linux {
        print_error("Must specify at least one target: --windows or --linux");
        return;
    }

    let project_name = config
        .get("project")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .unwrap_or("unknown");
    let project_version = config
        .get("project")
        .and_then(|p| p.get("version"))
        .and_then(|v| v.as_str())
        .unwrap_or("0.1.0");
    print_info(&format!("Building {} v{}...", project_name, project_version));

    let start = Instant::now();

    let release = args.iter().any(|a| a == "--release");
    let verbose = args.iter().any(|a| a == "--verbose");
    if verbose {
        std::env::set_var("NULA_VERBOSE", "1");
    }

    print_info("Resolving dependencies...");
    resolve_dependencies();

    print_info("Scanning for .nula files...");
    let mut nula_files = vec![];
    for entry in WalkDir::new(".").into_iter().filter_map(|e| e.ok()) {
        if entry.path().extension().and_then(|s| s.to_str()) == Some("nula") {
            nula_files.push(entry.path().to_path_buf());
        }
    }

    if nula_files.is_empty() {
        print_warning("No .nula files found in project");
        return;
    }

    for file in &nula_files {
        print_compiling(file.to_str().unwrap_or("unknown"));
        print_parsing(file.to_str().unwrap_or("unknown"));
        let content = match fs::read_to_string(file) {
            Ok(c) => c,
            Err(e) => {
                print_error(&format!("Failed to read file: {}", e));
                continue;
            }
        };
        let ast = match parse_nula_file(file) {
            Ok(a) => a,
            Err(err) => {
                print_parse_error(&err, file.to_str().unwrap_or("unknown"), &content);
                continue;
            }
        };

        let optimized_ast = if release {
            print_optimizing();
            optimize_ast(ast)
        } else {
            ast
        };

        if windows {
            let win_name = format!("{}_windows", project_name);
            if let Err(e) = generate_cranelift(&optimized_ast, &win_name, release, "x86_64-pc-windows-msvc") {
                print_error(&format!("Cranelift generation for Windows failed: {}", e));
            }
        }
        if linux {
            let lin_name = format!("{}_linux", project_name);
            if let Err(e) = generate_cranelift(&optimized_ast, &lin_name, release, "x86_64-unknown-linux-gnu") {
                print_error(&format!("Cranelift generation for Linux failed: {}", e));
            }
        }
    }

    let duration = start.elapsed().as_secs_f64();
    print_finished(if release { "release (Cranelift)" } else { "dev" }, duration);
}

pub fn run_project(args: &[String]) {
    if !is_in_project() {
        print_error("Must be in a Nula project directory");
        return;
    }

    let file_path = if args.len() > 2 {
        PathBuf::from(&args[2])
    } else {
        PathBuf::from("main.nula")
    };

    if !file_path.exists() {
        print_warning(&format!("File {:?} not found", file_path));
        return;
    }

    print_info(&format!("Parsing and running {:?}", file_path));
    let content = match fs::read_to_string(&file_path) {
        Ok(c) => c,
        Err(e) => {
            print_error(&format!("Failed to read file: {}", e));
            return;
        }
    };
    let ast = match parse_nula_file(&file_path) {
        Ok(a) => a,
        Err(err) => {
            print_parse_error(&err, file_path.to_str().unwrap_or("unknown"), &content);
            return;
        }
    };

    interpret_ast(&ast);
}

pub fn check_project(args: &[String]) {
    if !is_in_project() {
        print_error("Must be in a Nula project directory");
        return;
    }

    let file_path = if args.len() > 2 {
        PathBuf::from(&args[2])
    } else {
        PathBuf::from("main.nula")
    };

    if !file_path.exists() {
        print_warning(&format!("File {:?} not found", file_path));
        return;
    }

    let content = match fs::read_to_string(&file_path) {
        Ok(c) => c,
        Err(e) => {
            print_error(&format!("Failed to read file: {}", e));
            return;
        }
    };
    print_info(&format!("Checking {:?}", file_path));
    match parse_nula_file(&file_path) {
        Ok(_) => print_success(&format!("No syntax errors in {:?}", file_path)),
        Err(err) => {
            print_parse_error(&err, file_path.to_str().unwrap_or("unknown"), &content);
        }
    }
}

pub fn format_project(args: &[String]) {
    if !is_in_project() {
        print_error("Must be in a Nula project directory");
        return;
    }

    let file_path = if args.len() > 2 {
        PathBuf::from(&args[2])
    } else {
        PathBuf::from("main.nula")
    };

    if !file_path.exists() {
        print_warning(&format!("File {:?} not found", file_path));
        return;
    }

    print_info(&format!("Formatting {:?}", file_path));
    let content = match fs::read_to_string(&file_path) {
        Ok(c) => c,
        Err(e) => {
            print_error(&format!("Failed to read file: {}", e));
            return;
        }
    };

    let formatted = format_code(&content);
    if let Err(e) = fs::write(&file_path, formatted) {
        print_error(&format!("Failed to write formatted file: {}", e));
        return;
    }
    print_success(&format!("Formatted {:?}", file_path));
}

fn format_code(content: &str) -> String {
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
            "    ".repeat(indent_level as usize)
        } else {
            "    ".repeat(indent_level as usize)
        };

        if trimmed.ends_with('{') || trimmed.contains(" fn ") || trimmed.contains(" for ") || trimmed.contains(" while ") || trimmed.contains(" if ") || trimmed.ends_with("do") {
            formatted.push(format!("{}{}", indent, trimmed));
            indent_level += 1;
        } else {
            formatted.push(format!("{}{}", indent, trimmed));
        }
    }

    formatted.join("\n") + "\n"
}

pub fn test_project(_args: &[String]) {
    if !is_in_project() {
        print_error("Must be in a Nula project directory");
        return;
    }

    print_info("Scanning for test files...");
    let mut test_files = vec![];
    for entry in WalkDir::new(".").into_iter().filter_map(|e| e.ok()) {
        if entry
            .path()
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.starts_with("test_") && n.ends_with(".nula"))
            .unwrap_or(false)
        {
            test_files.push(entry.path().to_path_buf());
        }
    }

    if test_files.is_empty() {
        print_warning("No test files found (expected files starting with 'test_' and ending with '.nula')");
        return;
    }

    let mut passed = 0;
    let mut failed = 0;

    for file in test_files {
        print_info(&format!("Running test {:?}", file));
        let content = match fs::read_to_string(&file) {
            Ok(c) => c,
            Err(e) => {
                print_error(&format!("Failed to read file: {}", e));
                failed += 1;
                continue;
            }
        };
        let ast = match parse_nula_file(&file) {
            Ok(a) => a,
            Err(err) => {
                print_parse_error(&err, file.to_str().unwrap_or("unknown"), &content);
                failed += 1;
                continue;
            }
        };

        if interpret_test(&ast) {
            print_success(&format!("Test passed: {:?}", file));
            passed += 1;
        } else {
            print_error(&format!("Test failed: {:?}", file));
            failed += 1;
        }
    }

    print_info(&format!("Test summary: {} passed, {} failed", passed, failed));
}

fn interpret_test(ast: &AstNode) -> bool {
    let result = interpret_ast(ast);
    result == 0.0
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
