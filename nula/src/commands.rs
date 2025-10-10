use crate::cli::{print_error, print_info, print_success, print_warning, print_compiling, print_parsing, print_optimizing, print_finished, print_parse_error};
use crate::cranelift_generator::generate_cranelift;
use crate::interpreter::interpret_ast;
use crate::optimizer::optimize_ast;
use crate::parser::parse_nula_file;
use crate::repl::start_repl;
use crate::utils::is_in_project;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;
use toml::Value;
use walkdir::WalkDir;

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
    crate::project_commands::resolve_dependencies();

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
            let win_name = format!("{}windows", project_name);
            if let Err(e) = generate_cranelift(&optimized_ast, &win_name, release, "x86_64-pc-windows-msvc") {
                print_error(&format!("Cranelift generation for Windows failed: {}", e));
            }
        }

        if linux {
            let lin_name = format!("{}linux", project_name);
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
            .map(|n| n.starts_with("test") && n.ends_with(".nula"))
            .unwrap_or(false)
            {
                test_files.push(entry.path().to_path_buf());
            }
    }

    if test_files.is_empty() {
        print_warning("No test files found (expected files starting with 'test' and ending with '.nula')");
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

        if crate::project_commands::interpret_test(&ast) {
            print_success(&format!("Test passed: {:?}", file));
            passed += 1;
        } else {
            print_error(&format!("Test failed: {:?}", file));
            failed += 1;
        }
    }

    print_info(&format!("Test summary: {} passed, {} failed", passed, failed));
}

pub fn repl(_args: &[String]) {
    print_info("Starting Nula REPL...");
    start_repl();
    print_success("REPL exited successfully");
}
