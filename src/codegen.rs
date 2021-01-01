//! Generate ARM assembly from AST

use crate::ast::{Ast, CmpStmt, Expr, FuncDecl, ReturnType, Stmt};

pub fn gen_asm(ast: &Ast) -> String {
    let mut g = ArmGen::from(ast);
    g.gen();
    g.str
}

struct ArmGen<'a> {
    ast: &'a Ast,
    str: String,
}

impl<'a> From<&'a Ast> for ArmGen<'a> {
    fn from(ast: &'a Ast) -> Self {
        ArmGen {
            ast,
            str: String::new(),
        }
    }
}

impl<'a> ArmGen<'a> {
    /// generate ARM assembly for the AST
    fn gen(&mut self) {
        let begin = ".text";
        self.pushln_tab_str(begin);
        self.ast.func_decls.iter().for_each(|f| self.gen_func(f));
    }

    fn gen_func(&mut self, func: &FuncDecl) {
        // decl
        self.pushln_tab(format!(".global {}", func.name));
        self.pushln(format!("{}:", func.name));

        // body with statement
        self.emit_cmp_stmt(&func.cmp_stmt);

        // finish function
        if func.return_type == ReturnType::Void {
            self.pushln_tab_str("ret");
        }
    }

    fn emit_cmp_stmt(&mut self, cmp_stmt: &CmpStmt) {
        cmp_stmt.stmts.iter().for_each(|stmt| self.emit_stmt(stmt))
    }

    fn emit_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Return(opt) => {
                if let Some(expr) = opt {
                    self.emit_expr(expr);
                }
                self.pushln_tab_str("ret");
            }
            _ => panic!("not supported: {:?}", stmt),
        }
    }

    fn emit_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::IntConst(v) => self.pushln_tab(format!("mov x0, #{}", v)),
            _ => panic!("not supported: {:?}", expr),
        }
    }
}

trait Render {
    fn pushln_tab(&mut self, str: String);
    fn pushln(&mut self, str: String);
    fn pushln_tab_str(&mut self, str: &str);
    fn pushln_str(&mut self, str: &str);
}

impl<'a> Render for ArmGen<'a> {
    fn pushln_tab(&mut self, str: String) {
        self.pushln_tab_str(&str);
    }

    fn pushln(&mut self, str: String) {
        self.pushln_str(&str);
    }

    fn pushln_tab_str(&mut self, str: &str) {
        self.str.push('\t');
        self.pushln_str(str);
    }

    fn pushln_str(&mut self, str: &str) {
        self.str.push_str(str);
        self.str.push('\n');
    }
}

#[cfg(test)]
mod test {
    use crate::{parse, scan};

    use super::gen_asm;

    #[test]
    fn expect_header() {
        let v = gen_asm(&parse(scan("int main(){return 1;}")));
        vec![".text", ".global main", "main:", "mov x0, #1", "ret"]
            .iter()
            .for_each(|i| {
                if !v.contains(i) {
                    panic!("'{}' is not generated", i)
                }
            });
    }
}
