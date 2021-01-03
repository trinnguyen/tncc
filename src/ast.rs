//! Data structure for abstract syntax tree

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

#[derive(Debug, PartialEq)]
pub enum ReturnType {
    Void,
    Data(DataType),
}

#[derive(Debug)]
pub struct ParamDecl {
    pub data_type: DataType,
    pub name: String,
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
    Expr(Expr)
}

#[derive(Debug)]
pub enum Expr {
    IntConst(i64),
    FunctionCall(String, Vec<Expr>),
    VarRef(String),
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
