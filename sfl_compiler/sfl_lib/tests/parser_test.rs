use sfl_lib::parser::SflParser;
use pest::Parser;
use sfl_lib::parser::Rule;

fn parse(input : &str, level : Rule) -> Rule {
  let parse_result = SflParser::parse(level, input);

  match parse_result {
    Ok(p) => {return p.next().unwrap().se},
    Err(e) => {println!("{}", e.to_string())}
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