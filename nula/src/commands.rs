use crate::ast::AstNode;
use crate::cli::{print_error, print_info, print_note, print_success, print_warning, print_compiling, print_parsing, print_optimizing, print_generating_asm, print_assembling, print_linking, print_finished, print_verbose, print_generating_llvm, print_parse_error};
use crate::generator::generate_assembly;
use crate::llvm_generator::generate_llvm;
use crate::interpreter::interpret_ast;
use crate::optimizer::optimize_ast;
use crate::parser::parse_nula_file;
use crate::utils::{get_nula_go_path, get_nula_zig_path, is_in_project, get_lib_dir};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;
use walkdir::WalkDir;
use toml::Value;

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
print_note("Run 'nula build' in the project directory to compile.");
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
print_note("Run 'nula build' to compile.");
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

pub fn build_project(args: &[String], config: &Value) {
    if !is_in_project() {
        print_error("Must be in a Nula project directory (missing nula.toml or main.nula)");
        return;
    }

    let project_name = config.get("project").and_then(|p| p.get("name")).and_then(|n| n.as_str()).unwrap_or("unknown");
    let project_version = config.get("project").and_then(|p| p.get("version")).and_then(|v| v.as_str()).unwrap_or("0.1.0");
    print_info(&format!("Building {} v{}...", project_name, project_version));

    let start = Instant::now();

    let use_gcc = args.iter().any(|a| a == "--gcc");
    let release = args.iter().any(|a| a == "--release");
    let verbose = args.iter().any(|a| a == "--verbose");
    if verbose {
        std::env::set_var("NULA_VERBOSE", "1");
    }
    let target = args.iter().position(|a| a == "--target").and_then(|p| args.get(p + 1).cloned());

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

    let mut asm_files = vec![];
    let mut objects = vec![];

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

        if release {
            print_generating_llvm(file.to_str().unwrap_or("unknown"));
            if let Err(e) = generate_llvm(&optimized_ast, &project_name, release, target.as_deref()) {
                print_error(&format!("LLVM generation failed: {}", e));
                continue;
            }
            // Since generate_llvm compiles to binary, skip further
            continue;
        } else {
            print_generating_asm(file.to_str().unwrap_or("unknown"));
            let asm_code = generate_assembly(&optimized_ast, release, target.as_deref());

            let asm_path = file.with_extension("s");
            if let Err(e) = fs::write(&asm_path, asm_code) {
                print_error(&format!("Failed to write assembly: {}", e));
                continue;
            }
            asm_files.push(asm_path.clone());

            let nula_zig = get_nula_zig_path();
            let mut zig_cmd = Command::new(&nula_zig);
            zig_cmd.arg("optimize").arg(asm_path.to_str().unwrap_or(""));
            if release {
                zig_cmd.arg("--release");
            }
            if let Some(t) = &target {
                zig_cmd.arg("--target").arg(t);
            }
            let zig_output = match zig_cmd.output() {
                Ok(o) => o,
                Err(e) => {
                    print_error(&format!("Failed to execute nula-zig: {}", e));
                    continue;
                }
            };

            if !zig_output.status.success() {
                print_error(&String::from_utf8_lossy(&zig_output.stderr).to_string());
                continue;
            }

            let opt_asm_path = if release {
                asm_path.with_file_name(asm_path.file_stem().unwrap().to_str().unwrap().to_string() + ".opt.s")
            } else {
                asm_path.clone()
            };

            if use_gcc {
                // Use GCC to compile .s to binary
                let bin_path = PathBuf::from(project_name);
                let mut gcc_cmd = Command::new("gcc");
                gcc_cmd.arg("-o").arg(&bin_path);
                gcc_cmd.arg(&opt_asm_path);
                let gcc_output = match gcc_cmd.output() {
                    Ok(o) => o,
                    Err(e) => {
                        print_error(&format!("gcc command failed: {}", e));
                        continue;
                    }
                };
                if !gcc_output.status.success() {
                    print_error(&String::from_utf8_lossy(&gcc_output.stderr).to_string());
                    continue;
                }
                print_success(&format!("Built executable with GCC: {:?}", bin_path));
            } else {
                // Default: assemble to .o
                print_assembling(opt_asm_path.to_str().unwrap_or("unknown"));
                let obj_path = opt_asm_path.with_extension("o");
                let mut as_cmd = Command::new("as");
                as_cmd.arg("-o").arg(&obj_path).arg(&opt_asm_path);
                if let Some(t) = &target {
                    as_cmd.arg(format!("--{}", t));
                }
                let as_output = match as_cmd.output() {
                    Ok(o) => o,
                    Err(e) => {
                        print_error(&format!("as command failed: {}", e));
                        continue;
                    }
                };
                if !as_output.status.success() {
                    print_error(&String::from_utf8_lossy(&as_output.stderr).to_string());
                    continue;
                }
                objects.push(obj_path);
            }
        }
    }

    if !release && !use_gcc && !objects.is_empty() {
        print_linking(&project_name);
        let bin_path = PathBuf::from(project_name);
        let mut ld_cmd = Command::new("ld");
        ld_cmd.arg("-o").arg(&bin_path);
        for obj in &objects {
            ld_cmd.arg(obj);
        }
        ld_cmd.arg("-lc");
        let ld_output = match ld_cmd.output() {
            Ok(o) => o,
            Err(e) => {
                print_error(&format!("ld command failed: {}", e));
                return;
            }
        };
        if !ld_output.status.success() {
            print_error(&String::from_utf8_lossy(&ld_output.stderr).to_string());
            return;
        }
        print_success(&format!("Built executable: {:?}", bin_path));
    } else if !release {
        print_success(&format!("Generated assembly files: {:?}", asm_files));
    }

    let duration = start.elapsed().as_secs_f64();
    print_finished(if release { "release (LLVM)" } else if use_gcc { "gcc" } else { "dev" }, duration);
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
        if entry.path().file_name().and_then(|n| n.to_str()).map(|n| n.starts_with("test_") && n.ends_with(".nula")).unwrap_or(false) {
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
