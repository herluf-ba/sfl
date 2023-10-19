use std::fmt::Debug;

use crate::language::token::Token;

#[derive(Clone, PartialEq, Debug)]
pub enum Ast {
    Err,
    Expr(Box<Ast>),
    Abstraction(Token, Box<Ast>),
    Application(Box<Ast>, Box<Ast>),
    Literal(Token),
    Let(Token, Box<Ast>, Box<Ast>),
    Name(Token),
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
        };

        let children_str = match self {
            Ast::Expr(e) => vec![e.print(level + 1)],
            Ast::Abstraction(t, e) => vec![format!("  '{}'", t.text()), e.print(level + 1)],
            Ast::Application(e1, e2) => vec![e1.print(level + 1), e2.print(level + 1)],
            Ast::Let(t, e1, e2) => vec![
                format!("  '{}'", t.text()),
                e1.print(level + 1),
                e2.print(level + 1),
            ],
            _ => Vec::new(),
        }
        .iter()
        .map(|s| format!("{inset}{s}"))
        .collect::<Vec<String>>()
        .join("\n");

        format!("{inset}{kind_str}\n{children_str}")
    }

    pub fn pretty_print(&self) -> String {
        self.print(0)
    }
}
