//! Symbol table for the AST to support decorating the AST

use std::{
    collections::{hash_map::Entry, HashMap},
    fmt::Display,
};

use crate::ast::{FuncDecl, GlobalVarDecl, ParamDecl, VarDecl};

#[derive(Debug)]
pub struct SymTable<'a> {
    stack: Vec<SymScope<'a>>,
}

impl<'a> SymTable<'a> {
    pub const fn new() -> Self {
        SymTable { stack: Vec::new() }
    }

    pub fn push_scope(&mut self) {
        let scope = SymScope::new();
        debug!("push \n{}", self);
        self.stack.push(scope);
    }

    pub fn pop_scope(&mut self) {
        debug!("pop \n{}", self);
        let _ = self.stack.pop();
    }

    pub fn cur_scope(&mut self) -> &mut SymScope<'a> {
        let len = self.stack.len();
        self.stack.get_mut(len - 1).unwrap()
    }
}

#[derive(Debug)]
pub struct SymScope<'a> {
    map: HashMap<String, DeclRef<'a>>,
}

impl<'a> SymScope<'a> {
    pub fn new() -> Self {
        SymScope {
            map: HashMap::new(),
        }
    }

    pub fn insert_decl<T>(&mut self, name: &str, decl: &'a T)
    where
        T: DeclRefCreation<'a>,
    {
        match self.map.entry(name.to_string()) {
            Entry::Occupied(v) => {
                panic!("{} is already define as {}", name, v.get().format_type());
            }
            Entry::Vacant(_) => {
                let v: DeclRef<'a> = decl.to_decl_ref();
                self.map.insert(name.to_string(), v);
            }
        };
    }

    pub fn lookup_decl<T>(&self, name: &str) -> Option<&DeclRef> {
        self.map.get(name)
    }
}

#[derive(Debug)]
pub enum DeclRef<'a> {
    GlobalVar(&'a GlobalVarDecl),
    Var(&'a VarDecl),
    Param(&'a ParamDecl),
    Func(&'a FuncDecl),
}

impl<'a> DeclRef<'a> {
    fn format_type(&self) -> &str {
        match self {
            DeclRef::GlobalVar(_) => "global variable",
            DeclRef::Var(_) => "local variable",
            DeclRef::Param(_) => "function parameter",
            DeclRef::Func(_) => "funcation",
        }
    }
}

impl<'a> Display for SymTable<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v = self
            .stack
            .iter()
            .rev()
            .map(|s| format!("{}", s))
            .fold(String::from("----------------------"), |acc, v| {
                acc + v.as_str() + "\n----------------------"
            });
        write!(f, "{}", v)
    }
}

impl<'a> Display for SymScope<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v = self
            .map
            .iter()
            .map(|(k, v)| format!("{} -> {}", k, v))
            .fold(String::new(), |acc, v| acc + "\n" + v.as_str());
        write!(f, "{}", v)
    }
}

impl<'a> Display for DeclRef<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name: &str = match self {
            DeclRef::GlobalVar(d) => &d.1,
            DeclRef::Var(d) => &d.1,
            DeclRef::Param(d) => &d.name,
            DeclRef::Func(d) => &d.name,
        };
        write!(f, "{} '{}'", self.format_type(), name)
    }
}

pub trait DeclRefCreation<'a> {
    fn to_decl_ref(&'a self) -> DeclRef<'a>;
}

impl<'a> DeclRefCreation<'a> for FuncDecl {
    fn to_decl_ref(&'a self) -> DeclRef<'a> {
        DeclRef::Func(self)
    }
}

impl<'a> DeclRefCreation<'a> for VarDecl {
    fn to_decl_ref(&'a self) -> DeclRef<'a> {
        DeclRef::Var(self)
    }
}

impl<'a> DeclRefCreation<'a> for GlobalVarDecl {
    fn to_decl_ref(&'a self) -> DeclRef<'a> {
        DeclRef::GlobalVar(self)
    }
}

impl<'a> DeclRefCreation<'a> for ParamDecl {
    fn to_decl_ref(&'a self) -> DeclRef<'a> {
        DeclRef::Param(self)
    }
}
