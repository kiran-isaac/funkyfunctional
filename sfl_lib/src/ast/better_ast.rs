// This is what the AST definition should of looked like
// The overhaul is not worth it. 

struct AST {
    vec: Vec<ASTNode>,
    root: &'a ASTNode,
}

enum ASTNodeType<'a> {
    Identifier{name: String},
    Literal{value: String, _type: PrimitiveType},
    Pair{first: &'a ASTNode, second: &'a ASTNode},
    Application{f: &'a ASTNode, x: &'a ASTNode},
    Assignment{to: String, expr: &'a ASTNode},
    Abstraction{var: String, expr: &'a ASTNode},
    Module{assigns: Vec<&'a ASTNode>},
    Match{expr: &'a ASTNode, cases: Vec<&'a ASTNode>}
} 

// Line and Col specified here.
struct ASTNodeSyntaxInfo { }

struct ASTNode<'a> {
    t: ASTNodeType<'a>,
    info: ASTNodeSyntaxInfo
}

// struct ASTNodeSyntaxInfo {...}

// struct ASTNode {
//     t: ASTNodeType,
//     token: Option<Token>,
//     children: Vec<usize>,
//     line: usize,
//     col: usize,
//     type_assignment: Option<Type>,
//     additional_syntax_information: ASTNodeSyntaxInfo
// }