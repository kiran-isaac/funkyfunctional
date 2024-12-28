use sfl_lib as lib;
use std::{
    env, fs,
    io::{self, Read, Write},
    rc,
};

fn main() {
    let argv: Vec<String> = env::args().collect();

    // let argv = vec![
    //     "sfl_cli".to_string(),
    //     "../test.sfl".to_string(),
    // ];

    if argv.len() != 2 {
        eprintln!("Incorrect args");
        std::process::exit(1);
    }

    let file_path = argv[1].clone();

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

    let exp = ast.get_exp(ast.get_main(ast.root));

    let mut rcs = lib::find_redex_contraction_pairs(&ast, ast.root, exp);

    println!("{}\n", ast.to_string(ast.root));

    println!("{}", ast.to_string(exp));

    while rcs.len() != 0 {
        for (i, rc) in rcs.iter().enumerate() {
            println!("{}) {} => {:?}", i + 1, ast.to_string(rc.0), rc.1);
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

        let choice = &rcs[choice - 1];

        ast.replace_from_other_root(&choice.1, choice.0);

        let exp = ast.get_exp(ast.get_main(ast.root));

        rcs = lib::find_redex_contraction_pairs(&ast, ast.root, exp);
        println!("\n{}", ast.to_string(exp));
    }
}
