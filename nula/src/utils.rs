use crate::cli::print_error;
use std::env;
use std::path::{Path, PathBuf};

pub fn is_in_project() -> bool {
    Path::new("nula.toml").exists() || Path::new("main.nula").exists()
}

pub fn get_nula_go_path() -> PathBuf {
    let path = get_nula_bin_path("nula-go");
    if !path.exists() {
        print_error(&format!("nula-go not found at {:?}", path));
    }
    path
}

pub fn get_nula_zig_path() -> PathBuf {
    let path = get_nula_bin_path("nula-zig");
    if !path.exists() {
        print_error(&format!("nula-zig not found at {:?}", path));
    }
    path
}

fn get_nula_bin_path(name: &str) -> PathBuf {
    if cfg!(target_os = "windows") {
        env::current_exe()
        .unwrap_or_default()
        .parent()
        .unwrap_or(Path::new(""))
        .join(format!("{}.exe", name))
    } else {
        env::current_exe()
        .unwrap_or_default()
        .parent()
        .unwrap_or(Path::new(""))
        .join(name)
    }
}
