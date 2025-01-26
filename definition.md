# Welcome
This is an interactive term rewrite system for a simple functional language. The language is a lambda calculus with integers, floats, booleans, pairs and if-then-else expressions. The language is statically typed with type inference. The language is designed to be simple and easy to understand, and is a good starting point for learning about functional programming.

This is a Bachelors disertation project written by Kiran Sturt, at University of Bristol. Please get in touch with any feedback or questions at kiran.sturt@bristol.ac.uk.

## How to use
Enter your program into the code editor, and press "run". Your program will be type checked, and types inferred where not provided. The type checked program will be displayed below the box, showing what types have been inferred. An error may appear here instead, apologies for the type errors being awful I am working on it!

After this, you will be presented with some buttons, representing the "next steps" the system has detected for you. The left hand size is the current expression, and the right hand side is the next step for this expression.

You can click on these buttons to step through the evaluation of your program. The top button is labled "laziest" and it will automatically take the laziest step for you.

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