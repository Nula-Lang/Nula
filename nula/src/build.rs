use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=nula.pest");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    pest_meta::generator::generate(
        PathBuf::from("nula.pest"),
        &out_dir.join("nula.rs"),
    )
    .unwrap();
}
