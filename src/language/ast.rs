use std::fmt::Debug;

use crate::language::token::Token;

use super::token::Position;

#[derive(Clone, PartialEq, Debug)]
pub enum Ast {
    Err,
    Expr(Box<Ast>),
    Abstraction(Token, Box<Ast>),
    Application(Box<Ast>, Box<Ast>),
    Literal(Token),
    Let(Token, Box<Ast>, Box<Ast>),
    Name(Token),
    BinaryOp(Token, Box<Ast>, Box<Ast>),
}

impl Ast {
    fn print(&self, level: usize) -> String {
        let inset = "  ".repeat(level).to_string();
        if let Ast::Name(t) = self {
            return format!("{inset}Name '{}'", t.text());
        }

        if let Ast::Literal(t) = self {
            return format!("{inset}Literal '{}'", t.text());
        }

        let kind_str = match self {
            Ast::Err => "Err",
            Ast::Expr(_) => "Expr",
            Ast::Abstraction(_, _) => "Abstraction",
            Ast::Application(_, _) => "Application",
            Ast::Literal(_) => "Literal",
            Ast::Let(_, _, _) => "Let",
            Ast::Name(_) => "Name",
            Ast::BinaryOp(_, _, _) => "BinaryOp",
        };

        let children_str = match self {
            Ast::Expr(e) => vec![e.print(level + 1)],
            Ast::Abstraction(t, e) => vec![format!("{inset}  '{}'", t.text()), e.print(level + 1)],
            Ast::Application(e1, e2) => vec![e1.print(level + 1), e2.print(level + 1)],
            Ast::Let(t, e1, e2) => vec![
                format!("{inset}  '{}'", t.text()),
                e1.print(level + 1),
                e2.print(level + 1),
            ],
            Ast::BinaryOp(op, e1, e2) => vec![
                e1.print(level + 1),
                format!("{inset}  '{}'", op.text()),
                e2.print(level + 1),
            ],
            _ => Vec::new(),
        }
        .join("\n");

        format!("{inset}{kind_str}\n{children_str}")
    }

    pub fn pretty_print(&self) -> String {
        self.print(0)
    }
}

impl Into<Position> for Ast {
    fn into(self) -> Position {
        match self {
            Ast::Err => Position {
                line: 0,
                column: 0,
                begin: 0,
                end: 0,
            },
            Ast::Expr(e) => e.as_ref().to_owned().into(),
            Ast::Abstraction(_, e) => {
                let e_pos: Position = e.as_ref().to_owned().into();
                e_pos
            }
            Ast::Application(e1, _) => {
                let e1_pos: Position = e1.as_ref().to_owned().into();
                e1_pos
            }
            Ast::Literal(t) => t.position,
            Ast::Let(n, _, _) => n.position,
            Ast::Name(t) => t.position,
            Ast::BinaryOp(op, _, _) => op.position,
        }
    }
}
