use crate::ast::AstNode;
use cranelift_codegen::isa::{self, CallConv};
use cranelift_codegen::settings::{self, Configurable};
use cranelift_codegen::Context as CodegenContext;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_module::{DataContext, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::process::Command;
use target_lexicon::Triple;
use cranelift_codegen::ir::{types, AbiParam, Value, StackSlot, StackSlotData, StackSlotKind, condcodes::FloatCC, Function, UserFuncName, InstBuilder};
use cranelift_module::{FuncId, DataId, FuncOrDataId};
use std::str::FromStr;

#[derive(Debug)]
enum CraneliftValue {
    Float(Value),
    Pointer(Value),
}

impl CraneliftValue {
    fn expect_float(self) -> Value {
        if let CraneliftValue::Float(v) = self {
            v
        } else {
            panic!("Expected float value");
        }
    }

    fn expect_pointer(self) -> Value {
        if let CraneliftValue::Pointer(v) = self {
            v
        } else {
            panic!("Expected pointer value");
        }
    }

    fn as_value(self) -> Value {
        match self {
            CraneliftValue::Float(v) => v,
            CraneliftValue::Pointer(v) => v,
        }
    }
}

pub fn generate_cranelift(ast: &AstNode, project_name: &str, release: bool, target: &str) -> Result<(), String> {
    let mut flag_builder = settings::builder();
    flag_builder.set("opt_level", if release { "speed" } else { "none" }).map_err(|e| format!("{}", e))?;

    let triple = Triple::from_str(target).map_err(|e| format!("Invalid target: {}", e))?;
    let isa_builder = isa::lookup(triple.clone()).map_err(|e| format!("{}", e))?;
    let isa = isa_builder.finish(settings::Flags::new(flag_builder)).map_err(|e| format!("{}", e))?;

    let builder = ObjectBuilder::new(isa, project_name.as_bytes().to_vec(), cranelift_module::default_libcall_names()).map_err(|e| format!("{}", e))?;
    let mut module = ObjectModule::new(builder);

    let pointer_type = module.isa().pointer_type();

    // Declare printf
    let mut printf_sig = module.make_signature();
    printf_sig.params.push(AbiParam::new(pointer_type));
    printf_sig.params.push(AbiParam::new(types::F64));
    printf_sig.returns.push(AbiParam::new(types::I32));
    printf_sig.call_conv = CallConv::triple_default(&triple);
    let printf = module.declare_function("printf", Linkage::Import, &printf_sig).map_err(|e| format!("{}", e))?;

    // Main function
    let mut main_sig = module.make_signature();
    main_sig.returns.push(AbiParam::new(types::I32));
    main_sig.call_conv = CallConv::triple_default(&triple);
    let main = module.declare_function("main", Linkage::Export, &main_sig).map_err(|e| format!("{}", e))?;

    let mut func = Function::with_name_signature(UserFuncName::testcase("main"), main_sig.clone());
    let mut func_ctx = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut func, &mut func_ctx);

    let entry = builder.create_block();
    builder.switch_to_block(entry);
    builder.seal_block(entry);

    let mut env: HashMap<String, StackSlot> = HashMap::new();
    let mut var_count = 0;
    let mut data_count = 0;

    build_cranelift_node(ast, &mut builder, &mut module, main, &mut env, &mut var_count, &mut data_count, printf)?;

    let zero = builder.ins().iconst(types::I32, 0);
    builder.ins().return_(&[zero]);

    builder.finalize();

    let mut ctx = CodegenContext::new();
    ctx.func = func;
    module.define_function(main, &mut ctx).map_err(|e| format!("{}", e))?;

    let product = module.finish();
    let obj = product.emit().map_err(|e| format!("{}", e))?;
    let obj_path = format!("{}.o", project_name);
    let mut file = File::create(&obj_path).map_err(|e| format!("Failed to create object file: {}", e))?;
    file.write_all(&obj).map_err(|e| format!("Failed to write object file: {}", e))?;

    // Link
    let bin_path = if target.contains("windows") { format!("{}.exe", project_name) } else { project_name.to_string() };
    if target.contains("linux") {
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
    } else if target.contains("windows") {
        let mut link_cmd = Command::new("link.exe");
        link_cmd.arg(format!("/OUT:{}", bin_path));
        link_cmd.arg(&obj_path);
        link_cmd.arg("kernel32.lib");
        link_cmd.arg("user32.lib");
        link_cmd.arg("gdi32.lib");
        link_cmd.arg("msvcrt.lib");
        let output = link_cmd.output().map_err(|e| format!("Failed to execute linker: {}", e))?;
        if !output.status.success() {
            return Err(format!("Linker failed: {}", String::from_utf8_lossy(&output.stderr)));
        }
    } else {
        return Err("Unsupported target for linking".to_string());
    }

    Ok(())
}

fn create_string_constant(module: &mut ObjectModule, s: &str, data_count: &mut usize) -> Result<DataId, String> {
    let name = format!("const_{}", *data_count);
    *data_count += 1;
    let mut bytes = s.as_bytes().to_vec();
    if !bytes.ends_with(&[0]) {
        bytes.push(0);
    }
    let mut data_ctx = DataContext::new();
    data_ctx.define(bytes.into_boxed_slice());
    let data_id = module.declare_data(&name, Linkage::Local, true, false).map_err(|e| format!("{}", e))?;
    module.define_data(data_id, &data_ctx).map_err(|e| format!("{}", e))?;
    Ok(data_id)
}

fn build_cranelift_node(
    node: &AstNode,
    builder: &mut FunctionBuilder,
    module: &mut ObjectModule,
    current_fn: FuncId,
    env: &mut HashMap<String, StackSlot>,
    var_count: &mut usize,
    data_count: &mut usize,
    printf: FuncId,
) -> Result<(), String> {
    match node {
        AstNode::Program(nodes) => {
            for n in nodes {
                build_cranelift_node(n, builder, module, current_fn, env, var_count, data_count, printf)?;
            }
        }
        AstNode::Translation(_, _) | AstNode::Dependency(_) | AstNode::Import(_) | AstNode::Comment(_) => {}
        AstNode::VariableDecl(name, expr) | AstNode::Assignment(name, expr) => {
            let val = build_cranelift_expression(expr, builder, module, env, var_count, data_count, printf)?.expect_float();
            let stack_slot = builder.create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, types::F64.bytes()));
            builder.ins().stack_store(val, stack_slot, 0);
            env.insert(name.clone(), stack_slot);
        }
        AstNode::FunctionDef(name, params, body) => {
            let mut sig = module.make_signature();
            for _ in params {
                sig.params.push(AbiParam::new(types::F64));
            }
            sig.returns.push(AbiParam::new(types::F64));
            sig.call_conv = CallConv::triple_default(&module.isa().triple());
            let fn_id = module.declare_function(name, Linkage::Local, &sig).map_err(|e| format!("{}", e))?;

            let mut func = Function::with_name_signature(UserFuncName::testcase(name), sig.clone());
            let mut fn_ctx = FunctionBuilderContext::new();
            let mut local_builder = FunctionBuilder::new(&mut func, &mut fn_ctx);
            let entry = local_builder.create_block();
            local_builder.append_block_params_for_function_params(entry);
            local_builder.switch_to_block(entry);
            local_builder.seal_block(entry);

            let mut local_env = env.clone();
            for (i, param_name) in params.iter().enumerate() {
                let param = local_builder.block_params(entry)[i];
                let stack_slot = local_builder.create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, types::F64.bytes()));
                local_builder.ins().stack_store(param, stack_slot, 0);
                local_env.insert(param_name.clone(), stack_slot);
            }

            for stmt in body {
                build_cranelift_node(stmt, &mut local_builder, module, fn_id, &mut local_env, var_count, data_count, printf)?;
            }

            if !local_builder.is_unreachable() {
                let zero = local_builder.ins().f64const(0.0);
                local_builder.ins().return_(&[zero]);
            }

            local_builder.finalize();

            let mut ctx = CodegenContext::new();
            ctx.func = func;
            module.define_function(fn_id, &mut ctx).map_err(|e| format!("{}", e))?;
        }
        AstNode::ForLoop(var, iter, body) => {
            let start = builder.ins().f64const(0.0);
            let end = build_cranelift_expression(iter, builder, module, env, var_count, data_count, printf)?.expect_float();
            let var_slot = builder.create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, types::F64.bytes()));
            builder.ins().stack_store(start, var_slot, 0);

            let loop_bb = builder.create_block();
            let body_bb = builder.create_block();
            let inc_bb = builder.create_block();
            let after_bb = builder.create_block();

            builder.ins().jump(loop_bb, &[]);
            builder.switch_to_block(loop_bb);
            builder.seal_block(loop_bb);

            let var_val = builder.ins().stack_load(types::F64, var_slot, 0);
            let cond = builder.ins().fcmp(FloatCC::LessThan, var_val, end);
            builder.ins().brnz(cond, body_bb, &[]);
            builder.ins().jump(after_bb, &[]);

            builder.switch_to_block(body_bb);
            let mut loop_env = env.clone();
            loop_env.insert(var.clone(), var_slot);
            for stmt in body {
                build_cranelift_node(stmt, builder, module, current_fn, &mut loop_env, var_count, data_count, printf)?;
            }
            builder.ins().jump(inc_bb, &[]);
            builder.seal_block(body_bb);

            builder.switch_to_block(inc_bb);
            let var_val = builder.ins().stack_load(types::F64, var_slot, 0);
            let step = builder.ins().f64const(1.0);
            let next = builder.ins().fadd(var_val, step);
            builder.ins().stack_store(next, var_slot, 0);
            builder.ins().jump(loop_bb, &[]);
            builder.seal_block(inc_bb);

            builder.switch_to_block(after_bb);
            builder.seal_block(after_bb);
        }
        AstNode::WhileLoop(cond, body) => {
            let loop_bb = builder.create_block();
            let body_bb = builder.create_block();
            let after_bb = builder.create_block();

            builder.ins().jump(loop_bb, &[]);
            builder.switch_to_block(loop_bb);
            let cond_val = build_cranelift_expression(cond, builder, module, env, var_count, data_count, printf)?.expect_float();
            let zero = builder.ins().f64const(0.0);
            let cond_bool = builder.ins().fcmp(FloatCC::NotEqual, cond_val, zero);
            builder.ins().brnz(cond_bool, body_bb, &[]);
            builder.ins().jump(after_bb, &[]);
            builder.seal_block(loop_bb);

            builder.switch_to_block(body_bb);
            for stmt in body {
                build_cranelift_node(stmt, builder, module, current_fn, env, var_count, data_count, printf)?;
            }
            builder.ins().jump(loop_bb, &[]);
            builder.seal_block(body_bb);

            builder.switch_to_block(after_bb);
            builder.seal_block(after_bb);
        }
        AstNode::If(cond, body, else_ifs, else_body) => {
            let cond_val = build_cranelift_expression(cond, builder, module, env, var_count, data_count, printf)?.expect_float();
            let zero = builder.ins().f64const(0.0);
            let cond_bool = builder.ins().fcmp(FloatCC::NotEqual, cond_val, zero);
            let then_bb = builder.create_block();
            let mut next_bb = builder.create_block();
            let cont_bb = builder.create_block();

            builder.ins().brnz(cond_bool, then_bb, &[]);
            builder.ins().jump(next_bb, &[]);
            builder.seal_block(builder.current_block().unwrap());

            builder.switch_to_block(then_bb);
            for stmt in body {
                build_cranelift_node(stmt, builder, module, current_fn, env, var_count, data_count, printf)?;
            }
            builder.ins().jump(cont_bb, &[]);
            builder.seal_block(then_bb);

            builder.switch_to_block(next_bb);
            for (ei_cond, ei_body) in else_ifs {
                let ei_cond_val = build_cranelift_expression(ei_cond, builder, module, env, var_count, data_count, printf)?.expect_float();
                let zero = builder.ins().f64const(0.0);
                let ei_cond_bool = builder.ins().fcmp(FloatCC::NotEqual, ei_cond_val, zero);
                let ei_then_bb = builder.create_block();
                let ei_else_bb = builder.create_block();

                builder.ins().brnz(ei_cond_bool, ei_then_bb, &[]);
                builder.ins().jump(ei_else_bb, &[]);
                builder.seal_block(builder.current_block().unwrap());

                builder.switch_to_block(ei_then_bb);
                for stmt in ei_body {
                    build_cranelift_node(stmt, builder, module, current_fn, env, var_count, data_count, printf)?;
                }
                builder.ins().jump(cont_bb, &[]);
                builder.seal_block(ei_then_bb);

                next_bb = ei_else_bb;
                builder.switch_to_block(next_bb);
            }
            if let Some(eb) = else_body {
                for stmt in eb {
                    build_cranelift_node(stmt, builder, module, current_fn, env, var_count, data_count, printf)?;
                }
            }
            builder.ins().jump(cont_bb, &[]);
            builder.seal_block(next_bb);

            builder.switch_to_block(cont_bb);
            builder.seal_block(cont_bb);
        }
        AstNode::Write(expr) => {
            let val = build_cranelift_expression(expr, builder, module, env, var_count, data_count, printf)?;
            let fmt_str = match expr {
                AstNode::StringLit(_) => "%s\n",
                AstNode::NumberLit(_) => "%f\n",
                AstNode::BoolLit(_) => "%s\n",
                _ => "%f\n",
            };
            let fmt_str_id = create_string_constant(module, fmt_str, data_count)?;
            let gv = module.declare_data_in_func(fmt_str_id, builder.func);
            let fmt_ptr = builder.ins().global_value(module.isa().pointer_type(), gv);
            let args = if matches!(expr, AstNode::StringLit(_) | AstNode::BoolLit(_)) {
                let str_val = match expr {
                    AstNode::StringLit(ref s) => s.as_str(),
                    AstNode::BoolLit(b) => if *b { "true" } else { "false" },
                    _ => unreachable!(),
                };
                let str_id = create_string_constant(module, str_val, data_count)?;
                let str_gv = module.declare_data_in_func(str_id, builder.func);
                let str_ptr = builder.ins().global_value(module.isa().pointer_type(), str_gv);
                vec![fmt_ptr, str_ptr]
            } else {
                vec![fmt_ptr, val.expect_float()]
            };
            let callee = module.declare_func_in_func(printf, builder.func);
            builder.ins().call(callee, &args);
        }
        AstNode::Return(expr) => {
            if let Some(e) = expr {
                let val = build_cranelift_expression(e, builder, module, env, var_count, data_count, printf)?.expect_float();
                builder.ins().return_(&[val]);
            } else {
                let zero = builder.ins().f64const(0.0);
                builder.ins().return_(&[zero]);
            }
        }
        _ => return Err("Unsupported AST node in build_cranelift_node".to_string()),
    }
    Ok(())
}

fn build_cranelift_expression(
    expr: &AstNode,
    builder: &mut FunctionBuilder,
    module: &mut ObjectModule,
    env: &HashMap<String, StackSlot>,
    var_count: &mut usize,
    data_count: &mut usize,
    printf: FuncId,
) -> Result<CraneliftValue, String> {
    match expr {
        AstNode::StringLit(s) => {
            let data_id = create_string_constant(module, s, data_count)?;
            let gv = module.declare_data_in_func(data_id, builder.func);
            let ptr = builder.ins().global_value(module.isa().pointer_type(), gv);
            Ok(CraneliftValue::Pointer(ptr))
        }
        AstNode::NumberLit(num) => {
            let val = builder.ins().f64const(*num);
            Ok(CraneliftValue::Float(val))
        }
        AstNode::BoolLit(b) => {
            let val = builder.ins().f64const(if *b { 1.0 } else { 0.0 });
            Ok(CraneliftValue::Float(val))
        }
        AstNode::Ident(name) => {
            if let Some(&slot) = env.get(name) {
                let val = builder.ins().stack_load(types::F64, slot, 0);
                Ok(CraneliftValue::Float(val))
            } else {
                Err(format!("Undefined variable: {}", name))
            }
        }
        AstNode::Call(name, args) => {
            let ext_id = module.get_name(name).ok_or(format!("Undefined function: {}", name))?;
            let func_id = match ext_id {
                FuncOrDataId::Func(f) => f,
                _ => return Err("Not a function".to_string()),
            };
            let callee = module.declare_func_in_func(func_id, builder.func);
            let arg_vals: Vec<Value> = args
            .iter()
            .map(|arg| build_cranelift_expression(arg, builder, module, env, var_count, data_count, printf).map(|v| v.expect_float()))
            .collect::<Result<Vec<_>, _>>()?;
            let call = builder.ins().call(callee, &arg_vals);
            let res = builder.inst_results(call)[0];
            Ok(CraneliftValue::Float(res))
        }
        AstNode::Binary(left, op, right) => {
            let lhs = build_cranelift_expression(left, builder, module, env, var_count, data_count, printf)?.expect_float();
            let rhs = build_cranelift_expression(right, builder, module, env, var_count, data_count, printf)?.expect_float();
            let res = match op.as_str() {
                "+" => builder.ins().fadd(lhs, rhs),
                "-" => builder.ins().fsub(lhs, rhs),
                "*" => builder.ins().fmul(lhs, rhs),
                "/" => builder.ins().fdiv(lhs, rhs),
                "==" | "eq" => {
                    let cmp = builder.ins().fcmp(FloatCC::Equal, lhs, rhs);
                    let ext = builder.ins().uextend(types::I64, cmp);
                    builder.ins().fcvt_from_uint(types::F64, ext)
                }
                "!=" | "ne" => {
                    let cmp = builder.ins().fcmp(FloatCC::NotEqual, lhs, rhs);
                    let ext = builder.ins().uextend(types::I64, cmp);
                    builder.ins().fcvt_from_uint(types::F64, ext)
                }
                "<" | "lt" => {
                    let cmp = builder.ins().fcmp(FloatCC::LessThan, lhs, rhs);
                    let ext = builder.ins().uextend(types::I64, cmp);
                    builder.ins().fcvt_from_uint(types::F64, ext)
                }
                ">" | "gt" => {
                    let cmp = builder.ins().fcmp(FloatCC::GreaterThan, lhs, rhs);
                    let ext = builder.ins().uextend(types::I64, cmp);
                    builder.ins().fcvt_from_uint(types::F64, ext)
                }
                "<=" | "le" => {
                    let cmp = builder.ins().fcmp(FloatCC::LessThanOrEqual, lhs, rhs);
                    let ext = builder.ins().uextend(types::I64, cmp);
                    builder.ins().fcvt_from_uint(types::F64, ext)
                }
                ">=" | "ge" => {
                    let cmp = builder.ins().fcmp(FloatCC::GreaterThanOrEqual, lhs, rhs);
                    let ext = builder.ins().uextend(types::I64, cmp);
                    builder.ins().fcvt_from_uint(types::F64, ext)
                }
                "and" | "&&" => {
                    let zero = builder.ins().f64const(0.0);
                    let lhs_bool = builder.ins().fcmp(FloatCC::NotEqual, lhs, zero);
                    let rhs_bool = builder.ins().fcmp(FloatCC::NotEqual, rhs, zero);
                    let and = builder.ins().band(lhs_bool, rhs_bool);
                    let ext = builder.ins().uextend(types::I64, and);
                    builder.ins().fcvt_from_uint(types::F64, ext)
                }
                "or" | "||" => {
                    let zero = builder.ins().f64const(0.0);
                    let lhs_bool = builder.ins().fcmp(FloatCC::NotEqual, lhs, zero);
                    let rhs_bool = builder.ins().fcmp(FloatCC::NotEqual, rhs, zero);
                    let or = builder.ins().bor(lhs_bool, rhs_bool);
                    let ext = builder.ins().uextend(types::I64, or);
                    builder.ins().fcvt_from_uint(types::F64, ext)
                }
                _ => return Err(format!("Unsupported binary operator: {}", op)),
            };
            Ok(CraneliftValue::Float(res))
        }
        AstNode::Unary(op, expr) => {
            let val = build_cranelift_expression(expr, builder, module, env, var_count, data_count, printf)?.expect_float();
            let res = match op.as_str() {
                "-" => builder.ins().fneg(val),
                "not" => {
                    let zero = builder.ins().f64const(0.0);
                    let cmp = builder.ins().fcmp(FloatCC::Equal, val, zero);
                    let not = builder.ins().bnot(cmp);
                    let ext = builder.ins().uextend(types::I64, not);
                    builder.ins().fcvt_from_uint(types::F64, ext)
                }
                _ => return Err(format!("Unsupported unary operator: {}", op)),
            };
            Ok(CraneliftValue::Float(res))
        }
        _ => Err("Unsupported expression in build_cranelift_expression".to_string()),
    }
}
