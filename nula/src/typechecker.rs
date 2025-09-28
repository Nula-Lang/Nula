use crate::ast::{AstTree, AstNode, Expr, Type, AstError, FnType};
use std::collections::HashMap;

pub fn typecheck_ast(tree: &mut AstTree) -> Result<(), AstError> {
    // Zbieraj definicje
    collect_defs(tree)?;

    // Check each node
    for node_id in tree.root.descendants(&tree.arena).skip(1) {
        typecheck_node(tree, node_id)?;
    }

    // Check trait impls
    check_trait_impls(tree)?;

    Ok(())
}

fn collect_defs(tree: &mut AstTree) -> Result<(), AstError> {
    for node_id in tree.root.children(&tree.arena) {
        let node = tree.arena.get(node_id).unwrap().get().clone();
        match node {
            AstNode::StructDef { name, generics, fields, .. } => {
                tree.type_env.insert(name.clone(), Type::Struct { name, fields, generics: generics.iter().map(|g| Type::Generic(g.clone())).collect() });
            }
            AstNode::EnumDef { name, generics, variants } => {
                tree.type_env.insert(name.clone(), Type::Enum { name, variants, generics: generics.iter().map(|g| Type::Generic(g.clone())).collect() });
            }
            AstNode::TraitDef { name, methods } => {
                tree.type_env.insert(name.clone(), Type::Trait { name, methods });
            }
            AstNode::ClassDef { name, generics, fields, .. } => {
                // Treat class as struct + impl
                tree.type_env.insert(name.clone(), Type::Struct { name, fields, generics: generics.iter().map(|g| Type::Generic(g.clone())).collect() });
            }
            AstNode::Impl { ty, trait_name, .. } => {
                if let Some(tn) = trait_name {
                    if let Type::Struct { name, .. } | Type::Enum { name, .. } = &ty {
                        tree.trait_impls.insert((name.clone(), tn), true);
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn typecheck_node(tree: &AstTree, node_id: NodeId) -> Result<Type, AstError> {
    let node = tree.arena.get(node_id).unwrap().get();
    match node {
        AstNode::Let { name, ty, value } => {
            let val_ty = typecheck_expr(tree, value)?;
            let declared_ty = ty.clone().unwrap_or(val_ty.clone());
            if val_ty != declared_ty {
                return Err(AstError::TypeMismatch(format!("{:?}", declared_ty), format!("{:?}", val_ty)));
            }
            // Add to local env, but for global assume
            Ok(declared_ty)
        }
        AstNode::Fn { generics, params, ret_ty, body, .. } => {
            // Local env with generics, params
            let mut local_env = HashMap::new();
            for g in generics {
                local_env.insert(g.clone(), Type::Generic(g.clone()));
            }
            for (p, t) in params {
                local_env.insert(p.clone(), t.clone());
            }
            for b in body {
                typecheck_node(tree, tree.arena.get_node_id(b).unwrap())?;  // Assume id
            }
            Ok(Type::Unknown)  // Fn type separate
        }
        // ... Poważna rozbudowa dla wszystkich nodes: check cond is Bool, etc.
        AstNode::Expr(e) => typecheck_expr(tree, e),
        // Dla impl: check methods match trait
        _ => Ok(Type::Unknown),
    }
}

fn typecheck_expr(tree: &AstTree, expr: &Expr) -> Result<Type, AstError> {
    match expr {
        Expr::Int(_) => Ok(Type::Int),
        Expr::Float(_) => Ok(Type::Float),
        Expr::Str(_) => Ok(Type::Str),
        Expr::Bool(_) => Ok(Type::Bool),
        Expr::Var { name, generics } => {
            let mut ty = tree.type_env.get(name).ok_or(AstError::Undefined(name.clone()))?.clone();
            tree.resolve_generics(&name[..], generics)?;  // Bind
            Ok(ty)
        }
        Expr::BinOp { op, left, right } => {
            let l_ty = typecheck_expr(tree, left)?;
            let r_ty = typecheck_expr(tree, right)?;
            if l_ty != r_ty {
                Err(AstError::TypeMismatch(format!("{:?}", l_ty), format!("{:?}", r_ty)))
            } else if op == "+" || op == "-" || op == "*" {
                if matches!(l_ty, Type::Int | Type::Float) {
                    Ok(l_ty)
                } else {
                    Err(AstError::TypeMismatch("Int or Float".to_string(), format!("{:?}", l_ty)))
                }
            } else if op == ">" || op == "<" {
                Ok(Type::Bool)
            } else {
                Err(AstError::Undefined(op.clone()))
            }
        }
        // ... Dla unary, call (resolve fn type, check args), array (check elem types same), index (check arr is Array, idx Int), length (arr is Array)
        Expr::MethodCall { obj, method, args } => {
            let obj_ty = typecheck_expr(tree, obj)?;
            // Resolve method in type or trait
            if let Type::Struct { name, .. } = &obj_ty {
                if let Some(trait_name) = find_trait_for_method(tree, name, method) {
                    if tree.trait_impls.get(&(name.clone(), trait_name)).is_some() {
                        // Check args
                        Ok(Type::Unknown)  // Get ret type from method sig
                    } else {
                        Err(AstError::TraitNotImplemented(name.clone()))
                    }
                } else {
                    Err(AstError::Undefined(method.clone()))
                }
            } else {
                Err(AstError::TypeMismatch("Struct".to_string(), format!("{:?}", obj_ty)))
            }
        }
        // ... Więcej dla struct_lit (check fields match def), enum_lit (check variant exists), field_access
        _ => Ok(Type::Unknown),
    }
}

fn check_trait_impls(tree: &AstTree) -> Result<(), AstError> {
    // Dla każdego impl, check if methods match trait def
    Ok(())
}

fn find_trait_for_method(tree: &AstTree, ty_name: &str, method: &str) -> Option<String> {
    // Search traits that have this method
    None  // Impl
}
