use crate::cli::print_error;
use std::env;
use std::path::{Path, PathBuf};

pub fn is_in_project() -> bool {
    Path::new("nula.toml").exists() || Path::new("main.nula").exists()
}

pub fn get_lib_dir() -> PathBuf {
    if cfg!(target_os = "windows") {
        env::var("APPDATA").map(PathBuf::from).unwrap_or_default().join("nula")
    } else {
        env::var("HOME").map(PathBuf::from).unwrap_or_default().join(".nula/lib")
    }
}

pub fn get_nula_go_path() -> PathBuf {
    let path = get_lib_dir().join(if cfg!(target_os = "windows") { "nula-go.exe" } else { "nula-go" });
    if !path.exists() {
        print_error(&format!("nula-go not found at {:?}", path));
    }
    path
}

#[allow(dead_code)]
pub fn get_nula_zig_path() -> PathBuf {
    let path = get_lib_dir().join(if cfg!(target_os = "windows") { "nula-zig.exe" } else { "nula-zig" });
    if !path.exists() {
        print_error(&format!("nula-zig not found at {:?}", path));
    }
    path
}
