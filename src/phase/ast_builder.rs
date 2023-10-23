use std::{collections::HashMap, path::PathBuf};

use crate::{
    language::{
        ast::Ast,
        cst::{Child, Tree, TreeKind::*},
    },
    phase::{Phase, PhaseResult},
};

pub struct AstBuilder;

fn build(tree: &Tree) -> Ast {
    match tree.kind {
        ErrorTree => Ast::Err,
        Expr => {
            let Child::Tree(ref t) = tree.children[1] else {
                return Ast::Err;
            };
            Ast::Expr(Box::new(build(&t)))
        }
        Abstraction => {
            let Child::Tree(ref n) = tree.children[1] else {
                return Ast::Err;
            };
            let Child::Token(ref t) = n.children[0] else {
                return Ast::Err;
            };
            let Child::Tree(ref e) = tree.children[3] else {
                return Ast::Err;
            };
            Ast::Abstraction(t.clone(), Box::new(build(&e)))
        }
        Application => {
            let Child::Tree(ref e1) = tree.children[0] else {
                return Ast::Err;
            };

            let Child::Tree(ref e2) = tree.children[1] else {
                return Ast::Err;
            };

            Ast::Application(Box::new(build(&e1)), Box::new(build(&e2)))
        }
        Let => {
            let Child::Tree(ref n) = tree.children[1] else {
                return Ast::Err;
            };
            let Child::Token(ref t) = n.children[0] else {
                return Ast::Err;
            };
            let Child::Tree(ref e1) = tree.children[3] else {
                return Ast::Err;
            };
            let Child::Tree(ref e2) = tree.children[5] else {
                return Ast::Err;
            };

            Ast::Let(t.clone(), Box::new(build(&e1)), Box::new(build(&e2)))
        }
        Name => {
            let Child::Token(ref v) = tree.children[0] else {
                return Ast::Err;
            };

            Ast::Name(v.clone())
        }
        Literal => {
            let Child::Token(ref v) = tree.children[0] else {
                return Ast::Err;
            };

            Ast::Literal(v.clone())
        }

        BinaryOp => {
            let Child::Tree(ref e1) = tree.children[0] else {
                return Ast::Err;
            };
            let Child::Token(ref op) = tree.children[1] else {
                return Ast::Err;
            };
            let Child::Tree(ref e2) = tree.children[2] else {
                return Ast::Err;
            };

            Ast::BinaryOp(op.clone(), Box::new(build(e1)), Box::new(build(e2)))
        }
        If => todo!(),
    }
}

pub type Input = crate::phase::parser::Output;
pub type Output = HashMap<PathBuf, Ast>;
impl Phase<Input, Output> for AstBuilder {
    fn new() -> Self {
        AstBuilder
    }

    fn run(self: &mut Self, _config: &crate::config::Config, input: &Input) -> PhaseResult<Output> {
        let mut out = HashMap::new();

        for (source_path, cst) in input {
            let ast = build(cst);
            out.insert(source_path.clone(), ast);
        }

        PhaseResult::Ok(out)
    }
}
