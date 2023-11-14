use std::fmt::Debug;

use crate::language::token::Token;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum TreeKind {
    ErrorTree,
    File,
    Expr,
    Definition,
    Params,
    Param,
    Call,
    Args,
    Arg,
    TypeExpr,
    Literal,
    Binary,
    If,
    Let,
    Name,
    Block,
    Statement,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Child {
    Token(Token),
    Tree(Tree),
}

#[derive(Clone, PartialEq, Debug)]
pub struct Tree {
    pub kind: TreeKind,
    pub children: Vec<Child>,
}

impl Tree {
    #[allow(dead_code)]
    fn print(&self, level: usize) -> String {
        let inset = "  ".repeat(level).to_string();
        let children = self
            .children
            .iter()
            .map(|c| match c {
                Child::Token(t) => format!("{inset}  '{}'", t.text()),
                Child::Tree(t) => t.print(level + 1),
            })
            .collect::<Vec<String>>()
            .join("\n");

        format!("{inset}{:?}\n{}", self.kind, children).to_string()
    }

    #[allow(dead_code)]
    pub fn pretty_print(&self) -> String {
        self.print(0)
    }
}
