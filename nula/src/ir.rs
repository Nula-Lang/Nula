use crate::ast::{Type, Expr, AstNode};

// Intermediate Representation for optimization before codegen
#[derive(Debug, Clone)]
pub enum IrInstr {
    Load { dest: String, src: String },
    Store { dest: String, value: IrValue },
    BinOp { dest: String, op: String, left: IrValue, right: IrValue },
    Call { dest: Option<String>, fn_name: String, args: Vec<IrValue> },
    Branch { cond: Option<IrValue>, true_label: String, false_label: String },
    Label(String),
    Ret(IrValue),
    Alloca { dest: String, ty: Type },
    Gep { dest: String, ptr: String, indices: Vec<IrValue> },
    // Więcej dla loops, structs, etc.
}

#[derive(Debug, Clone)]
pub enum IrValue {
    Reg(String),
    ImmInt(i64),
    ImmFloat(f64),
    ImmStr(String),
    ImmBool(bool),
}

pub struct IrFunction {
    name: String,
    params: Vec<(String, Type)>,
    ret_ty: Type,
    body: Vec<IrInstr>,
}

pub struct IrModule {
    functions: Vec<IrFunction>,
    globals: HashMap<String, Type>,
}

impl IrModule {
    pub fn new() -> Self {
        IrModule { functions: vec![], globals: HashMap::new() }
    }

    pub fn add_function(&mut self, fn_ir: IrFunction) {
        self.functions.push(fn_ir);
    }

    // Methods for optimization: dead code elim, const fold, etc.
    pub fn optimize(&mut self) {
        for f in &mut self.functions {
            const_fold(f);
            dead_code_elim(f);
        }
    }
}

fn const_fold(f: &mut IrFunction) {
    // Traverse body, fold const binops
    // e.g. BinOp dest, +, Imm 2, Imm 3 -> Store dest, Imm 5
}

fn dead_code_elim(f: &mut IrFunction) {
    // Remove unused instr
}

pub fn ast_to_ir(tree: &AstTree) -> Result<IrModule, AstError> {
    let mut module = IrModule::new();
    // Convert each top-level fn/struct to IR
    for child in tree.root.children(&tree.arena) {
        let node = tree.arena.get(child).unwrap().get();
        match node {
            AstNode::Fn { name, params, ret_ty, body, .. } => {
                let mut ir_body = vec![];
                // Gen IR for body recursively
                gen_ir_body(tree, body, &mut ir_body)?;
                module.add_function(IrFunction { name: name.clone(), params: params.clone(), ret_ty: ret_ty.clone(), body: ir_body });
            }
            // ... Dla globals, structs (as global types)
            _ => {},
        }
    }
    Ok(module)
}

fn gen_ir_body(tree: &AstTree, nodes: &[AstNode], ir: &mut Vec<IrInstr>) -> Result<(), AstError> {
    for n in nodes {
        match n {
            AstNode::Let { name, value, .. } => {
                ir.push(IrInstr::Alloca { dest: name.clone(), ty: Type::Unknown });  // Get ty
                let val_ir = gen_ir_expr(value)?;
                ir.push(IrInstr::Store { dest: name.clone(), value: val_ir });
            }
            // ... Poważna konwersja dla wszystkich: if -> branch, for -> loop with labels, etc.
            _ => {},
        }
    }
    Ok(())
}

fn gen_ir_expr(expr: &Expr) -> Result<IrValue, AstError> {
    match expr {
        Expr::Int(i) => Ok(IrValue::ImmInt(*i)),
        // ... Dla binop: gen left, right, if const fold, else new reg + BinOp
        _ => Ok(IrValue::Reg("temp".to_string())),
    }
}
