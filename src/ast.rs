//! Data structure for abstract syntax tree

use crate::util::TargetOs;

/// Abstract syntax tree parsed from source
#[derive(Debug)]
pub struct Ast {
    pub func_decls: Vec<FuncDecl>,
}

#[derive(Debug)]
pub struct FuncDecl {
    pub return_type: ReturnType,
    pub name: String,
    pub params: Vec<ParamDecl>,
    pub cmp_stmt: CmpStmt,
}

impl FuncDecl {
    pub fn symbol_name(&self, target: &TargetOs) -> String {
        match target {
            TargetOs::MacOs => format!("_{}", self.name),
            _ => self.name.clone()
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ReturnType {
    Void,
    Data(DataType),
}

#[derive(Debug)]
pub struct ParamDecl {
    data_type: DataType,
    name: String,
}

/// Compound statement
#[derive(Debug)]
pub struct CmpStmt {
    pub stmts: Vec<Stmt>,
}

#[derive(Debug)]
pub enum Stmt {
    VarDecl(DataType, String, Option<Expr>),
    Assignment(String, Expr),
    Return(Option<Expr>),
}

#[derive(Debug)]
pub enum Expr {
    IntConst(i64),
    Arith(Box<Expr>, ArithOp, Box<Expr>),
}

#[derive(Debug)]
pub enum ArithOp {
    Add,
    Sub,
}

#[derive(Debug, PartialEq)]
pub enum DataType {
    Int,
    Float,
}
