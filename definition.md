# Welcome
This is an interactive term rewrite system for a simple functional language. The language is a lambda calculus with integers, floats, booleans, pairs and if-then-else expressions. The language is statically typed with type inference. The language is designed to be simple and easy to understand, and is a good starting point for learning about functional programming.

This is a Bachelors disertation project written by Kiran Sturt, at University of Bristol. Please get in touch with any feedback or questions at kiran.sturt@bristol.ac.uk.

---

## How to use
Enter your program into the code editor, and press "run". Your program will be type checked, and types inferred where not provided. The types of all lables will be displayed below the input box. An error may appear here instead, apologies for the type errors being awful I am working on it!

Next to the text input box, you are presented with some buttons: Lazy and Free Choice. Pressing either will start the evaluation of your progra. These buttons mean
- "Lazy" : you will be presented with the laziest next step
- "Free Choice" : you will be presented with all possible next evaluation steps. Note that the first option in the list will be the laziest next step. 

Once evaluation has begun, you will see some buttons if a next step is possible. You can click on these buttons to step through the evaluation of your program. 


## Programming
The language is a lambda calculus with integers, floats, booleans, pairs and if-then-else expressions. The expression labled "main" will be evaluated. 

### Syntax
Terms
- **Integers** are written as `1`, `2`, `3`, etc.
- **Floats** are written as `1.1`, `2.2`, `3.3`, etc.
- **Booleans** are written as `true` or `false`.
- **Identifiers** are written as `x`, `y`, `z`, etc. Identifiers must start with a lowercase letter.
- **Pairs** are written as `(e1, e2)`.
- **If-then-else** expressions are written as `if e1 then e2 else e3`. `if` is typed as `if : Bool -> a -> a -> a`.
- **Lambda Abstraction** is written as `\x.e`, where `x` is the variable name and `e` is the expression. `\x y.e` is syntax sugar for `\x.\y.e`.

Types
- Inbuilt Types:
  - **Int**: 64 bit integer.
  - **Float**: 64 bit floating point number.
  - **Bool**: Boolean value.
- More Types
  - **(T1, T2)**: A pair of types.
  - **T1 -> T2**: A function from type `T1` to type `T2`.
  - **Any Lowercase Identifier**, Type variables. 
  - **Any Uppercase Identifier**: A type, or a type constructor (see user defined types).
- User Defined Types
  - **Type Aliases** : `type UppercaseIdentifier = Type`. This is syntax sugar for replacing all instances of `UppercaseIdentifier` with `Type`.
  - **Union Types** : Examples
    - `data Maybe a = Just a | Nothing`
    - `data Either a b = Left a | Right b`
    - `data List a = Nil | Cons a (List a)`
    - `data Tree a = Leaf a | Node (Tree a) (Tree a)`

---

## Language Specification

**Float Lit (at least one of LHS and RHS must be non empty, so "1.1" "1." and ".1" are allowed but not ".")**  
*`f ::= (-)?[(1..9)+.(1..9)* | (1..9)*.(1..9)+]`*

**Int Lit**  
*`i ::= (-)?(1..9)+`*

**Boolean Literal**  
*`b ::= true | false`*

**Literals**  
*`l ::= b | i | f`*

**Identifiers (c identifier rules apply)**  
*`x ::= [_a..zA..Z][_a..zA..Z0..9]`*

**Infix Operators (all operators are right associative)**  
*`o ::= + | - | * | / | < | > | <= | >= | == | != | && | ||`*

**Lambda Abstraction Variable (identifiers pairs of identifiers are possible to unpack paired expressions)**  
*`v ::= v | (v, v)`*

**Expressions (application is left associative, abstraction binds the least tight. "e1 o e1" is interpreted as "o e1 e2", e.g. "1 + 2" is parsed as "+ (+ 1 2) 3")**  
*`e ::= x | l | \v.e | e e | (e, e) | e o e | if e then e else e`*

Assignment (with optional variables before the equals sign which is syntax sugar for abstraction, e.g. `f x = e` is the same as `f = \x.e`)  
*`a ::= x (x)* = e`*

Module (set of assignments and type assignments (see more about types below), seperated by one or more newline)  
*`m ::= ([x = e | x :: T](\n)+)*`*

### Examples
*`a = 1`*  
*`b = \x . x`*  
*`first = \(x, y) . x`*  
*`second = \(x, y) . y`*  
*`pair x y = (x, y)`*  
*`fib n = if n < 2 then n else fib (n - 1) + fib (n - 2)`*  

## Types
*`T ::= forall a . T | T -> T | Bool | Int | Float | (T, T)`*

The type inference is based on "complete and easy bidirectional typechecking for higher-rank polymorphism" by Dunfield and Krishnaswami. 