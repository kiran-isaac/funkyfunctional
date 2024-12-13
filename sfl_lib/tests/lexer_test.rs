use sfl_lib::Lexer;

fn main() {
    let mut lexer = Lexer::new("test".to_string(), "a b c".to_string());
    let token = lexer.get_token();
    println!("{:?}", token);
}