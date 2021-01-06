use crate::{
    ast::*,
    common::{TokType, Token},
};

pub fn parse(tokens: Vec<Token>) -> Ast {
    let mut parser = Parser::new(tokens);
    return parser.parse();
}

struct Parser {
    tokens: Vec<Token>,
    index: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser { tokens, index: 0 }
    }

    pub fn parse(&mut self) -> Ast {
        let mut ast = Ast { 0: Vec::new() };

        // parse external decl
        loop {
            let peek = self.peek();
            match peek {
                Some(t) if (self.is_data_type(t)) => {
                    let (return_type, name) = self.parse_declarator();
                    let ext = match self.peek() {
                        // parse function
                        Some(t) if t.tok == TokType::ParentOpen => {
                            let (params, cmp_stmt) = self.parse_func_params_body();
                            ExtDecl::Func(FuncDecl {
                                return_type,
                                name,
                                params,
                                cmp_stmt,
                            })
                        }
                        // parse global function
                        _ => {
                            let expr = match self.peek() {
                                Some(t) if t.tok == TokType::Assign => {
                                    self.consume_any();
                                    Some(self.parse_expr())
                                }
                                _ => None,
                            };
                            self.consume(TokType::Semicolon);
                            ExtDecl::Global(GlobalVarDecl(return_type, name, expr))
                        }
                    };
                    ast.0.push(ext);
                }
                None => break,
                Some(t) => panic!("unexpected {}", t),
            }
        }

        ast
    }

    /// parse function parameters and body (compound statement)
    fn parse_func_params_body(&mut self) -> (Vec<ParamDecl>, CmpStmt) {
        // parameters
        self.consume(TokType::ParentOpen);
        let params = self.parse_parameters();
        self.consume(TokType::ParentClose);

        // compound statement
        let cmp_stmt = self.parse_compound_stmt();

        (params, cmp_stmt)
    }

    /// parse list of parameters
    fn parse_parameters(&mut self) -> Vec<ParamDecl> {
        let mut vec: Vec<ParamDecl> = Vec::new();
        match self.peek() {
            Some(t) if self.is_data_type(t) => {
                vec.push(self.parse_parameter());

                // check if comma
                loop {
                    match self.peek() {
                        Some(t) if t.tok == TokType::Comma => {
                            self.consume_any();
                            vec.push(self.parse_parameter());
                        }
                        _ => break,
                    }
                }
            }
            _ => (),
        }

        vec
    }

    fn parse_parameter(&mut self) -> ParamDecl {
        let (dt, id) = self.parse_declarator();
        ParamDecl {
            data_type: dt,
            name: id,
        }
    }

    fn parse_declarator(&mut self) -> (DataType, String) {
        let dt = self.parse_data_type();
        (dt, self.parse_id())
    }

    fn parse_compound_stmt(&mut self) -> CmpStmt {
        self.consume(TokType::BracketOpen);

        let mut stmts: Vec<Stmt> = Vec::new();

        // parse stmts
        loop {
            if let Some(stmt) = self.parse_stmt() {
                stmts.push(stmt);
            } else {
                break;
            }
        }

        self.consume(TokType::BracketClose);

        CmpStmt { stmts }
    }

    fn parse_stmt(&mut self) -> Option<Stmt> {
        if self.is_expr() {
            return Some(self.parse_expr_stmt());
        }

        let stmt = match self.peek() {
            Some(t) if self.is_data_type(t) => self.parse_var_decl_stmt(),
            Some(t) if t.tok == TokType::KeywordReturn => self.parse_return_stmt(),
            Some(t) if t.tok == TokType::BracketOpen => Stmt::Compound(self.parse_compound_stmt()),
            Some(t) if t.tok == TokType::BracketClose => return None,
            Some(t) => panic!("unexpected {}", t),
            _ => panic!("unexpected EOF"),
        };
        Some(stmt)
    }

    fn parse_var_decl_stmt(&mut self) -> Stmt {
        let decl = self.parse_var_decl();
        self.consume(TokType::Semicolon);
        Stmt::VarDecl(decl)
    }

    fn parse_var_decl(&mut self) -> VarDecl {
        let data_type = self.parse_data_type();
        let name: String = self.parse_id();
        let expr = if self.is_peek_tok(TokType::Assign) {
            self.consume(TokType::Assign);
            Some(self.parse_expr())
        } else {
            None
        };
        VarDecl(data_type, name, expr)
    }

    fn parse_return_stmt(&mut self) -> Stmt {
        self.consume(TokType::KeywordReturn);
        let expr: Option<Expr> = if self.is_expr() {
            Some(self.parse_expr())
        } else {
            None
        };
        self.consume(TokType::Semicolon);
        Stmt::Return(expr)
    }

    /// statement that invoke an expression, i.e function call
    fn parse_expr_stmt(&mut self) -> Stmt {
        let e = self.parse_expr();
        self.consume(TokType::Semicolon);
        Stmt::Expr(e)
    }

    fn is_expr(&mut self) -> bool {
        self.is_int_const_expr() || self.is_ref()
    }

    fn parse_expr(&mut self) -> Expr {
        if self.is_int_const_expr() {
            self.parse_int_const_expr()
        } else if self.is_ref() {
            self.parse_ref_expr()
        } else {
            panic!("expected expression but {:?}", self.peek())
        }
    }

    fn is_int_const_expr(&mut self) -> bool {
        match self.peek() {
            Some(Token {
                tok: TokType::NumInt(_),
                loc: _,
            }) => true,
            _ => false,
        }
    }

    fn parse_int_const_expr(&mut self) -> Expr {
        match self.next() {
            Some(Token {
                tok: TokType::NumInt(v),
                loc: _,
            }) => Expr::IntConst(*v as i64),
            Some(t) => panic!("expected int constant but {}", t),
            None => panic!("unexpected EOF"),
        }
    }

    fn is_ref(&mut self) -> bool {
        match self.peek() {
            Some(Token {
                tok: TokType::ID(_),
                loc: _,
            }) => true,
            _ => false,
        }
    }

    /// parse function or variable call
    ///
    /// TODO parse array index
    fn parse_ref_expr(&mut self) -> Expr {
        let name = self.parse_id();
        match self.peek() {
            Some(t) if t.tok == TokType::ParentOpen => self.parse_function_call_expr(name),
            _ => Expr::VarRef(name),
        }
    }

    fn parse_function_call_expr(&mut self, name: String) -> Expr {
        self.consume(TokType::ParentOpen);
        let args = self.parse_arguments();
        self.consume(TokType::ParentClose);
        Expr::FunctionCall(name, args)
    }

    fn parse_arguments(&mut self) -> Vec<Expr> {
        if self.is_expr() {
            let mut vec: Vec<Expr> = Vec::new();
            vec.push(self.parse_expr());
            loop {
                if self.is_peek_tok(TokType::Comma) {
                    self.consume_any();
                    vec.push(self.parse_expr());
                } else {
                    break;
                }
            }
            vec
        } else {
            Vec::with_capacity(0)
        }
    }

    fn is_data_type(&self, tok: &Token) -> bool {
        self.has_value(Parser::parse_data_type_opt(tok))
    }

    const fn parse_data_type_opt(tok: &Token) -> Option<DataType> {
        match tok.tok {
            TokType::KeywordInt => Some(DataType::Int),
            TokType::KeywordVoid => Some(DataType::Void),
            _ => None,
        }
    }

    fn parse_data_type(&mut self) -> DataType {
        let t = self.next().expect("unexpected EOF");
        Parser::parse_data_type_opt(t).expect(&format!("expected data type but {}", t))
    }

    fn parse_id(&mut self) -> String {
        match self.next() {
            Some(Token {
                tok: TokType::ID(s),
                loc: _,
            }) => s.to_string(),
            Some(t) => panic!("exepcted ID but {}", t),
            _ => panic!("unexpected EOF"),
        }
    }

    fn is_id(&mut self) -> bool {
        match self.peek() {
            Some(Token {
                tok: TokType::ID(_),
                loc: _,
            }) => true,
            _ => false,
        }
    }

    fn has_value<T>(&self, opt: Option<T>) -> bool {
        match opt {
            Some(_) => true,
            _ => false,
        }
    }

    fn is_peek_tok(&mut self, tok: TokType) -> bool {
        match self.peek() {
            Some(Token { tok: t, loc: _ }) if *t == tok => true,
            _ => false,
        }
    }

    fn consume_any(&mut self) {
        let _ = self.next();
    }

    fn consume(&mut self, tok: TokType) {
        let item = self
            .next()
            .expect(format!("expected {} but EOF", tok).as_str());
        match item {
            Token { tok: t, loc: _ } if *t == tok => (),
            t => panic!("expected {} but {}", tok, t),
        }
    }
}

enum ExprRefType {
    FunctionCall,
    ArrayIndex,
    VarRef,
}

trait TokenPeeker {
    fn next(&mut self) -> Option<&Token>;
    fn peek(&self) -> Option<&Token>;
    fn lookahead(&self, i: usize) -> Option<&Token>;
    fn peek_tok(&self) -> Option<&TokType>;
    fn lookahead_tok(&self, i: usize) -> Option<&TokType>;
}

impl TokenPeeker for Parser {
    fn next(&mut self) -> Option<&Token> {
        let t = self.tokens.get(self.index);
        self.index = self.index + 1;
        t
    }

    fn peek(&self) -> Option<&Token> {
        self.lookahead(0)
    }

    fn lookahead(&self, i: usize) -> Option<&Token> {
        self.tokens.get(self.index + i)
    }

    fn peek_tok(&self) -> Option<&TokType> {
        self.lookahead_tok(0)
    }

    fn lookahead_tok(&self, i: usize) -> Option<&TokType> {
        self.lookahead(i).map(|t| &t.tok)
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
    #[test_case("void test() { int a = 3; return a; }")]
    #[test_case("void foo(int x, int y) {}")]
    #[test_case("void foo() { int a = undefined(x, 3); }")]
    #[test_case("void foo() { undefined(3); }")]
    fn pass_program(src: &str) {
        parse(scan(src));
    }

    #[test_case("main" => panics "unexpected identifier 'main' at 1:1")]
    #[test_case("int main" => panics "expected ; but EOF")]
    #[test_case("int test {" => panics "expected ; but {")]
    #[test_case("int test() {" => panics "unexpected EOF")]
    #[test_case("int main() { return 1 }" => panics "expected ; but }")]
    fn failed_program(src: &str) {
        parse(scan(src));
    }

    #[test_case("int g = 101; void foo() { int a = g;}")]
    #[test_case("int g = 101; void foo() { int g = 2; { int g = 3; }}")]
    fn parse_global(src: &str) {
        parse(scan(src));
    }

    // #[test_case("int main() { int a; a = 1; }")]
    // fn parse_stmt(src: &str) {
    //     parse(scan(src));
    // }
}
