pub enum TokenType {
    Identifier,

    LParen,
    RParen,

    Assignment,
}

pub struct Token {
    pub tt: TokenType,
    pub value: String,
}