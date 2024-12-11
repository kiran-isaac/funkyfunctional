use pest::Parser;
use sfl_lib::parser::Rule;
use sfl_lib::parser::SflParser;
use super::common::test_parser;

test_parser!(
    file_fail, 
    Rule::File, 
    ["\"", "#"], 
    false
);

test_parser!(
    file_pass, 
    Rule::File, 
    ["", "\n// Comment", "/*\n\nMulti Line Comment\n*/"], 
    true
);

test_parser!(
    identifier_pass,
    Rule::Identifier,
    [
        "hello",
        "scrongleBongle",
        "camelCase",
        "snake_case",
        "ending_underscore_"
    ],
    true
);

test_parser!(
    identifier_fail,
    Rule::Identifier,
    ["", "#", "StartingCap", "_startingUnderscore"],
    false
);