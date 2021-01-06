//! Semantics analysis and type checking
//!
//! Decorate abstract syntax tree with type information

use crate::{
    ast::{Ast, CmpStmt, ExtDecl, Stmt},
    symtable::SymTable,
};

pub fn analyse(ast: &mut Ast) {
    // create symbol table
    let mut table = SymTable::new();

    // enter new scope
    table.push_scope();

    // travel through the ast
    for ext_decl in &ast.0 {
        match ext_decl {
            ExtDecl::Func(decl) => {
                table.cur_scope().insert_decl(&decl.name, decl);
                analyse_cmp_stmt(&mut table, &decl.cmp_stmt);
            }
            ExtDecl::Global(decl) => table.cur_scope().insert_decl(&(decl.1), decl),
        }
    }

    // pop scope
    table.pop_scope();
}

pub fn analyse_cmp_stmt<'a>(table: &mut SymTable<'a>, cmp_stmt: &'a CmpStmt) {
    // enter new scope
    table.push_scope();

    for stmt in &cmp_stmt.stmts {
        match stmt {
            Stmt::Compound(st) => analyse_cmp_stmt(table, st),
            Stmt::VarDecl(decl) => table.cur_scope().insert_decl(&decl.1, decl),
            Stmt::Assignment(_, _) => {}
            Stmt::Return(_) => {}
            Stmt::Expr(_) => {}
        }
    }

    // pop scope
    table.pop_scope();
}

#[cfg(test)]
mod test {}
