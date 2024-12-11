use sfl_lib::parser::SflParser;
use pest::Parser;
use sfl_lib::parser::Rule;

fn parse(input : &str, level : Rule) -> Result<Rule, ()> {
  let parse_result = SflParser::parse(level, input);

  match parse_result {
    Ok(mut p) => {Ok(p.next().unwrap().as_rule())},
    Err(e) => {
      println!("{}", e.to_string());
      Err(())
    }
  }
}

#[test]
fn test1() -> Result<(), pest::error::Error<Rule>> {
  let parse_result = SflParser::parse(Rule::File, "##");

  match parse_result {
    Ok(_) => {},
    Err(e) => {println!("{}", e.to_string())}
  }

  Ok(())
}