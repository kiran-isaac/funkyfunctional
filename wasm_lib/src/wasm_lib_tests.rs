use super::*;

#[test]
fn eta_reduced_length() {
    let program = r#"
    data List a = Cons a (List a) | Nil

    foldr :: (a -> b -> b) -> b -> List a -> b
    foldr f acc list = match list :: List a {
      | Nil -> acc
      | Cons x xs -> f x (foldr f acc xs)
    }

    length2 :: List a -> Int
    length2 = foldr (\_ i.i + 1) 0

    main :: Int
    main = length2 Nil // $ filter isEven $ range 1 (Just 5)"#;

    let mut ast_info = parse_no_prelude(program).unwrap();
    let rcs = unsafe {get_one_redex(&ast_info)};
    ast_info = unsafe {pick_rc_and_free(&mut ast_info, rcs, 0)};
    println!("{}", unsafe { main_to_string(&ast_info)} );

    let rcs = unsafe {get_one_redex(&ast_info)};
    ast_info = unsafe {pick_rc_and_free(&mut ast_info, rcs, 0)};
    println!("{}", unsafe { main_to_string(&ast_info)} );

    let rcs = unsafe {get_one_redex(&ast_info)};
    ast_info = unsafe {pick_rc_and_free(&mut ast_info, rcs, 0)};
    println!("{}", unsafe { main_to_string(&ast_info)} );
}