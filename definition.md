# Language Specification

Float Lit (at least one of LHS and RHS must be non empty, so "1.1" "1." and ".1" are allowed but not ".")  
*`f ::= (-)?[(1..9)+.(1..9)* | (1..9)*.(1..9)+]`*

Int Lit  
*`i ::= (-)?(1..9)+`*

Boolean Literal  
*`b ::= true | false`*

Literals  
*`l ::= b | i | f`*

Identifiers (c identifier rules apply)  
*`x ::= [_a..zA..Z][_a..zA..Z0..9]`*

Expressions  
*`e ::= x | l | λx.e | e e | () | if e then e else e`*

Module (set of labled expressions)  
*`m ::= (x = e)*`*