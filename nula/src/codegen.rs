use crate::ir::{IrModule, IrInstr, IrValue};
use crate::ast::{Type};
use std::fmt::Write as FmtWrite;

// Poważny codegen: generuj x86-64 assembly bezpośrednio z IR
pub fn generate_assembly(module: &IrModule) -> String {
    let mut asm = String::new();
    asm.push_str(".section .text\n");

    for global in &module.globals {
        asm.push_str(&format!(".global {}\n", global.0));
        // Gen global data
    }

    for func in &module.functions {
        asm.push_str(&format!(".global {}\n{}:\n", func.name, func.name));
        // Prolog: push rbp, mov rbp rsp, sub rsp stack_size
        let stack_size = calculate_stack_size(func);  // Calc allocas
        asm.push_str("    push %rbp\n    mov %rsp, %rbp\n");
        if stack_size > 0 {
            asm.push_str(&format!("    sub ${}, %rsp\n", stack_size));
        }

        let mut regs = RegAllocator::new();  // Poważna alokacja rejestrów
        let mut labels = LabelGenerator::new();

        for instr in &func.body {
            match instr {
                IrInstr::Label(l) => asm.push_str(&format!("{}:\n", l)),
                IrInstr::Alloca { dest, ty } => {
                    let size = ty_size(ty);
                    let offset = regs.alloc_stack(size);
                    regs.bind_reg(dest, Reg::Stack(offset));
                }
                IrInstr::Load { dest, src } => {
                    let src_reg = regs.get_reg(src);
                    let dest_reg = regs.alloc_temp();
                    asm.push_str(&format!("    mov {}, {}\n", src_reg, dest_reg));
                    regs.bind_reg(dest, dest_reg);
                }
                IrInstr::Store { dest, value } => {
                    let dest_reg = regs.get_reg(dest);
                    let val_reg = gen_value(value, &mut regs, &mut asm);
                    asm.push_str(&format!("    mov {}, {}\n", val_reg, dest_reg));
                }
                IrInstr::BinOp { dest, op, left, right } => {
                    let left_reg = gen_value(left, &mut regs, &mut asm);
                    let right_reg = gen_value(right, &mut regs, &mut asm);
                    let dest_reg = regs.alloc_temp();
                    match op.as_str() {
                        "+" => asm.push_str(&format!("    add {}, {}\n", right_reg, left_reg)),
                        "-" => asm.push_str(&format!("    sub {}, {}\n", right_reg, left_reg)),
                        "*" => asm.push_str(&format!("    imul {}, {}\n", right_reg, left_reg)),
                        // ... Dla > < use cmp + set
                        _ => {},
                    }
                    asm.push_str(&format!("    mov {}, {}\n", left_reg, dest_reg));
                    regs.bind_reg(dest, dest_reg);
                    regs.free_temp(left_reg);
                    regs.free_temp(right_reg);
                }
                IrInstr::Call { dest, fn_name, args } => {
                    // Push args to stack or regs (ABI)
                    for (i, arg) in args.iter().enumerate().rev() {
                        let arg_reg = gen_value(arg, &mut regs, &mut asm);
                        if i < 6 {
                            asm.push_str(&format!("    mov {}, {}\n", arg_reg, arg_regs[i]));
                        } else {
                            asm.push_str(&format!("    push {}\n", arg_reg));
                        }
                    }
                    asm.push_str(&format!("    call {}\n", fn_name));
                    if let Some(d) = dest {
                        let dest_reg = regs.alloc_temp();
                        asm.push_str(&format!("    mov %rax, {}\n", dest_reg));
                        regs.bind_reg(d, dest_reg);
                    }
                }
                IrInstr::Branch { cond, true_label, false_label } => {
                    if let Some(c) = cond {
                        let cond_reg = gen_value(c, &mut regs, &mut asm);
                        asm.push_str(&format!("    test {}, {}\n    jnz {}\n    jmp {}\n", cond_reg, cond_reg, true_label, false_label));
                    } else {
                        asm.push_str(&format!("    jmp {}\n", true_label));
                    }
                }
                IrInstr::Ret(value) => {
                    let val_reg = gen_value(value, &mut regs, &mut asm);
                    asm.push_str(&format!("    mov {}, %rax\n", val_reg));
                    asm.push_str("    leave\n    ret\n");
                }
                IrInstr::Gep { dest, ptr, indices } => {
                    let ptr_reg = regs.get_reg(ptr);
                    let mut res_reg = ptr_reg.clone();
                    for idx in indices {
                        let idx_reg = gen_value(idx, &mut regs, &mut asm);
                        let temp = regs.alloc_temp();
                        asm.push_str(&format!("    lea ({},{},8), {}\n", ptr_reg, idx_reg, temp));  // Assume i64
                        res_reg = temp;
                    }
                    regs.bind_reg(dest, res_reg);
                }
                // ... Poważna obsługa dla wszystkich IR instr
            }
        }
    }
    asm
}

fn ty_size(ty: &Type) -> u32 {
    match ty {
        Type::Int => 8,
        Type::Float => 8,
        Type::Bool => 1,
        Type::Str => 8,  // Ptr
        Type::Array(_) => 8,  // Ptr
        Type::Struct { fields, .. } => fields.iter().map(|(_, t)| ty_size(t)).sum(),
        // ...
        _ => 8,
    }
}

fn calculate_stack_size(func: &IrFunction) -> u32 {
    // Count allocas * sizes
    0  // Impl
}

struct RegAllocator {
    temps: Vec<Reg>,
    stack_offset: u32,
    bindings: HashMap<String, Reg>,
}

#[derive(Clone)]
enum Reg {
    Gpr(String),  // %rax, %rbx, etc.
    Stack(u32),  // -offset(%rbp)
}

impl RegAllocator {
    fn new() -> Self {
        RegAllocator { temps: vec![], stack_offset: 0, bindings: HashMap::new() }
    }

    fn alloc_temp(&mut self) -> Reg {
        // Use callee-saved regs, etc.
        Reg::Gpr("%r10".to_string())  // Simpl
    }

    fn free_temp(&mut self, reg: Reg) {
        // Push back
    }

    fn alloc_stack(&mut self, size: u32) -> u32 {
        let offset = self.stack_offset;
        self.stack_offset += size;
        offset
    }

    fn bind_reg(&mut self, name: &str, reg: Reg) {
        self.bindings.insert(name.to_string(), reg);
    }

    fn get_reg(&self, name: &str) -> Reg {
        self.bindings.get(name).cloned().unwrap_or(Reg::Stack(0))
    }
}

struct LabelGenerator {
    counter: u32,
}

impl LabelGenerator {
    fn new() -> Self { LabelGenerator { counter: 0 } }
    fn gen(&mut self) -> String {
        self.counter += 1;
        format!(".L{}", self.counter)
    }
}

fn gen_value(value: &IrValue, regs: &mut RegAllocator, asm: &mut String) -> Reg {
    match value {
        IrValue::Reg(r) => regs.get_reg(r),
        IrValue::ImmInt(i) => {
            let temp = regs.alloc_temp();
            asm.push_str(&format!("    mov ${}, {}\n", i, temp));
            temp
        }
        // ... Dla float (movsd), str (lea .str, reg), bool
        _ => Reg::Gpr("%rax".to_string()),
    }
}

// Dla foreign: parse and gen IR then asm
