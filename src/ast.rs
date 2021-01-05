//! Data structure for abstract syntax tree

/// Abstract syntax tree parsed from source
#[derive(Debug)]
pub struct Ast(pub Vec<ExtDecl>);

#[derive(Debug)]
pub enum ExtDecl {
    Func(FuncDecl),
    Global(VarDecl),
}

#[derive(Debug)]
pub struct FuncDecl {
    pub return_type: DataType,
    pub name: String,
    pub params: Vec<ParamDecl>,
    pub cmp_stmt: CmpStmt,
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
    Compound(CmpStmt),
    VarDecl(VarDecl),
    Assignment(String, Expr),
    Return(Option<Expr>),
    Expr(Expr),
}

#[derive(Debug)]
pub enum Expr {
    IntConst(i64),
    FunctionCall(String, Vec<Expr>),
    VarRef(String),
    Arith(Box<Expr>, ArithOp, Box<Expr>),
}

#[derive(Debug)]
pub struct VarDecl(pub DataType, pub String, pub Option<Expr>);

#[derive(Debug)]
pub enum ArithOp {
    Add,
    Sub,
}

#[derive(Debug, PartialEq)]
pub enum DataType {
    Void,
    Char,
    Short,
    Int,
    Long,
    Float,
    Double,
}
