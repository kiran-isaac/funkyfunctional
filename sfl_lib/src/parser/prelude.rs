pub static PRELUDE: &str = r#"
// Use as "if _ then _ else
if :: Bool -> a -> a -> a
@if cond then_branch else_branch = match cond {
  | true -> then_branch
  | false -> else_branch
}

data Either a b = Left a | Right b
data Maybe a = Just a | Nothing
data List a = Cons a (List a) | Nil

foldr :: (a -> b -> b) -> b -> List a -> b
foldr f acc list = match list :: List a {
  | Nil -> acc
  | Cons x xs -> f x (foldr f acc xs)
}

length :: List a -> Int
length xs = foldr (\_ i. i + 1) 0 xs

range :: Int -> Maybe Int -> List Int
range lower upperMaybe = match upperMaybe :: Maybe Int {
  | Just upper  -> if lower >= upper then Nil else Cons lower (range (lower + 1) upperMaybe)
  | Nothing     -> Cons lower (range (lower + 1) upperMaybe)
}
"#;
