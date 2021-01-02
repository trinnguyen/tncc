//! Generate ARM assembly from AST

use crate::{ast::{Ast, CmpStmt, Expr, FuncDecl, ReturnType, Stmt}, util::TargetOs};

pub fn gen_asm(ast: &Ast, target: &TargetOs) -> String {
    let mut g = ArmGen::new(ast, target);
    g.gen();
    g.str
}

struct ArmGen<'a> {
    ast: &'a Ast,
    str: String,
    target: TargetOs
}

impl<'a> ArmGen<'a> {
    /// create new arm
    fn new(ast: &'a Ast, target: &TargetOs) -> Self {
        ArmGen {
            ast,
            str: String::new(),
            target: *target
        }
    }

    /// generate ARM assembly for the AST
    fn gen(&mut self) {
        let begin = ".text";
        self.pushln_tab_str(begin);
        self.ast.func_decls.iter().for_each(|f| self.gen_func(f));
    }

    fn gen_func(&mut self, func: &FuncDecl) {
        // decl
        self.pushln_tab(format!(".global {}", func.symbol_name(&self.target)));
        self.pushln_tab_str(".p2align 2");
        self.pushln(format!("{}:", func.symbol_name(&self.target)));

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
    use crate::{parse, scan, util::TargetOs};

    use super::gen_asm;

    #[test]
    fn expect_header_linux() {
        let v = gen_asm(&parse(scan("int main(){return 1;}")), &TargetOs::Linux);
        vec![".text", ".global main", "main:", ".p2align 2", "mov x0, #1", "ret"]
            .iter()
            .for_each(|i| {
                if !v.contains(i) {
                    panic!("'{}' is not generated", i)
                }
            });
    }

    #[test]
    fn expect_header_macos() {
        let v = gen_asm(&parse(scan("int main(){return 1;}")), &TargetOs::MacOs);
        vec![".text", ".global _main", ".p2align 2", "_main:", "mov x0, #1", "ret"]
            .iter()
            .for_each(|i| {
                if !v.contains(i) {
                    panic!("'{}' is not generated", i)
                }
            });
    }
}
