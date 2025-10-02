use inkwell::AddressSpace;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::passes::PassManager;
use inkwell::targets::{CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetTriple};
use inkwell::types::BasicMetadataTypeEnum;
use inkwell::values::{BasicMetadataValueEnum, BasicValue, BasicValueEnum, FunctionValue, PointerValue};
use inkwell::FloatPredicate;
use inkwell::OptimizationLevel;
use crate::ast::AstNode;
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

// Note: If you encounter linking errors (e.g., undefined references to LLVMAdd*Pass),
// this is likely due to the LLVM library not being linked properly.
// To fix this, create a 'build.rs' file in the root of your Cargo project with the following content:
// fn main() {
//     println!("cargo:rustc-link-lib=LLVM-20");
// }
// Additionally, ensure your Cargo.toml specifies the correct feature for LLVM 20:
// inkwell = { version = "0.6.0", features = ["llvm20-0"] }
// If llvm-config is named llvm-config-20, set the environment variable:
// export LLVM_CONFIG_PATH=/usr/bin/llvm-config-20
// before running cargo build.
// Make sure the inkwell version supports LLVM 20 (check the latest version on crates.io or use git dependency if necessary).

pub fn generate_llvm(ast: &AstNode, project_name: &str, release: bool, target: Option<&str>) -> Result<(), String> {
    let context = Context::create();
    let module = context.create_module(project_name);
    let builder = context.create_builder();

    Target::initialize_all(&InitializationConfig::default());

    let triple = target.map_or_else(|| TargetTriple::create("x86_64-pc-linux-gnu"), TargetTriple::create);
    let target = Target::from_triple(&triple).map_err(|e| format!("Failed to create target: {}", e))?;
    let target_machine = target
    .create_target_machine(
        &triple,
        "generic",
        "",
        if release { OptimizationLevel::Aggressive } else { OptimizationLevel::None },
            RelocMode::PIC,
            CodeModel::Default,
    )
    .ok_or_else(|| "Failed to create target machine".to_string())?;

    module.set_triple(&triple);
    module.set_data_layout(&target_machine.get_target_data().get_data_layout());

    let void_type = context.void_type();
    let main_fn_type = void_type.fn_type(&[], false);
    let main_fn = module.add_function("main", main_fn_type, None);
    let entry = context.append_basic_block(main_fn, "entry");
    builder.position_at_end(entry);

    let mut env = HashMap::new();
    build_llvm_node(ast, &builder, &context, &module, main_fn, &mut env)?;

    if builder.get_insert_block().is_none() {
        builder.position_at_end(entry);
    }
    builder.build_return(None);

    if let Err(e) = module.verify() {
        return Err(format!("Module verification failed: {}", e));
    }

    let fpm = PassManager::create(&module);
    if release {
        // Commented out due to linking issues with LLVM-20; fix linking as per note above
        // fpm.add_function_inlining_pass();
        // fpm.add_global_dce_pass();
        // fpm.add_instruction_combining_pass();
        // fpm.add_reassociate_pass();
        // fpm.add_gvn_pass();
        // fpm.add_cfg_simplification_pass();
        // fpm.add_dead_store_elimination_pass();
        // fpm.add_aggressive_dce_pass();
        // fpm.add_loop_deletion_pass();
        // fpm.add_loop_rotate_pass();
        // fpm.add_ind_var_simplify_pass();
        // Removed add_loop_unroll_and_jam_pass due to potential LLVM version incompatibility
    }
    fpm.run_on(&main_fn);

    let obj_path = format!("{}.o", project_name);
    target_machine
    .write_to_file(&module, FileType::Object, Path::new(&obj_path))
    .map_err(|e| format!("Failed to write object file: {}", e))?;

    let bin_path = project_name.to_string();
    let mut ld_cmd = Command::new("ld");
    ld_cmd
    .arg("-o")
    .arg(&bin_path)
    .arg(&obj_path)
    .arg("-lc")
    .arg("-dynamic-linker")
    .arg("/lib64/ld-linux-x86-64.so.2");
    let output = ld_cmd.output().map_err(|e| format!("Failed to execute linker: {}", e))?;
    if !output.status.success() {
        return Err(format!("Linker failed: {}", String::from_utf8_lossy(&output.stderr)));
    }

    Ok(())
}

fn build_llvm_node<'ctx>(
    node: &AstNode,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    module: &Module<'ctx>,
    current_fn: FunctionValue<'ctx>,
    env: &mut HashMap<String, PointerValue<'ctx>>,
) -> Result<(), String> {
    match node {
        AstNode::Program(nodes) => {
            for n in nodes {
                build_llvm_node(n, builder, context, module, current_fn, env)?;
            }
        }
        AstNode::Translation(_, _) | AstNode::Dependency(_) | AstNode::Import(_) | AstNode::Comment(_) => {}
        AstNode::VariableDecl(name, expr) | AstNode::Assignment(name, expr) => {
            let val = build_llvm_expression(expr, builder, context, module, env)?.into_float_value();
            let alloca = builder.build_alloca(context.f64_type(), name);
            builder.build_store(alloca, val);
            env.insert(name.clone(), alloca);
        }
        AstNode::FunctionDef(name, params, body) => {
            let param_types: Vec<BasicMetadataTypeEnum> = vec![context.f64_type().into(); params.len()];
            let fn_type = context.f64_type().fn_type(&param_types, false);
            let fn_val = module.add_function(name, fn_type, None);
            let entry = context.append_basic_block(fn_val, "entry");
            let prev_block = builder.get_insert_block();
            builder.position_at_end(entry);

            let mut local_env = env.clone();
            for (i, param_name) in params.iter().enumerate() {
                let param = fn_val
                .get_nth_param(i as u32)
                .ok_or_else(|| format!("Parameter {} not found in function {}", i, name))?;
                let alloca = builder.build_alloca(context.f64_type(), param_name);
                builder.build_store(alloca, param);
                local_env.insert(param_name.clone(), alloca);
            }

            for stmt in body {
                build_llvm_node(stmt, builder, context, module, fn_val, &mut local_env)?;
            }

            if builder.get_insert_block().map_or(true, |bb| bb.get_terminator().is_none()) {
                builder.build_return(Some(&context.f64_type().const_float(0.0)));
            }

            if let Some(prev) = prev_block {
                builder.position_at_end(prev);
            }
        }
        AstNode::ForLoop(var, iter, body) => {
            let start = context.f64_type().const_float(0.0);
            let end = build_llvm_expression(iter, builder, context, module, env)?.into_float_value();
            let var_alloca = builder.build_alloca(context.f64_type(), var);
            builder.build_store(var_alloca, start);

            let loop_bb = context.append_basic_block(current_fn, "loop");
            let body_bb = context.append_basic_block(current_fn, "body");
            let inc_bb = context.append_basic_block(current_fn, "inc");
            let after_bb = context.append_basic_block(current_fn, "after");

            builder.build_unconditional_branch(loop_bb);
            builder.position_at_end(loop_bb);

            let var_val = builder.build_load(context.f64_type(), var_alloca, var).into_float_value();
            let cond = builder.build_float_compare(FloatPredicate::OLT, var_val, end, "cmp");
            builder.build_conditional_branch(cond, body_bb, after_bb);

            builder.position_at_end(body_bb);
            let mut loop_env = env.clone();
            loop_env.insert(var.clone(), var_alloca);
            for stmt in body {
                build_llvm_node(stmt, builder, context, module, current_fn, &mut loop_env)?;
            }
            builder.build_unconditional_branch(inc_bb);

            builder.position_at_end(inc_bb);
            let var_val = builder.build_load(context.f64_type(), var_alloca, var).into_float_value();
            let step = context.f64_type().const_float(1.0);
            let next = builder.build_float_add(var_val, step, "inc");
            builder.build_store(var_alloca, next);
            builder.build_unconditional_branch(loop_bb);

            builder.position_at_end(after_bb);
        }
        AstNode::WhileLoop(cond, body) => {
            let loop_bb = context.append_basic_block(current_fn, "loop");
            let body_bb = context.append_basic_block(current_fn, "body");
            let after_bb = context.append_basic_block(current_fn, "after");

            builder.build_unconditional_branch(loop_bb);
            builder.position_at_end(loop_bb);

            let cond_val = build_llvm_expression(cond, builder, context, module, env)?.into_float_value();
            let cond_bool = builder.build_float_compare(FloatPredicate::ONE, cond_val, context.f64_type().const_zero(), "cmp");
            builder.build_conditional_branch(cond_bool, body_bb, after_bb);

            builder.position_at_end(body_bb);
            for stmt in body {
                build_llvm_node(stmt, builder, context, module, current_fn, env)?;
            }
            builder.build_unconditional_branch(loop_bb);

            builder.position_at_end(after_bb);
        }
        AstNode::If(cond, body, else_ifs, else_body) => {
            let cond_val = build_llvm_expression(cond, builder, context, module, env)?.into_float_value();
            let cond_bool = builder.build_float_compare(FloatPredicate::ONE, cond_val, context.f64_type().const_zero(), "cmp");
            let then_bb = context.append_basic_block(current_fn, "then");
            let mut next_bb = context.append_basic_block(current_fn, "else");
            let cont_bb = context.append_basic_block(current_fn, "cont");

            builder.build_conditional_branch(cond_bool, then_bb, next_bb);
            builder.position_at_end(then_bb);
            for stmt in body {
                build_llvm_node(stmt, builder, context, module, current_fn, env)?;
            }
            builder.build_unconditional_branch(cont_bb);

            builder.position_at_end(next_bb);
            for (ei_cond, ei_body) in else_ifs {
                let ei_cond_val = build_llvm_expression(ei_cond, builder, context, module, env)?.into_float_value();
                let ei_cond_bool = builder.build_float_compare(FloatPredicate::ONE, ei_cond_val, context.f64_type().const_zero(), "cmp");
                let ei_then_bb = context.append_basic_block(current_fn, "else_if_then");
                let ei_else_bb = context.append_basic_block(current_fn, "else_if_else");

                builder.build_conditional_branch(ei_cond_bool, ei_then_bb, ei_else_bb);
                builder.position_at_end(ei_then_bb);
                for stmt in ei_body {
                    build_llvm_node(stmt, builder, context, module, current_fn, env)?;
                }
                builder.build_unconditional_branch(cont_bb);

                next_bb = ei_else_bb;
                builder.position_at_end(next_bb);
            }
            if let Some(eb) = else_body {
                for stmt in eb {
                    build_llvm_node(stmt, builder, context, module, current_fn, env)?;
                }
            }
            builder.build_unconditional_branch(cont_bb);

            builder.position_at_end(cont_bb);
        }
        AstNode::Write(expr) => {
            let printf = get_printf(module, context);
            let val = build_llvm_expression(expr, builder, context, module, env)?;
            let fmt_str = match &**expr {
                AstNode::StringLit(s) => builder.build_global_string_ptr(&format!("{}\n", s), "fmt_str"),
                AstNode::NumberLit(_) => builder.build_global_string_ptr("%f\n", "fmt_num"),
                AstNode::BoolLit(b) => builder.build_global_string_ptr(if *b { "true\n" } else { "false\n" }, "fmt_bool"),
                _ => builder.build_global_string_ptr("%f\n", "fmt"),
            };
            let args: Vec<BasicMetadataValueEnum> = vec![fmt_str.as_basic_value_enum().into(), val.into()];
            builder.build_call(printf, &args, "printf");
        }
        AstNode::Return(expr) => {
            if let Some(e) = expr {
                let val = build_llvm_expression(e, builder, context, module, env)?.into_float_value();
                builder.build_return(Some(&val));
            } else {
                builder.build_return(Some(&context.f64_type().const_zero()));
            }
        }
        AstNode::Add(left, right) => {
            let lhs = build_llvm_expression(left, builder, context, module, env)?.into_float_value();
            let rhs = build_llvm_expression(right, builder, context, module, env)?.into_float_value();
            builder.build_float_add(lhs, rhs, "add");
        }
        AstNode::Mul(left, right) => {
            let lhs = build_llvm_expression(left, builder, context, module, env)?.into_float_value();
            let rhs = build_llvm_expression(right, builder, context, module, env)?.into_float_value();
            builder.build_float_mul(lhs, rhs, "mul");
        }
        AstNode::Binary(left, op, right) => {
            let lhs = build_llvm_expression(left, builder, context, module, env)?.into_float_value();
            let rhs = build_llvm_expression(right, builder, context, module, env)?.into_float_value();
            match op.as_str() {
                "+" => builder.build_float_add(lhs, rhs, "add"),
                "-" => builder.build_float_sub(lhs, rhs, "sub"),
                "*" => builder.build_float_mul(lhs, rhs, "mul"),
                "/" => builder.build_float_div(lhs, rhs, "div"),
                "==" | "eq" => {
                    let cmp = builder.build_float_compare(FloatPredicate::OEQ, lhs, rhs, "eq");
                    builder.build_unsigned_int_to_float(cmp, context.f64_type(), "eq_float")
                }
                "!=" | "ne" => {
                    let cmp = builder.build_float_compare(FloatPredicate::ONE, lhs, rhs, "ne");
                    builder.build_unsigned_int_to_float(cmp, context.f64_type(), "ne_float")
                }
                "<" | "lt" => {
                    let cmp = builder.build_float_compare(FloatPredicate::OLT, lhs, rhs, "lt");
                    builder.build_unsigned_int_to_float(cmp, context.f64_type(), "lt_float")
                }
                ">" | "gt" => {
                    let cmp = builder.build_float_compare(FloatPredicate::OGT, lhs, rhs, "gt");
                    builder.build_unsigned_int_to_float(cmp, context.f64_type(), "gt_float")
                }
                "<=" | "le" => {
                    let cmp = builder.build_float_compare(FloatPredicate::OLE, lhs, rhs, "le");
                    builder.build_unsigned_int_to_float(cmp, context.f64_type(), "le_float")
                }
                ">=" | "ge" => {
                    let cmp = builder.build_float_compare(FloatPredicate::OGE, lhs, rhs, "ge");
                    builder.build_unsigned_int_to_float(cmp, context.f64_type(), "ge_float")
                }
                "and" | "&&" => {
                    let lhs_bool = builder.build_float_compare(FloatPredicate::ONE, lhs, context.f64_type().const_zero(), "lhs_bool");
                    let rhs_bool = builder.build_float_compare(FloatPredicate::ONE, rhs, context.f64_type().const_zero(), "rhs_bool");
                    let and = builder.build_and(lhs_bool, rhs_bool, "and");
                    builder.build_unsigned_int_to_float(and, context.f64_type(), "and_float")
                }
                "or" | "||" => {
                    let lhs_bool = builder.build_float_compare(FloatPredicate::ONE, lhs, context.f64_type().const_zero(), "lhs_bool");
                    let rhs_bool = builder.build_float_compare(FloatPredicate::ONE, rhs, context.f64_type().const_zero(), "rhs_bool");
                    let or = builder.build_or(lhs_bool, rhs_bool, "or");
                    builder.build_unsigned_int_to_float(or, context.f64_type(), "or_float")
                }
                _ => return Err(format!("Unsupported binary operator: {}", op)),
            };
        }
        AstNode::Unary(op, expr) => {
            let val = build_llvm_expression(expr, builder, context, module, env)?.into_float_value();
            match op.as_str() {
                "-" => builder.build_float_neg(val, "neg"),
                "not" => {
                    let zero = context.f64_type().const_zero();
                    let cmp = builder.build_float_compare(FloatPredicate::OEQ, val, zero, "cmp");
                    let not = builder.build_not(cmp, "not");
                    builder.build_unsigned_int_to_float(not, context.f64_type(), "not_float")
                }
                _ => return Err(format!("Unsupported unary operator: {}", op)),
            };
        }
        AstNode::Call(name, args) => {
            if let Some(fn_val) = module.get_function(name) {
                let arg_vals: Vec<BasicMetadataValueEnum> = args
                .iter()
                .map(|arg| build_llvm_expression(arg, builder, context, module, env).map(|v| v.into()))
                .collect::<Result<Vec<_>, _>>()?;
                builder
                .build_call(fn_val, &arg_vals, "call")
                .try_as_basic_value()
                .left()
                .ok_or_else(|| format!("Function call {} returned no value", name))?
                .into_float_value();
            } else {
                return Err(format!("Undefined function: {}", name));
            }
        }
        _ => return Err("Unsupported AST node in build_llvm_node".to_string()),
    }
    Ok(())
}

fn build_llvm_expression<'ctx>(
    expr: &AstNode,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    module: &Module<'ctx>,
    env: &HashMap<String, PointerValue<'ctx>>,
) -> Result<BasicValueEnum<'ctx>, String> {
    match expr {
        AstNode::StringLit(s) => Ok(builder.build_global_string_ptr(s, "str").as_basic_value_enum()),
        AstNode::NumberLit(num) => Ok(context.f64_type().const_float(*num).into()),
        AstNode::BoolLit(b) => Ok(context.f64_type().const_float(if *b { 1.0 } else { 0.0 }).into()),
        AstNode::Ident(name) => {
            if let Some(&ptr) = env.get(name) {
                Ok(builder.build_load(context.f64_type(), ptr, name))
            } else {
                Err(format!("Undefined variable: {}", name))
            }
        }
        AstNode::Call(name, args) => {
            if let Some(fn_val) = module.get_function(name) {
                let arg_vals: Vec<BasicMetadataValueEnum> = args
                .iter()
                .map(|arg| build_llvm_expression(arg, builder, context, module, env).map(|v| v.into()))
                .collect::<Result<Vec<_>, _>>()?;
                Ok(builder
                .build_call(fn_val, &arg_vals, "call")
                .try_as_basic_value()
                .left()
                .ok_or_else(|| format!("Function call {} returned no value", name))?)
            } else {
                Err(format!("Undefined function: {}", name))
            }
        }
        AstNode::Binary(left, op, right) => {
            let lhs = build_llvm_expression(left, builder, context, module, env)?.into_float_value();
            let rhs = build_llvm_expression(right, builder, context, module, env)?.into_float_value();
            let res = match op.as_str() {
                "+" => builder.build_float_add(lhs, rhs, "add").into(),
                "-" => builder.build_float_sub(lhs, rhs, "sub").into(),
                "*" => builder.build_float_mul(lhs, rhs, "mul").into(),
                "/" => builder.build_float_div(lhs, rhs, "div").into(),
                "==" | "eq" => {
                    let cmp = builder.build_float_compare(FloatPredicate::OEQ, lhs, rhs, "eq");
                    builder.build_unsigned_int_to_float(cmp, context.f64_type(), "eq_float").into()
                }
                "!=" | "ne" => {
                    let cmp = builder.build_float_compare(FloatPredicate::ONE, lhs, rhs, "ne");
                    builder.build_unsigned_int_to_float(cmp, context.f64_type(), "ne_float").into()
                }
                "<" | "lt" => {
                    let cmp = builder.build_float_compare(FloatPredicate::OLT, lhs, rhs, "lt");
                    builder.build_unsigned_int_to_float(cmp, context.f64_type(), "lt_float").into()
                }
                ">" | "gt" => {
                    let cmp = builder.build_float_compare(FloatPredicate::OGT, lhs, rhs, "gt");
                    builder.build_unsigned_int_to_float(cmp, context.f64_type(), "gt_float").into()
                }
                "<=" | "le" => {
                    let cmp = builder.build_float_compare(FloatPredicate::OLE, lhs, rhs, "le");
                    builder.build_unsigned_int_to_float(cmp, context.f64_type(), "le_float").into()
                }
                ">=" | "ge" => {
                    let cmp = builder.build_float_compare(FloatPredicate::OGE, lhs, rhs, "ge");
                    builder.build_unsigned_int_to_float(cmp, context.f64_type(), "ge_float").into()
                }
                "and" | "&&" => {
                    let lhs_bool = builder.build_float_compare(FloatPredicate::ONE, lhs, context.f64_type().const_zero(), "lhs_bool");
                    let rhs_bool = builder.build_float_compare(FloatPredicate::ONE, rhs, context.f64_type().const_zero(), "rhs_bool");
                    let and = builder.build_and(lhs_bool, rhs_bool, "and");
                    builder.build_unsigned_int_to_float(and, context.f64_type(), "and_float").into()
                }
                "or" | "||" => {
                    let lhs_bool = builder.build_float_compare(FloatPredicate::ONE, lhs, context.f64_type().const_zero(), "lhs_bool");
                    let rhs_bool = builder.build_float_compare(FloatPredicate::ONE, rhs, context.f64_type().const_zero(), "rhs_bool");
                    let or = builder.build_or(lhs_bool, rhs_bool, "or");
                    builder.build_unsigned_int_to_float(or, context.f64_type(), "or_float").into()
                }
                _ => return Err(format!("Unsupported binary operator: {}", op)),
            };
            Ok(res)
        }
        AstNode::Unary(op, expr) => {
            let val = build_llvm_expression(expr, builder, context, module, env)?.into_float_value();
            let res = match op.as_str() {
                "-" => builder.build_float_neg(val, "neg").into(),
                "not" => {
                    let zero = context.f64_type().const_zero();
                    let cmp = builder.build_float_compare(FloatPredicate::OEQ, val, zero, "cmp");
                    let not = builder.build_not(cmp, "not");
                    builder.build_unsigned_int_to_float(not, context.f64_type(), "not_float").into()
                }
                _ => return Err(format!("Unsupported unary operator: {}", op)),
            };
            Ok(res)
        }
        _ => Err("Unsupported expression in build_llvm_expression".to_string()),
    }
}

fn get_printf<'ctx>(module: &Module<'ctx>, context: &'ctx Context) -> FunctionValue<'ctx> {
    let i8p_type = context.i8_type().ptr_type(AddressSpace::default());
    let printf_type = context.i32_type().fn_type(&[i8p_type.into()], true);
    module.get_function("printf").unwrap_or_else(|| module.add_function("printf", printf_type, None))
}
