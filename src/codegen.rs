//! Generate ARM assembly from AST

use crate::{
    ast::{Ast, CmpStmt, DataType, Expr, FuncDecl, ReturnType, Stmt},
    util::TargetOs,
};

/// register for frame pointer      
const FP: &str = "x29";

/// register for link pointer
const LP: &str = "x30";

static ARG_REGS: &'static [&str] = &["x0", "x1", "x2", "x3", "x4", "x5", "x6", "x7"];

pub fn gen_asm(ast: &Ast, target: &TargetOs) -> String {
    let mut g = ArmGen::new(ast, target);
    g.gen();
    g.str
}

struct ArmGen<'a> {
    ast: &'a Ast,
    str: String,
    target: TargetOs,
}

impl<'a> ArmGen<'a> {
    /// create new arm
    fn new(ast: &'a Ast, target: &TargetOs) -> Self {
        ArmGen {
            ast,
            str: String::new(),
            target: *target,
        }
    }

    /// generate ARM assembly for the AST
    fn gen(&mut self) {
        let begin = ".text";
        self.ptab(begin);
        self.ast.func_decls.iter().for_each(|f| self.gen_func(f));
    }

    fn gen_func(&mut self, func: &FuncDecl) {
        // decl
        self.ptab(&format!(".global {}", self.to_symbol(&func.name)));
        self.ptab(".p2align 2");
        self.pln(&format!("{}:", self.to_symbol(&func.name)));

        // calculate space needed for arguments and local variables
        let size: u32 = gen_util::size_args_local(func);

        let sp_offset: u32 = gen_util::get_sp_offset(size);
        // save sp
        if sp_offset > 0 {
            self.ptab(&format!("sub sp, sp, #{}", sp_offset));
        }

        // emit args
        let mut arg_offset = sp_offset;
        func.params
            .iter()
            .take(ARG_REGS.len())
            .enumerate()
            .for_each(|(i, arg)| {
                arg_offset = arg_offset - arg.data_type.get_size();
                let reg = ARG_REGS.get(i).unwrap();
                self.ptab(&format!("str {}, [sp, #{}]", *reg, arg_offset));
            });

        // body with statement
        self.emit_cmp_stmt(&func.cmp_stmt);

        // restore sp
        if sp_offset > 0 {
            self.ptab(&format!("add sp, sp, #{}", sp_offset));
        }

        // finish function
        self.ptab("ret");

        // empty new line
        self.pln("");
    }

    /// emit compound statement
    fn emit_cmp_stmt(&mut self, cmp_stmt: &CmpStmt) {
        cmp_stmt.stmts.iter().for_each(|stmt| self.emit_stmt(stmt))
    }

    /// emit statement
    fn emit_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Return(opt) => {
                if let Some(expr) = opt {
                    self.emit_expr(expr, Some("x0"));
                }
                // ret inst is emitted by the function
            }
            Stmt::Expr(e) => self.emit_expr(e, None),
            _ => panic!("not supported: {:?}", stmt),
        }
    }

    /// emit expression and return value to reg
    fn emit_expr(&mut self, expr: &Expr, dst_reg: Option<&str>) {
        match expr {
            Expr::IntConst(v) => {
                self.util_move_reg(dst_reg, &format!("#{}", v));
            }
            Expr::FunctionCall(name, args) => {
                // push fp, lr
                self.ptab(&format!("stp {}, {}, [sp, #-16]!", FP, LP));

                // update fp
                self.ptab(&format!("mov {}, sp", FP));

                // move arguments to registers (8)
                args.iter()
                    .take(ARG_REGS.len())
                    .enumerate()
                    .for_each(|(i, arg)| {
                        let reg = ARG_REGS.get(i).unwrap();
                        self.emit_expr(arg, Some(*reg));
                    });

                // call
                self.ptab(&format!("bl {}", self.to_symbol(name)));

                // pop fp, lr
                self.ptab(&format!("ldp {}, {}, [sp], #16", FP, LP));

                // return value (in x0) to reg
                self.util_move_reg(dst_reg, "x0");
            }
            _ => panic!("not supported: {:?}", expr),
        }
    }

    /// util move to reg with optimization
    fn util_move_reg(&mut self, dst: Option<&str>, src: &str) {
        dst.map(|r| {
            if r != src {
                self.ptab(&format!("mov {}, {}", r, src))
            }
        });
    }

    /// gen symbol name based on os
    fn to_symbol(&self, name: &str) -> String {
        match self.target {
            TargetOs::MacOs => format!("_{}", name),
            _ => String::from(name),
        }
    }
}

trait Render {
    /// push with tab and new line
    fn ptab(&mut self, str: &str);

    /// push with new line
    fn pln(&mut self, str: &str);
}

impl<'a> Render for ArmGen<'a> {
    fn ptab(&mut self, str: &str) {
        self.str.push('\t');
        self.pln(str);
    }

    fn pln(&mut self, str: &str) {
        self.str.push_str(str);
        self.str.push('\n');
    }
}

trait AddrSize {
    fn get_size(&self) -> u32;
}

impl AddrSize for DataType {
    fn get_size(&self) -> u32 {
        match self {
            DataType::Int => 4,
            _ => panic!("not supported: {:?}", self),
        }
    }
}
mod gen_util {
    use crate::ast::FuncDecl;

    use super::AddrSize;

    pub fn size_args_local(func: &FuncDecl) -> u32 {
        func.params.iter().map(|p| p.data_type.get_size()).sum()
    }

    pub fn get_sp_offset(size: u32) -> u32 {
        16 * (size / 16 + (if size % 16 == 0 { 0 } else { 1 }))
    }
}

#[cfg(test)]
mod test {
    use crate::{parse, scan, util::TargetOs};
    use test_case::test_case;

    use super::{gen_asm, gen_util};

    #[test]
    fn expect_header_linux() {
        let v = gen_asm(&parse(scan("int main(){return 1;}")), &TargetOs::Linux);
        vec![
            ".text",
            ".global main",
            "main:",
            ".p2align 2",
            "mov x0, #1",
            "ret",
        ]
        .iter()
        .for_each(|i| {
            if !v.contains(i) {
                panic!("'{}' is not generated", i)
            }
        });
    }

    // single function -> emit directives
    #[test_case("int main(){return 1;}", vec![
        ".text",
        ".global _main",
        ".p2align 2",
        "_main:",
        "mov x0, #1",
        "ret",
    ])]

    // function with arguments
    #[test_case("int foo(int x, int y) {}", vec![
        "sub sp, sp, #16",
        "str x0, [sp, #12]",
        "str x1, [sp, #8]",
        "add sp, sp, #16",
        "ret",
    ])]

    // function call
    #[test_case("int foo(int x, int y) {} int main() { return foo(3,4);}", vec![
        "stp x29, x30, [sp, #-16]!",
        "mov x29, sp",
        "mov x0, #3",
        "mov x1, #4",
        "bl _foo",
        "ldp x29, x30, [sp], #16"
    ])]
    fn test_function_with_args(src: &str, vec: Vec<&str>) {
        let v = gen_asm(&parse(scan(src)), &TargetOs::MacOs);
        vec.iter().for_each(|i| {
            if !v.contains(i) {
                panic!("'{}' is not generated", i)
            }
        });
    }

    #[test_case(10, 16)]
    #[test_case(16, 16)]
    #[test_case(20, 32)]
    #[test_case(32, 32)]
    #[test_case(33, 48)]
    fn test_get_sp_offset(size: u32, expected: u32) {
        assert_eq!(gen_util::get_sp_offset(size), expected);
    }
}
