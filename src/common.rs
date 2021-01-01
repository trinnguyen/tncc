use std::fmt::{self, Display};

/// Token for ANSI C grammar
#[derive(Debug)]
pub struct Token {
    /// token type with optional value (for id, number)
    pub tok: TokType,
    /// location (line,column) starting from 1
    pub loc: (u32, u32),
}

/// Token type with attached value
#[derive(Debug, PartialEq, PartialOrd)]
pub enum TokType {
    KeywordVoid,   // 'void'
    KeywordInt,    // 'int'
    KeywordReturn, // 'return'
    ID(String),    // Identifier
    NumInt(u64),   // 0, 1
    NumReal(f64),  // 0.1, 1.1
    ParentOpen,    // (
    ParentClose,   // )
    BracketOpen,   // {
    BracketClose,  // }
    Semicolon,     // ;
    Minus,         // -
    Plus,          // +
    Assign,        // =
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at {}:{}", self.tok, self.loc.0, self.loc.1)
    }
}

impl Display for TokType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s: &str = match self {
            TokType::KeywordVoid => "void",
            TokType::KeywordInt => "int",
            TokType::KeywordReturn => "return",
            TokType::ParentOpen => "(",
            TokType::ParentClose => ")",
            TokType::BracketOpen => "{",
            TokType::BracketClose => "}",
            TokType::Assign => "=",
            TokType::Semicolon => ";",
            TokType::ID(id) => return write!(f, "identifier '{}'", id),
            _ => return write!(f, "{:?}", self),
        };
        write!(f, "{}", s)
    }
}
