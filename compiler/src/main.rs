// src/main.rs - Main entry point for nula-compiler

use std::env;
use std::fs;
use std::io;
use std::path::Path;
use std::process;

use cranelift::prelude::*;
use cranelift_codegen::isa::{self, CallConv};
use cranelift_codegen::settings;
use cranelift_codegen::Context as CodegenContext;
use cranelift_module::{DataContext, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};

use nula_compiler::ast::Ast;
use nula_compiler::codegen::CodeGen;
use nula_compiler::parser::Parser;

mod ast;
mod parser;
mod codegen;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 || args[1] != "--platform" {
        eprintln!("Usage: nula-compiler --platform <platform> <file.nula>");
        process::exit(1);
    }
    let platform = &args[2];
    let file = if args.len() > 3 { &args[3] } else { "main.nula".to_string() };

    // Read code
    let code = fs::read_to_string(&file)?;

    // Parse
    let mut parser = Parser::new(&code);
    let ast = parser.parse();

    // Setup Cranelift
    let triple_str = match platform.as_str() {
        "linux" => "x86_64-unknown-linux-gnu",
        "windows" => "x86_64-pc-windows-msvc",
        "macos" => "x86_64-apple-darwin",
        _ => {
            eprintln!("Unsupported platform: {}", platform);
            process::exit(1);
        }
    };
    let flag_builder = settings::builder();
    let isa_builder = isa::lookup_by_name(triple_str).unwrap();
    let isa = isa_builder.finish(settings::Flags::new(flag_builder)).unwrap();

    let builder = ObjectBuilder::new(isa, "nula_bin".to_string(), cranelift_module::default_libcall_names()).unwrap();
    let mut module = ObjectModule::new(builder);

    // printf
    let mut printf_sig = module.make_signature();
    printf_sig.params.push(AbiParam::new(types::I64));
    printf_sig.returns.push(AbiParam::new(types::I32));
    printf_sig.call_conv = CallConv::C;
    let printf = module.declare_function("printf", Linkage::Import, &printf_sig).unwrap();

    // Main function
    let mut main_sig = module.make_signature();
    main_sig.returns.push(AbiParam::new(types::I32));
    let main_id = module.declare_function("main", Linkage::Export, &main_sig).unwrap();

    let mut ctx = CodegenContext::new();
    ctx.func.signature = main_sig;

    let mut builder_ctx = FunctionBuilderContext::new();
    let mut func_builder = FunctionBuilder::new(&mut ctx.func, &mut builder_ctx);

    let entry_block = func_builder.create_block();
    func_builder.switch_to_block(entry_block);
    func_builder.seal_block(entry_block);

    let mut codegen = CodeGen::new(&mut module, &mut func_builder, printf);

    for node in ast {
        codegen.gen_ast(&node);
    }

    let zero = codegen.builder.ins().iconst(types::I32, 0);
    codegen.builder.ins().return_(&[zero]);

    module.define_function(main_id, &mut ctx).unwrap();
    module.finalize_definitions();

    // Write object file
    let obj_bytes = module.object.write().unwrap();
    let project_dir = Path::new(&file).parent().unwrap_or(Path::new("."));
    let nula_dir = project_dir.join("nula");
    let bin_dir = nula_dir.join("bin");
    fs::create_dir_all(&bin_dir)?;
    let obj_path = bin_dir.join("nula_bin.o");
    fs::write(&obj_path, obj_bytes)?;

    // Link to executable
    let linker = match platform.as_str() {
        "linux" => "gcc",
        "windows" => "link.exe",
        "macos" => "clang",
        _ => unreachable!(),
    };
    let exe_path = bin_dir.join(if platform == "windows" { "nula_bin.exe" } else { "nula_bin" });
    let status = Command::new(linker)
        .arg(obj_path.to_str().unwrap())
        .arg("-o")
        .arg(exe_path.to_str().unwrap())
        .arg(if platform == "linux" { "-lc" } else { "" })
        .status()?;

    if !status.success() {
        eprintln!("Linking failed");
        process::exit(1);
    }

    println!("Compiled to {:?}", exe_path);
    Ok(())
}
