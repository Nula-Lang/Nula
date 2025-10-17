// src/codegen.rs - Code generation

use std::collections::HashMap;

use cranelift::prelude::*;
use cranelift_codegen::ir::{self, AbiParam, InstBuilder, MemFlags};
use cranelift_codegen::isa::CallConv;
use cranelift_codegen::Context as CodegenContext;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_module::{DataContext, FuncId, Linkage, Module};

use crate::ast::Ast;

pub struct CodeGen<'a, 'b> {
    pub module: &'a mut dyn Module,
    pub builder: &'b mut FunctionBuilder<'a>,
    variables: HashMap<String, Variable>,
    var_index: u32,
    functions: HashMap<String, FuncId>,
    printf: FuncId,
    // Add for arrays: array_vars: HashMap<String, (Value, u32)> // ptr, size
}

impl<'a, 'b> CodeGen<'a, 'b> {
    pub fn new(module: &'a mut dyn Module, builder: &'b mut FunctionBuilder<'a>, printf: FuncId) -> Self {
        CodeGen {
            module,
            builder,
            variables: HashMap::new(),
            var_index: 0,
            functions: HashMap::new(),
            printf,
        }
    }

    pub fn gen_ast(&mut self, ast: &Ast) -> Value {
        match ast {
            Ast::Literal(val) => self.builder.ins().fconst(types::F64, *val),
            Ast::StrLit(s) => {
                let mut data_ctx = DataContext::new();
                data_ctx.define(format!("{}\n\0", s).into_bytes().into_boxed_slice());
                let data_id = self.module.declare_data(&format!("str_{}", self.var_index), Linkage::Local, true, false).unwrap();
                self.var_index += 1;
                self.module.define_data(data_id, &data_ctx).unwrap();
                self.builder.ins().global_value(types::I64, data_id)
            }
            Ast::Var(name) => self.builder.use_var(*self.variables.get(name).expect("Undefined var")),
            Ast::BinOp(op, left, right) => {
                let l = self.gen_ast(left);
                let r = self.gen_ast(right);
                match op.as_str() {
                    "+" => self.builder.ins().fadd(l, r),
                    "-" => self.builder.ins().fsub(l, r),
                    "*" => self.builder.ins().fmul(l, r),
                    "/" => self.builder.ins().fdiv(l, r),
                    "^" => {
                        // For pow, declare powf
                        let mut sig = self.module.make_signature();
                        sig.params.push(AbiParam::new(types::F64));
                        sig.params.push(AbiParam::new(types::F64));
                        sig.returns.push(AbiParam::new(types::F64));
                        sig.call_conv = CallConv::C;
                        let powf = self.module.declare_function("powf", Linkage::Import, &sig).unwrap();
                        self.builder.ins().call(powf, &[l, r])[0]
                    }
                    _ => panic!("Unknown op"),
                }
            }
            Ast::Assign(name, expr) | Ast::VarDecl(name, expr) => {
                let val = self.gen_ast(expr);
                let var = if let Some(&v) = self.variables.get(name) {
                    v
                } else {
                    let v = Variable::new(self.var_index as usize);
                    self.var_index += 1;
                    self.builder.declare_var(v, types::F64);
                    self.variables.insert(name.clone(), v);
                    v
                };
                self.builder.def_var(var, val);
                val
            }
            Ast::If(cond, then_body, else_body) => {
                let c = self.gen_ast(cond);
                let cond_bool = self.builder.ins().fcmp(ir::condcodes::FloatCC::Ne, c, self.builder.ins().fconst(types::F64, 0.0));
                let then_block = self.builder.create_block();
                let else_block = self.builder.create_block();
                let merge_block = self.builder.create_block();

                self.builder.ins().brif(cond_bool, then_block, &[], else_block, &[]);

                self.builder.switch_to_block(then_block);
                self.builder.seal_block(then_block);
                for stmt in then_body {
                    self.gen_ast(stmt);
                }
                if !self.builder.is_unreachable() {
                    self.builder.ins().jump(merge_block, &[]);
                }

                self.builder.switch_to_block(else_block);
                self.builder.seal_block(else_block);
                if let Some(eb) = else_body {
                    for stmt in eb {
                        self.gen_ast(stmt);
                    }
                }
                if !self.builder.is_unreachable() {
                    self.builder.ins().jump(merge_block, &[]);
                }

                self.builder.switch_to_block(merge_block);
                self.builder.seal_block(merge_block);
                self.builder.ins().fconst(types::F64, 0.0) // Dummy
            }
            Ast::While(cond, body) => {
                let header_block = self.builder.create_block();
                let body_block = self.builder.create_block();
                let exit_block = self.builder.create_block();

                self.builder.ins().jump(header_block, &[]);
                self.builder.switch_to_block(header_block);
                let c = self.gen_ast(cond);
                let cond_bool = self.builder.ins().fcmp(ir::condcodes::FloatCC::Ne, c, self.builder.ins().fconst(types::F64, 0.0));
                self.builder.ins().brif(cond_bool, body_block, &[], exit_block, &[]);

                self.builder.switch_to_block(body_block);
                self.builder.seal_block(body_block);
                for stmt in body {
                    self.gen_ast(stmt);
                }
                self.builder.ins().jump(header_block, &[]);

                self.builder.switch_to_block(exit_block);
                self.builder.seal_block(header_block);
                self.builder.seal_block(exit_block);
                self.builder.ins().fconst(types::F64, 0.0)
            }
            Ast::For(var_name, start, end, body) => {
                let start_val = self.gen_ast(start);
                let end_val = self.gen_ast(end);
                let loop_var = Variable::new(self.var_index as usize);
                self.var_index += 1;
                self.builder.declare_var(loop_var, types::F64);
                self.builder.def_var(loop_var, start_val);
                self.variables.insert(var_name.clone(), loop_var);

                let header_block = self.builder.create_block();
                let body_block = self.builder.create_block();
                let exit_block = self.builder.create_block();

                self.builder.ins().jump(header_block, &[]);
                self.builder.switch_to_block(header_block);
                let current = self.builder.use_var(loop_var);
                let cond = self.builder.ins().fcmp(ir::condcodes::FloatCC::Olt, current, end_val);
                self.builder.ins().brif(cond, body_block, &[], exit_block, &[]);

                self.builder.switch_to_block(body_block);
                self.builder.seal_block(body_block);
                for stmt in body {
                    self.gen_ast(stmt);
                }
                let next = self.builder.ins().fadd(self.builder.use_var(loop_var), self.builder.ins().fconst(types::F64, 1.0));
                self.builder.def_var(loop_var, next);
                self.builder.ins().jump(header_block, &[]);

                self.builder.switch_to_block(exit_block);
                self.builder.seal_block(header_block);
                self.builder.seal_block(exit_block);
                self.builder.ins().fconst(types::F64, 0.0)
            }
            Ast::FuncDef(name, params, body) => {
                let mut sig = self.module.make_signature();
                for _ in params {
                    sig.params.push(AbiParam::new(types::F64));
                }
                sig.returns.push(AbiParam::new(types::F64));
                let func_id = self.module.declare_function(name, Linkage::Local, &sig).unwrap();
                self.functions.insert(name.clone(), func_id);

                let mut local_ctx = CodegenContext::new();
                local_ctx.func.signature = sig.clone();

                let mut local_builder_ctx = FunctionBuilderContext::new();
                let mut local_builder = FunctionBuilder::new(&mut local_ctx.func, &mut local_builder_ctx);

                let entry = local_builder.create_block();
                local_builder.append_block_params_for_function_params(entry);
                local_builder.switch_to_block(entry);
                local_builder.seal_block(entry);

                let mut local_codegen = CodeGen::new(self.module, &mut local_builder, self.printf);

                let block_params = local_builder.block_params(entry).to_vec();
                for (i, param_name) in params.iter().enumerate() {
                    let param_val = block_params[i];
                    let param_var = Variable::new(local_codegen.var_index as usize);
                    local_codegen.var_index += 1;
                    local_codegen.builder.declare_var(param_var, types::F64);
                    local_codegen.builder.def_var(param_var, param_val);
                    local_codegen.variables.insert(param_name.clone(), param_var);
                }

                for stmt in body {
                    local_codegen.gen_ast(stmt);
                }

                let ret_val = local_codegen.builder.ins().fconst(types::F64, 0.0);
                local_codegen.builder.ins().return_(&[ret_val]);

                self.module.define_function(func_id, &mut local_ctx).unwrap();

                self.builder.ins().fconst(types::F64, 0.0)
            }
            Ast::FuncCall(name, args) => {
                if name == "write" {
                    let arg = self.gen_ast(&args[0]);
                    self.builder.ins().call(self.printf, &[arg]);
                    self.builder.ins().fconst(types::F64, 0.0)
                } else {
                    let func_id = *self.functions.get(name).expect("Undefined function");
                    let mut call_args = Vec::new();
                    for arg in args {
                        call_args.push(self.gen_ast(arg));
                    }
                    let inst = self.builder.ins().call(func_id, &call_args);
                    self.builder.inst_results(inst)[0]
                }
            }
            Ast::Array(elements) => {
                // Allocate array on stack (simple, fixed size)
                let size = elements.len() as i64;
                let ptr = self.builder.ins().stack_alloc(types::F64, size as u32, MemFlags::new());
                for (i, elem) in elements.iter().enumerate() {
                    let val = self.gen_ast(elem);
                    let offset = self.builder.ins().iconst(types::I64, i as i64 * 8); // F64 = 8 bytes
                    let addr = self.builder.ins().iadd(ptr, offset);
                    self.builder.ins().store(MemFlags::new(), val, addr, 0);
                }
                // Return ptr (but for simplicity, we might store in var)
                // For now, assume assigned to var
                ptr
            }
            Ast::Index(name, index) => {
                // Assume array var is ptr
                let ptr = self.builder.use_var(*self.variables.get(name).expect("Undefined array"));
                let idx = self.gen_ast(index);
                let idx_i64 = self.builder.ins().fcvt_to_sint(types::I64, idx); // Assume index is f64, convert to i64
                // Bounds check
                // For memory safety: assume size stored somewhere, but for expansion, let's add size map later
                // Skip for now
                let offset = self.builder.ins().imul_imm(idx_i64, 8);
                let addr = self.builder.ins().iadd(ptr, offset);
                self.builder.ins().load(types::F64, MemFlags::new(), addr, 0)
            }
        }
    }
}
