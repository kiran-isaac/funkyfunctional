use crate::parser::*;

#[test]
fn parse_assign() -> Result<(), ParserError> {
    let str = "x = 5";
    let mut parser = Parser::from_string(str.to_string());

    let assign = parser.parse()?;
    println!("{:?}", assign);
    
    Ok(())
}