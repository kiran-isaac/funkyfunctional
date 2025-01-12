use sfl_lib::{self as lib, infer_or_check_assignment_types, LabelTable};
use std::{
    env, fs,
    io::{self, Write},
};

static HORIZONTAL_SEPARATOR: &str =
    "______________________________________________________________";

fn main() {
    let argv: Vec<String> = env::args().collect();

    let (file_path, typechecked) = if argv.len() == 2 {
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
        let module = ast.root;
        infer_or_check_assignment_types(&mut ast, module).unwrap_or_else(|e| {
            println!(
                "INPUT:\n\n{}\n{}",
                ast.to_string_sugar(ast.root, true),
                HORIZONTAL_SEPARATOR
            );
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

    println!(
        "INPUT:\n\n{}\n{}",
        ast.to_string_sugar(ast.root, true),
        HORIZONTAL_SEPARATOR
    );

    let mut expr = ast.get_assign_exp(match ast.get_main(ast.root) {
        Some(v) => v,
        None => {
            eprintln!("Main not found");
            std::process::exit(1);
        },
    });

    let mut rcs = lib::find_redex_contraction_pairs(&ast, Some(ast.root), expr, &lt);
    let mut rcs_filtered = ast.filter_identical_rcs(&rcs);

    println!("{}", ast.to_string_sugar(expr, false));

    while rcs.len() != 0 {
        for (i, rc) in rcs_filtered.iter().enumerate() {
            let s1 = ast.to_string_sugar(rc.0, false);
            let s2 = rc.1.to_string_sugar(rc.1.root, false);
            println!("{}) {} => {}", i + 1, s1, s2);
        }

        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        input = input.trim().to_string();
        let choice = if input == "" {
            &ast.get_laziest_rc(expr, &rcs_filtered).unwrap()
        } else {
            let num = input.trim().parse::<usize>().unwrap();
            if num > rcs.len() {
                eprintln!("Invalid choice\n");
                continue;
            }
            &rcs_filtered[num - 1]
        };

        ast.do_rc_subst_and_identical_rcs(choice, &rcs);

        expr = ast.get_assign_exp(match ast.get_main(ast.root) {
            Some(v) => v,
            None => {
                eprintln!("Main not found");
                std::process::exit(1);
            },
        });

        rcs = lib::find_redex_contraction_pairs(&ast, Some(ast.root), expr, &lt);
        rcs_filtered = ast.filter_identical_rcs(&rcs);
        println!("\n{}", ast.to_string_sugar(expr, false));
    }
}
