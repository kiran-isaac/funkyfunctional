use sfl_lib::{self as lib, typecheck_module, LabelTable};
use std::{
    env, fs,
    io::{self, Write},
};

fn main() {
    let argv: Vec<String> = env::args().collect();

    let (file_path, typechecked) =     if argv.len() == 2 {
        (argv[1].clone(), false)
    } else if argv.len() == 3 {
        if argv[1].as_str() != "t" {
            eprintln!("Unrecognized argument: {}", argv[1]);
            std::process::exit(1);
        }
        (argv[2].clone(), true)
    } else {
        eprintln!("Incorrect args");
        std::process::exit(1);
    };

    let file_string = if fs::metadata(&file_path).is_ok() {
        fs::read_to_string(&file_path).expect("Failed to read file")
    } else {
        eprintln!("File does not exist: {}", file_path);
        std::process::exit(1);
    };

    let ast = lib::Parser::from_string(file_string).parse_module();
    if let Err(e) = &ast {
        eprintln!("{:?}", e);
        std::process::exit(1);
    }
    let mut ast = ast.unwrap();

    // Typecheck
    let lt = if typechecked {
        typecheck_module(&ast, ast.root).unwrap_or_else(|e| {
            eprintln!("{:?}", e);
            std::process::exit(1)
        })
    } else {
        let mut lt = LabelTable::new();
        match &lt.consume_from_module(&ast, ast.root) {
            Ok(()) => lt,
            Err(e) => panic!("{:?}", e),
        }
    };

    let exp = ast.get_assign_exp(ast.get_main(ast.root));

    let mut rcs = lib::find_redex_contraction_pairs(&ast, ast.root, exp, &lt);

    println!("{}\n", ast.to_string(ast.root));

    println!("{}", ast.to_string(exp));

    while rcs.len() != 0 {
        for (i, rc) in rcs.iter().enumerate() {
            let s1 = ast.to_string(rc.0);
            let s2 = rc.1.to_string(rc.1.root);
            println!("{}) {} => {}", i + 1, s1, s2);
        }

        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        let choice: usize = input.trim().parse().expect("Invalid input");

        if choice > rcs.len() {
            eprintln!("Invalid choice\n");
            continue;
        }
        ast.do_rc_subst(&rcs[choice - 1]);

        let exp = ast.get_assign_exp(ast.get_main(ast.root));

        rcs = lib::find_redex_contraction_pairs(&ast, ast.root, exp, &lt);
        println!("\n{}", ast.to_string(exp));
    }
}
