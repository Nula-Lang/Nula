use crate::cli::{print_error, print_info, print_note, print_success, print_warning, print_compiling, print_finished};
use crate::generator::generate_assembly;
use crate::interpreter::interpret_ast;
use crate::optimizer::optimize_ast;
use crate::parser::parse_nula_file;
use crate::utils::{get_nula_go_path, get_nula_zig_path, is_in_project};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;
use walkdir::WalkDir;

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
    if let Err(e) = file.write_all(b"write \"Hello, Nula!\"\n") {
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
}

pub fn install_dependency(args: &[String]) {
    if args.len() < 3 {
        print_warning("Usage: nula install <dependency>");
        return;
    }
    let dep = &args[2];
    let temp_file = "/tmp/library.nula";

    print_info("Downloading library index...");
    let curl_output = Command::new("curl")
    .arg("-s")
    .arg("-o")
    .arg(temp_file)
    .arg("https://raw.githubusercontent.com/Nula-Lang/Nula/main/nula/library.nula")
    .output();

    match curl_output {
        Ok(o) if o.status.success() => print_note("Library index downloaded"),
        Ok(o) => {
            print_error(&String::from_utf8_lossy(&o.stderr).to_string());
            return;
        }
        Err(e) => {
            print_error(&format!("Failed to execute curl: {}", e));
            return;
        }
    }

    let content = match fs::read_to_string(temp_file) {
        Ok(c) => c,
        Err(e) => {
            print_error(&format!("Failed to read index: {}", e));
            return;
        }
    };

    let mut repo_url = None;
    for line in content.lines() {
        let parts: Vec<&str> = line.split("_>").collect();
        if parts.len() == 2 && parts[0].trim() == dep {
            repo_url = Some(parts[1].trim().to_string());
            break;
        }
    }

    let Some(url) = repo_url else {
        print_error(&format!("Dependency '{}' not found in index", dep));
        return;
    };

    let folder = url.split('/').last().unwrap_or(dep).replace(".git", "");
    let install_path = format!("/usr/lib/nula/{}", folder);

    if Path::new(&install_path).exists() {
        print_warning(&format!("Dependency '{}' already installed", dep));
        return;
    }

    print_info(&format!("Cloning {} into {}...", url, install_path));
    let git_output = Command::new("git")
    .arg("clone")
    .arg("--quiet")
    .arg(&url)
    .arg(&install_path)
    .output();

    match git_output {
        Ok(o) if o.status.success() => print_success(&format!("Installed dependency '{}'", dep)),
        Ok(o) => {
            print_error(&String::from_utf8_lossy(&o.stderr).to_string());
            return;
        }
        Err(e) => {
            print_error(&format!("Failed to execute git: {}", e));
            return;
        }
    }
}

pub fn remove_dependency(args: &[String]) {
    if args.len() < 3 {
        print_warning("Usage: nula remove <dependency>");
        return;
    }
    let dep = &args[2];
    let temp_file = "/tmp/library.nula";

    print_info("Downloading library index...");
    let curl_output = Command::new("curl")
    .arg("-s")
    .arg("-o")
    .arg(temp_file)
    .arg("https://raw.githubusercontent.com/Nula-Lang/Nula/main/nula/library.nula")
    .output();

    match curl_output {
        Ok(o) if o.status.success() => print_note("Library index downloaded"),
        Ok(o) => {
            print_error(&String::from_utf8_lossy(&o.stderr).to_string());
            return;
        }
        Err(e) => {
            print_error(&format!("Failed to execute curl: {}", e));
            return;
        }
    }

    let content = match fs::read_to_string(temp_file) {
        Ok(c) => c,
        Err(e) => {
            print_error(&format!("Failed to read index: {}", e));
            return;
        }
    };

    let mut folder = dep.to_string();
    for line in content.lines() {
        let parts: Vec<&str> = line.split("_>").collect();
        if parts.len() == 2 && parts[0].trim() == dep {
            folder = parts[1].trim().split('/').last().unwrap_or(dep).replace(".git", "");
            break;
        }
    }

    let install_path = format!("/usr/lib/nula/{}", folder);

    if !Path::new(&install_path).exists() {
        print_warning(&format!("Dependency '{}' not installed", dep));
        return;
    }

    print_info(&format!("Removing dependency '{}'...", dep));
    if let Err(e) = fs::remove_dir_all(&install_path) {
        print_error(&format!("Failed to remove directory: {}", e));
        return;
    }

    print_success(&format!("Removed dependency '{}'", dep));
}

pub fn build_project(args: &[String]) {
    if !is_in_project() {
        print_error("Must be in a Nula project directory (missing nula.toml or main.nula)");
        return;
    }

    let start = Instant::now();

    let release = args.iter().any(|a| a == "--release");
    let target = args
    .iter()
    .position(|a| a == "--target")
    .and_then(|p| args.get(p + 1).cloned());

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
        print_info(&format!("Parsing {:?}", file));
        let ast = match parse_nula_file(file) {
            Ok(a) => a,
            Err(err) => {
                print_error(&format!("Parse error in {:?}: {}", file, err));
                continue;
            }
        };

        let optimized_ast = if release {
            print_info("Optimizing AST...");
            optimize_ast(&ast)
        } else {
            ast
        };

        print_info(&format!("Generating assembly for {:?}", file));
        let asm_code = generate_assembly(&optimized_ast, release, target.as_deref());

        let asm_path = file.with_extension("s");
        if let Err(e) = fs::write(&asm_path, asm_code) {
            print_error(&format!("Failed to write assembly: {}", e));
            continue;
        }

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

        let bin_path = if release {
            file.with_extension("release")
        } else {
            file.with_extension("")
        };
        let mut gcc_cmd = Command::new("gcc");
        gcc_cmd
        .arg("-o")
        .arg(bin_path.to_str().unwrap_or(""))
        .arg(asm_path.to_str().unwrap_or(""));
        if release {
            gcc_cmd.arg("-O3");
        }
        if let Some(t) = &target {
            gcc_cmd.arg("-march=").arg(t);
        }
        let gcc_output = match gcc_cmd.output() {
            Ok(o) => o,
            Err(e) => {
                print_error(&format!("gcc not found or failed: {}", e));
                continue;
            }
        };

        if gcc_output.status.success() {
            print_success(&format!("Built {:?}", bin_path));
        } else {
            print_error(&String::from_utf8_lossy(&gcc_output.stderr).to_string());
        }
    }

    let duration = start.elapsed().as_secs_f64();
    print_finished(if release { "release" } else { "dev" }, duration);
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
    let ast = match parse_nula_file(&file_path) {
        Ok(a) => a,
        Err(err) => {
            print_error(&format!("Parse error: {}", err));
            return;
        }
    };

    interpret_ast(&ast);
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
    } else {
        print_error(&String::from_utf8_lossy(&output.stderr).to_string());
    }
}
