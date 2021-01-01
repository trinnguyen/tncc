use std::{iter::Peekable, slice::Iter};

use crate::{
    ast::*,
    common::{TokType, Token},
};

pub fn parse(tokens: Vec<Token>) -> Ast {
    let mut iter = tokens.iter().peekable();
    let mut decls: Vec<FuncDecl> = Vec::new();

    // parse functions
    loop {
        let peek = iter.peek();
        match peek {
            Some(t) if (is_data_type(t)) || t.tok == TokType::KeywordVoid => {
                decls.push(parse_function(&mut iter))
            }
            None => break,
            Some(t) => panic!("unexpected {}", t),
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
    let expr: Option<Expr> = if is_expr(iter) {
        Some(parse_expr(iter))
    } else {
        None
    };
    consume(iter, TokType::Semicolon);
    Stmt::Return(expr)
}

fn is_expr(iter: &mut Peekable<Iter<Token>>) -> bool {
    is_int_const_expr(iter)
}

fn is_int_const_expr(iter: &mut Peekable<Iter<Token>>) -> bool {
    match iter.peek() {
        Some(Token {
            tok: TokType::NumInt(_),
            loc: _,
        }) => true,
        _ => false,
    }
}

fn parse_expr(iter: &mut Peekable<Iter<Token>>) -> Expr {
    if is_int_const_expr(iter) {
        parse_int_const_expr(iter)
    } else {
        panic!("expected expression but {:?}", iter.peek())
    }
}

fn parse_int_const_expr(iter: &mut Peekable<Iter<Token>>) -> Expr {
    match iter.next() {
        Some(Token {
            tok: TokType::NumInt(v),
            loc: _,
        }) => Expr::IntConst(*v as i64),
        Some(t) => panic!("expected int constant but {}", t),
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
    let item = iter
        .next()
        .expect(format!("expected {} but EOF", tok).as_str());
    match item {
        Token { tok: t, loc: _ } if *t == tok => (),
        t => panic!("expected {} but {}", tok, t),
    }
}

#[cfg(test)]
mod test {
    use test_case::test_case;

    use crate::scan;

    use super::parse;

    #[test_case("int main() { return 1; }")]
    #[test_case("int main() { }")]
    #[test_case("void test() { return 1; }")]
    #[test_case("int main() { int a = 100; return 1; }")]
    fn pass_program(src: &str) {
        parse(scan(src));
    }

    #[test_case("main" => panics "unexpected identifier 'main' at 1:1")]
    #[test_case("int main" => panics "expected ( but EOF")]
    #[test_case("int test {" => panics "expected ( but {")]
    #[test_case("int test() {" => panics "expected } but EOF")]
    #[test_case("int main() { return 1 }" => panics "expected ; but }")]
    fn failed_program(src: &str) {
        parse(scan(src));
    }
}
