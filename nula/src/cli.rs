use colored::*;

pub fn show_help() {
    println!("{} Nula Commands:", "Nula CLI".cyan().bold());
    println!("  {} - Show this help", "?".yellow());
    println!("  {} - Build to binary", "build".yellow());
    println!("  {} - Run .nula file", "run".yellow());
    println!("  {} <name> - Create project", "create".yellow());
    println!("  {} <dep> - Install dep", "install".yellow());
    // Kolorowe błędy: w main używaj .red() dla error
}

pub fn error(msg: &str, detail: &str) {
    eprintln!("{} {}", "ERROR".red().bold(), msg.red());
    eprintln!("  {} {}", "Detail:".magenta(), detail.yellow());
    std::process::exit(1);
}
