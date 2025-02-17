// use crate::{
//     find_redexes::{reduce::*, RCPair},
//     functions::LabelTable,
//     ASTNodeType, Parser, AST,
// };

/// O(n^2) so only use for small things
#[allow(unused)]
fn assert_eq_in_any_order<T: PartialEq>(a: &Vec<T>, b: &Vec<T>) {
    for x in a {
        let mut found = false;
        for y in b {
            if x == y {
                found = true;
            }
        }
        assert!(found);
    }
}

// fn rc_pair_to_string(ast: &AST, rc: &RCPair) -> String {
//     format!("{} => {:?}", ast.to_string(rc.0), rc.1)
// }

// #[test]
// fn zero_ary_test() {
//     let program = "main = zero_ary_test";
//     let ast = Parser::from_string(program.to_string())
//         .parse_module()
//         .unwrap();

//     let module = ast.root;
//     let exp = ast.get_assign_exp(ast.get_main(module));

//     let rcs = find_redex_contraction_pairs(&ast, module, exp, &LabelTable::new());
//     assert!(rcs.len() == 1);

//     let rc = &rcs[0];
//     let redex = ast.get(rc.0);
//     let result = rc.1.get(rc.1.root);

//     assert!(redex.t == ASTNodeType::Identifier);
//     assert!(redex.get_value() == "zero_ary_test");
//     assert!(result.t == ASTNodeType::Literal);
//     assert!(result.get_value() == "0");
// }

// #[test]
// fn unary_neg_test() {
//     for _ in 0..1000 {
//         let rnd_i = rand::random::<i64>();
//         let rnd_f = rand::random::<f64>();

//         let program = format!("main = neg {}", rnd_i);
//         let programf = format!("main = negf {}", rnd_f);

//         let ast = Parser::from_string(program).parse_module().unwrap().ast;
//         let astf = Parser::from_string(programf).parse_module().unwrap().ast;

//         let module = ast.root;
//         let exp = ast.get_assign_exp(ast.get_main(module));

//         let modulef = astf.root;
//         let expf = astf.get_assign_exp(astf.get_main(modulef));

//         let rcs = find_redex_contraction_pairs(&ast, module, exp, &LabelTable::new());
//         let rcsf = find_redex_contraction_pairs(&astf, modulef, expf, &LabelTable::new());
//         assert!(rcs.len() == 1);
//         assert!(rcsf.len() == 1);

//         assert_eq!(
//             rc_pair_to_string(&ast, &rcs[0]),
//             format!("neg {} => {}", rnd_i, -rnd_i)
//         );
//         assert_eq!(
//             rc_pair_to_string(&astf, &rcsf[0]),
//             format!("negf {} => {}", rnd_f, -rnd_f)
//         );
//     }
// }

use crate::find_redexes::reduce::find_single_redex_contraction_pair;
use crate::{find_all_redex_contraction_pairs, typecheck, KnownTypeLabelTable, Parser};

#[test]
fn basic_add_test() {
    let program = "main :: Int\nmain = add 5 1";
    let ast = Parser::from_string(program.to_string())
        .parse_module(false)
        .unwrap()
        .ast;

    let module = ast.root;
    let exp = ast.get_assign_exp(ast.get_main(module).unwrap());

    let rcs =
        find_all_redex_contraction_pairs(&ast, Some(module), exp, &KnownTypeLabelTable::new());
    assert!(rcs.len() == 1);
    println!("{}", ast.rc_to_str(&rcs[0]));
}

#[test]
fn waits_for_eval() {
    let program = "func :: a -> a\nfunc x = x\nmain :: Int\nmain = func (add 5 1)";
    let pr = Parser::from_string(program.to_string())
        .parse_module(false)
        .unwrap();
    let mut ast = pr.ast;
    let mut lt = pr.lt;
    let tm = pr.tm;

    let module = ast.root;
    let exp = ast.get_assign_exp(ast.get_main(module).unwrap());

    typecheck(&mut ast, module, &mut lt, &tm).unwrap();

    let rcs = find_single_redex_contraction_pair(&ast, Some(module), exp, &lt).unwrap();
    println!("{}", ast.rc_to_str(&rcs));
}

#[test]
fn silent_if() {
    let program = "main :: Int\nmain = if true then 1 else 2";
    let pr = Parser::from_string(program.to_string())
        .parse_module(true)
        .unwrap();
    let mut ast = pr.ast;
    let mut lt = pr.lt;
    let tm = pr.tm;

    let module = ast.root;
    let exp = ast.get_assign_exp(ast.get_main(module).unwrap());

    typecheck(&mut ast, module, &mut lt, &tm).unwrap();

    let rcs = find_single_redex_contraction_pair(&ast, Some(module), exp, &lt).unwrap();
    println!("{}", ast.rc_to_str(&rcs));
}

// #[test]
// fn correct_abst_order() {
//     let program = "test f x y z = f x\nmain = test (\\x . x) 1 2 3";
//     // let program = "main = (\\f x y z. f x) id 1 2 3";
//     let pr = Parser::from_string(program.to_string())
//         .parse_module(false)
//         .unwrap();
//     let mut ast = pr.ast;
//     let mut lt = pr.lt;
//     let tm = pr.tm;
//
//     let module = ast.root;
//     let exp = ast.get_assign_exp(ast.get_main(module).unwrap());
//
//     infer_or_check_assignment_types(&mut ast, module, &mut lt, &tm).unwrap();
//
//     let rcs = find_single_redex_contraction_pair(&ast, Some(module), exp, &lt).unwrap();
//     println!("{}", ast.rc_to_str(&rcs));
// }

// #[test]
// fn multi_op_test() {
//     let a_int = rand::random::<u16>() as i64;
//     let b_int = rand::random::<u16>() as i64;
//     let c_int = rand::random::<u16>() as i64;
//     let d_int = rand::random::<u16>() as i64;
//     let program = format!(
//         "main = sub (add {} {}) (mul {} {})",
//         a_int, b_int, c_int, d_int
//     );
//     let mut ast = Parser::from_string(program).parse_module().unwrap().ast;

//     let module = ast.root;
//     let exp = ast.get_assign_exp(ast.get_main(module).unwrap());

//     let rcs = find_redex_contraction_pairs(&ast, Some(module), exp, &LabelTable::new());

//     let correct = vec![
//         format!("add {} {} => {}", a_int, b_int, a_int + b_int),
//         format!("mul {} {} => {}", c_int, d_int, c_int * d_int),
//     ];

//     let proposed: Vec<String> = rcs
//         .clone()
//         .into_iter()
//         .map(|rc| ast.rc_to_str(&rc))
//         .collect();

//     assert_eq_in_any_order(&correct, &proposed);

//     for (old, new) in rcs {
//         ast.do_rc_subst(&(old, new));
//     }

//     let rcs = find_redex_contraction_pairs(&ast, Some(module), exp, &LabelTable::new());
//     assert!(rcs.len() == 1);
//     for (old, new) in rcs {
//         ast.do_rc_subst(&(old, new));
//     }

//     assert_eq!(
//         format!("main = {}", (a_int + b_int) - (c_int * d_int)),
//         format! {"{:?}", ast}
//     )
// }

// #[test]
// fn inc_test() {
//     let program = "inc::Int -> Int\ninc = \\x.add 1 x\nmain::Int\nmain = inc 2".to_string();

//     let ast = Parser::from_string(program).parse_module().unwrap().ast;

//     TypeChecker::new().check_module(&ast, ast.root).unwrap();

//     let module = ast.root;
//     let exp = ast.get_assign_exp(ast.get_main(module));

//     let mut lt = LabelTable::new();
//     lt.consume_from_module(&ast, module).unwrap();

//     let rcs = find_redex_contraction_pairs(&ast, module, exp, &lt);

//     assert_eq!(rcs.len(), 1);

//     assert_eq!("inc 2 => add 1 2", rc_pair_to_string(&ast, &rcs[0]));
// }

#[test]
fn myadd_test() {
    let program =
        "myadd::Int -> Int -> Int\nmyadd = \\x y.add x y\nmain::Int\nmain = myadd 2 3".to_string();

    let pr = Parser::from_string(program).parse_module(false).unwrap();
    let ast = pr.ast;
    let lt = pr.lt;

    let module = ast.root;
    let exp = ast.get_assign_exp(ast.get_main(module).unwrap());

    let rc = find_single_redex_contraction_pair(&ast, Some(module), exp, &lt).unwrap();

    assert_eq!("myadd 2 3 -> add 2 3", ast.rc_to_str(&rc));
}

#[test]
fn redexes_match() {
    let program = r#"
    data List a = Cons a (List a) | Nil

    main :: Bool
    main = match (Cons (5) Nil) {
      | Nil -> true
      | Cons _ _ -> false
    }"#;

    let pr = Parser::from_string(program.to_string())
        .parse_module(false)
        .unwrap();

    let mut ast = pr.ast;
    let mut lt = pr.lt;
    let tm = pr.tm;

    let module = ast.root;
    let exp = ast.get_assign_exp(ast.get_main(module).unwrap());
    typecheck(&mut ast, module, &mut lt, &tm).unwrap();

    let rc = find_single_redex_contraction_pair(&ast, Some(module), exp, &lt).unwrap();
    assert_eq!("false", rc.1.to_string_sugar(rc.1.root, false));
}
