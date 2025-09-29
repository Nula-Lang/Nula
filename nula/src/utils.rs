use directories::BaseDirs;
use std::env;
use std::path::{Path, PathBuf};

pub fn is_in_project() -> bool {
    Path::new("nula.toml").exists() || Path::new("main.nula").exists()
}

pub fn get_lib_dir() -> PathBuf {
    if let Some(dirs) = BaseDirs::new() {
        dirs.data_local_dir().join(".nula-lib")
    } else {
        env::current_dir().unwrap_or_default().join(".nula-lib")
    }
}

pub fn get_nula_go_path() -> PathBuf {
    get_nula_bin_path("nula-go")
}

pub fn get_nula_zig_path() -> PathBuf {
    get_nula_bin_path("nula-zig")
}

fn get_nula_bin_path(name: &str) -> PathBuf {
    if cfg!(target_os = "windows") {
        env::current_exe().unwrap_or_default().parent().unwrap_or(Path::new("")).join(format!("{}.exe", name))
    } else {
        env::current_exe().unwrap_or_default().parent().unwrap_or(Path::new("")).join(name)
    }
}
