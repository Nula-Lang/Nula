use anyhow::Result;
use colored::*;
use reqwest::Client;
use git2::Repository;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;
use regex::Regex;

pub async fn install_dep(dep: &str) -> Result<()> {
    let lib_url = "https://raw.githubusercontent.com/Nula-Lang/Nula/main/nula/library.nula";
    let client = Client::new();
    let lib_content = client.get(lib_url).send().await?.text().await?;
    
    let mut repo_map = std::collections::HashMap::new();
    for line in lib_content.lines() {
        if let Some((key, val)) = line.split_once(" -> ") {
            repo_map.insert(key.trim().to_string(), val.trim().to_string());
        }
    }
    
    if let Some(repo) = repo_map.get(dep) {
        let tmp_dir = tempdir()?;
        let repo_path = tmp_dir.path().join(dep);
        Repository::clone(repo, &repo_path).context("Clone failed")?;
        
        let install_dir = Path::new("/usr/lib/.nula-lib/").join(dep);
        fs::create_dir_all(&install_dir)?;
        copy_dir(&repo_path, &install_dir)?;
        
        println!("{} Installed {}", "✓".green(), dep.green());
    } else {
        println!("{} Dep not found", "✗".red());
    }
    Ok(())
}

fn copy_dir(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dst_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir(&entry.path(), &dst_path)?;
        } else {
            fs::copy(entry.path(), dst_path)?;
        }
    }
    Ok(())
}

pub fn resolve_deps(deps: &[String]) -> Result<()> {
    for dep in deps {
        let path = Path::new("/usr/lib/.nula-lib/").join(dep);
        if !path.exists() {
            println!("{} Missing {}, please install", "⚠".yellow(), dep.yellow());
        } else {
            println!("{} Resolved {}", "✓".green(), dep.green());
        }
    }
    Ok(())
}

pub fn load_bottles_deps(project_dir: &PathBuf) -> Result<Vec<String>> {
    let bottles_file = project_dir.join("nula.bottles");
    if !bottles_file.exists() {
        return Ok(vec![]);  // Opcjonalny
    }
    let content = fs::read_to_string(&bottles_file)?;
    let re = Regex::new(r":<(\w+)>:").unwrap();
    let deps: Vec<String> = re.captures_iter(&content).map(|c| c[1].to_string()).collect();
    Ok(deps)
}
