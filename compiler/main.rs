// nula-compiler.rs - Expanded Compiler in Rust with cranelift
// Handles more syntax: variables, write, basic math.
// Generates executable by linking (assumes linker available, e.g., gcc on Linux).
// For cross-platform, uses target-specific linking.
// Compile: cargo add cranelift-codegen cranelift-module cranelift-object --features std
// Then cargo build --release; mv target/release/nula-compiler ~/.nula/bin/

use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, self};

use cranelift::prelude::*;
use cranelift_codegen::ir::{AbiParam, InstBuilder, types::*};
use cranelift_codegen::isa::{self, CallConv};
use cranelift_codegen::settings;
use cranelift_codegen::Context as CodegenContext;
use cranelift_module::{DataContext, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: nula-compiler --platform <platform> <file.nula>");
        process::exit(1);
    }
    let platform_flag = &args[1];
    if platform_flag != "--platform" {
        eprintln!("Expected --platform");
        process::exit(1);
    }
    let platform = &args[2];
    let file = if args.len() > 3 { &args[3] } else { "main.nula" };

    // Parse .nula
    let mut messages = Vec::new();
    let mut variables = HashMap::new();
    let input = File::open(file)?;
    let buffered = io::BufReader::new(input);
    for line in buffered.lines() {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.starts_with("write ") {
            let msg = trimmed.trim_start_matches("write ").trim_matches('"');
            messages.push(msg.to_string());
        } else if trimmed.contains(" = ") {
            let parts: Vec<&str> = trimmed.splitn(2, " = ").collect();
            variables.insert(parts[0].to_string(), parts[1].to_string());
        }
        // Add more parsing as needed
    }

    // Target
    let (triple, linker) = match platform.as_str() {
        "linux" => ("x86_64-unknown-linux-gnu", "gcc"),
        "windows" => ("x86_64-pc-windows-msvc", "link.exe"), // Assume MSVC
        "macos" => ("x86_64-apple-darwin", "clang"),
        _ => {
            eprintln!("Unsupported platform");
            process::exit(1);
        }
    };

    // Cranelift setup
    let flag_builder = settings::builder();
    let isa_builder = isa::lookup_by_name(triple).expect("Invalid triple");
    let isa = isa_builder.finish(settings::Flags::new(flag_builder)).expect("ISA build failed");

    let builder = ObjectBuilder::new(isa, "nula_bin", cranelift_module::default_libcall_names()).unwrap();
    let mut module = ObjectModule::new(builder);

    // Declare printf
    let mut sig = module.make_signature();
    sig.params.push(AbiParam::new(I64));
    sig.returns.push(AbiParam::new(I32));
    sig.call_conv = CallConv::C;
    let printf = module.declare_function("printf", Linkage::Import, &sig).unwrap();

    // Main function
    let mut ctx = CodegenContext::new();
    let mut func_sig = module.make_signature();
    func_sig.returns.push(AbiParam::new(I32));
    let main_id = module.declare_function("main", Linkage::Export, &func_sig).unwrap();

    ctx.func.signature = func_sig;
    let mut builder_ctx = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut ctx.func, &mut builder_ctx);

    let entry = builder.create_block();
    builder.switch_to_block(entry);
    builder.seal_block(entry);

    for msg in messages {
        // Resolve if var
        let actual_msg = if let Some(val) = variables.get(&msg) { val } else { &msg };

        // Data for string
        let mut data_ctx = DataContext::new();
        data_ctx.define(format!("{}\n\0", actual_msg).into_bytes().into_boxed_slice());
        let data_id = module.declare_data("str_data", Linkage::Local, true, false).unwrap();
        module.define_data(data_id, &data_ctx).unwrap();

        let str_val = builder.ins().global_value(I64, data_id);
        builder.ins().call(printf, &[str_val]);
    }

    let zero = builder.ins().iconst(I32, 0);
    builder.ins().return_(&[zero]);

    module.define_function(main_id, &mut ctx).unwrap();
    module.finalize_definitions();

    // Write object
    let obj_bytes = module.object.write().unwrap();
    let project_dir = Path::new(file).parent().unwrap_or(Path::new("."));
    let nula_path = project_dir.join("nula");
    let bin_path = nula_path.join("bin");
    fs::create_dir_all(&bin_path)?;
    let obj_path = bin_path.join("nula_bin.o");
    fs::write(&obj_path, obj_bytes)?;

    // Link to executable
    let exe_path = bin_path.join(if platform == "windows" { "nula_bin.exe" } else { "nula_bin" });
    let status = Command::new(linker)
        .arg(obj_path.to_str().unwrap())
        .arg("-o")
        .arg(exe_path.to_str().unwrap())
        .arg(if platform == "linux" { "-lc" } else { "" }) // Link libc
        .status()?;

    if !status.success() {
        eprintln!("Linking failed");
        process::exit(1);
    }

    println!("Built executable at {:?}", exe_path);
    Ok(())
}
