use std::fmt::Debug;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TokenType {
    EOF,
    Newline,

    Id,
    TypeId,

    IntLit,
    FloatLit,
    StringLit,
    CharLit,
    BoolLit,

    DoubleColon,
    RArrow,

    LParen,
    RParen,

    Lambda,

    Dot,

    Assignment,
}

#[derive(Clone)]
pub struct Token {
    pub tt: TokenType,
    pub value: String,
}

impl Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.tt {
            TokenType::IntLit
            | TokenType::CharLit
            | TokenType::StringLit
            | TokenType::FloatLit
            | TokenType::Id => write!(f, "{:?}: {}", self.tt, self.value),
            _ => write!(f, "{:?}", self.tt),
        }
    }
}
