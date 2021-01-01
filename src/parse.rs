use std::{iter::Peekable, slice::Iter};

use crate::{
    ast::*,
    common::{TokType, Token},
};

pub fn parse(tokens: Vec<Token>) -> Ast {
    let mut toks = tokens;
    let mut iter = toks.iter().peekable();
    let mut decls: Vec<FuncDecl> = Vec::new();

    // parse functions
    loop {
        let peek = iter.peek();
        match peek {
            Some(t) if is_data_type(t) || t.tok == TokType::KeywordVoid => {
                decls.push(parse_function(&mut iter))
            }
            None => break,
            _ => panic!("unexpected token: {:?}", peek),
        }
    }

    Ast { func_decls: decls }
}

fn parse_function(iter: &mut Peekable<Iter<Token>>) -> FuncDecl {
    let return_type = match iter.next().unwrap() {
        Token {
            tok: TokType::KeywordVoid,
            loc: _,
        } => ReturnType::Void,
        t => match parse_data_type(t) {
            Some(typ) => ReturnType::Data(typ),
            _ => panic!("expected 'int' or 'void' but {}", t),
        },
    };

    let name = parse_id(iter);

    // TODO parse parameters
    consume(iter, TokType::ParentOpen);
    consume(iter, TokType::ParentClose);

    // compound statement
    let cmp_stmt = parse_compound_stmt(iter);

    FuncDecl {
        return_type,
        name,
        params: Vec::new(),
        cmp_stmt,
    }
}

fn parse_compound_stmt(iter: &mut Peekable<Iter<Token>>) -> CmpStmt {
    consume(iter, TokType::BracketOpen);

    let mut stmts: Vec<Stmt> = Vec::new();

    // parse stmts
    loop {
        let stmt: Stmt = match iter.peek() {
            Some(t) if is_data_type(t) => parse_var_decl_stmt(iter),
            Some(t) if t.tok == TokType::KeywordReturn => parse_return_stmt(iter),
            Some(t) if t.tok == TokType::BracketClose => break,
            Some(t) => panic!("unexpected {}", t),
            None => break,
        };

        stmts.push(stmt);
    }

    consume(iter, TokType::BracketClose);

    CmpStmt { stmts }
}

fn parse_var_decl_stmt(iter: &mut Peekable<Iter<Token>>) -> Stmt {
    let data_type = parse_data_type(iter.next().unwrap()).expect("expected data type");
    let name: String = parse_id(iter);
    let expr = if is_peek_tok(iter, TokType::Assign) {
        consume(iter, TokType::Assign);
        Some(parse_expr(iter))
    } else {
        None
    };
    consume(iter, TokType::Semicolon);
    Stmt::VarDecl(data_type, name, expr)
}

fn parse_return_stmt(iter: &mut Peekable<Iter<Token>>) -> Stmt {
    consume(iter, TokType::KeywordReturn);
    let expr: Expr = parse_expr(iter);
    consume(iter, TokType::Semicolon);
    Stmt::Return(expr)
}

fn parse_expr(iter: &mut Peekable<Iter<Token>>) -> Expr {
    match iter.peek() {
        Some(Token {
            tok: TokType::NumInt(_),
            loc: _,
        }) => parse_int_const_expr(iter),
        Some(t) => panic!("exepcted expression but {}", t),
        None => panic!("unexpected EOF"),
    }
}

fn parse_int_const_expr(iter: &mut Peekable<Iter<Token>>) -> Expr {
    match iter.next() {
        Some(Token {
            tok: TokType::NumInt(v),
            loc: _,
        }) => Expr::IntConst(*v as i64),
        Some(t) => panic!("expected int const but {}", t),
        None => panic!("unexpected EOF"),
    }
}

fn is_data_type(tok: &Token) -> bool {
    has_value(parse_data_type(tok))
}

fn parse_data_type(tok: &Token) -> Option<DataType> {
    match tok.tok {
        TokType::KeywordInt => Some(DataType::Int),
        _ => None,
    }
}

fn parse_id(iter: &mut Peekable<Iter<Token>>) -> String {
    match iter.next() {
        Some(Token {
            tok: TokType::ID(s),
            loc: _,
        }) => s.to_string(),
        Some(t) => panic!("exepcted ID but {}", t),
        _ => panic!("unexpected EOF"),
    }
}

fn has_value<T>(opt: Option<T>) -> bool {
    match opt {
        Some(_) => true,
        _ => false,
    }
}

fn is_peek_tok(iter: &mut Peekable<Iter<Token>>, tok: TokType) -> bool {
    match iter.peek() {
        Some(Token { tok: t, loc: _ }) if *t == tok => true,
        _ => false,
    }
}

fn consume(iter: &mut Peekable<Iter<Token>>, tok: TokType) {
    let item = iter.next().expect("expected {} but EOF");
    match item {
        Token { tok: t, loc: _ } if *t == tok => (),
        t => panic!("expected {} but {}", tok, t),
    }
}
