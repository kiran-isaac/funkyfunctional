
macro_rules! test_parser {
    ($test_name:ident, $rule:expr, $strings:expr, $should_pass:expr) => {
        #[test]
        fn $test_name() -> Result<(), String> {
            for program in $strings.iter() {
                match SflParser::parse($rule, program) {
                    Ok(_) if !$should_pass => {
                        return Err(format!("Unexpected test pass: \"{}\"", program))
                    }
                    Err(_) if $should_pass => {
                        return Err(format!("Unexpected parse failure: \"{}\"", program))
                    },
                    _ => {}
                }
            }
            Ok(())
        }
    };
}