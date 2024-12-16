use crate::parser::*;

fn test_lex(str: String) -> Result<Vec<Token>, LexerError> {
    let mut lexer = Lexer::new(str, None);
    let mut tokens = Vec::new();
    loop {
        let token = lexer.get_token().unwrap();
        tokens.push(token.clone());
        if token.tt == TokenType::EOF {
            break;
        }
    }
    Ok(tokens)
}

macro_rules! lexer_tokentype_test {
    ($x:expr, $y:expr) => {
        let desired: Vec<TokenType> = $y;
        let tokens = test_lex($x.to_string()).unwrap();

        assert_eq!(tokens.len(), desired.len());
        for (i, token) in tokens.iter().enumerate() {
            assert_eq!(token.tt, desired[i]);
        }
    };
}

#[test]
fn int_lit() {
    lexer_tokentype_test!(
        "123 12 23 0",
        vec![
            TokenType::IntLit,
            TokenType::IntLit,
            TokenType::IntLit,
            TokenType::IntLit,
            TokenType::EOF
        ]
    );
}

#[test]
fn float_lit() {
    lexer_tokentype_test!(
        "123.0 12.0 23.0 0.0 0. .0",
        vec![
            TokenType::FloatLit,
            TokenType::FloatLit,
            TokenType::FloatLit,
            TokenType::FloatLit,
            TokenType::FloatLit,
            TokenType::FloatLit,
            TokenType::EOF
        ]
    );
}
