use std::{cell::Cell, collections::HashMap, path::PathBuf};

use crate::{
    language::{
        cst::{Child, Tree, TreeKind, TreeKind::*},
        token::{Position, Token, TokenKind, TokenKind::*},
    },
    message::{Content, Message, Severity},
    phase::{Phase, PhaseResult},
};

///// GRAMMAR /////

// expression -> binary_op | application
fn expression(p: &mut Parser) {
    let mut lhs = binary_op(p, Eof);
    loop {
        match p.nth(0) {
            Eof | KeywordIn | KeywordThen | KeywordElse | ParenR => break,
            _ => {
                let m = p.open_before(lhs);
                expression_delimited(p);
                lhs = p.close(m, Application)
            }
        }
    }
}

fn binary_op(p: &mut Parser, left: TokenKind) -> MarkClosed {
    let mut lhs = expression_delimited(p);

    match p.nth(0) {
        Plus | Minus => loop {
            let right = p.nth(0);
            if right_binds_tighter(left.clone(), right.clone()) {
                let m = p.open_before(lhs);
                p.advance();
                binary_op(p, right);
                lhs = p.close(m, BinaryOp);
            } else {
                break;
            }
        },
        _ => {}
    }

    lhs
}

fn right_binds_tighter(left: TokenKind, right: TokenKind) -> bool {
    fn tightness(kind: TokenKind) -> Option<usize> {
        [
            // Precedence table
            &[Plus, Minus],
            // &[Start, Slash],
        ]
        .iter()
        .position(|l| l.contains(&kind))
    }

    let Some(right_tightness) = tightness(right) else {
        return false;
    };

    let Some(left_tightness) = tightness(left.clone()) else {
        assert!(left == Eof);
        return true;
    };

    right_tightness > left_tightness
}

// expression_delimited -> let_binding | if_expression | abstraction | paren | literal | name;
fn expression_delimited(p: &mut Parser) -> MarkClosed {
    match p.nth(0) {
        Backslash => abstraction(p),
        KeywordLet => let_binding(p),
        KeywordIf => if_expression(p),
        ParenL => paren(p),
        TokenKind::Name(_) => name(p),
        LiteralInt(_) | LiteralBool(_) => literal(p),
        _ => {
            let m = p.open();
            if !p.eof() {
                p.advance()
            }
            p.close(m, ErrorTree)
        }
    }
}

// abstraction -> '\' name '->' expression;
fn abstraction(p: &mut Parser) -> MarkClosed {
    assert!(p.at(Backslash));
    let m = p.open();
    p.expect(Backslash);
    name(p);
    p.expect(Arrow);
    expression(p);
    p.close(m, Abstraction)
}

// let_binding -> 'let' name '=' expression 'in';
fn let_binding(p: &mut Parser) -> MarkClosed {
    assert!(p.at(KeywordLet));
    let m = p.open();
    p.expect(KeywordLet);
    name(p);
    p.expect(Equal);
    expression(p);
    p.expect(KeywordIn);
    expression(p);
    p.close(m, Let)
}

// if_expression -> 'if' expression 'then' expression 'else' expression;
fn if_expression(p: &mut Parser) -> MarkClosed {
    assert!(p.at(KeywordIf));
    let m = p.open();
    p.expect(KeywordIf);
    expression(p);
    p.expect(KeywordThen);
    expression(p);
    p.expect(KeywordElse);
    expression(p);
    p.close(m, If)
}

// paren -> '(' expression ')';
fn paren(p: &mut Parser) -> MarkClosed {
    assert!(p.at(ParenL));
    let m = p.open();
    p.expect(ParenL);
    expression(p);
    p.expect(ParenR);
    p.close(m, Expr)
}

fn literal(p: &mut Parser) -> MarkClosed {
    let m = p.open();
    p.advance();
    p.close(m, Literal)
}

fn name(p: &mut Parser) -> MarkClosed {
    let m = p.open();
    p.advance();
    p.close(m, TreeKind::Name)
}

///////// PARSER //////////
#[derive(Copy, Clone, PartialEq, Debug)]
enum Event {
    Open { kind: TreeKind },
    Close,
    Advance,
}

#[derive(Copy, Clone, PartialEq)]
struct MarkOpened {
    index: usize,
}

#[derive(Copy, Clone, PartialEq)]
struct MarkClosed {
    index: usize,
}

pub struct Parser {
    source_path: PathBuf,
    errors: Vec<Message>,
    tokens: Vec<Token>,
    pos: usize,
    fuel: Cell<u32>,
    events: Vec<Event>,
}

impl Parser {
    fn from(source_path: &PathBuf, tokens: &Vec<Token>) -> Self {
        Self {
            source_path: source_path.clone(),
            tokens: tokens.clone(),
            errors: Vec::new(),
            pos: 0,
            fuel: Cell::new(256),
            events: Vec::new(),
        }
    }

    fn open(&mut self) -> MarkOpened {
        let mark = MarkOpened {
            index: self.events.len(),
        };
        self.events.push(Event::Open { kind: ErrorTree });
        mark
    }

    fn close(&mut self, m: MarkOpened, kind: TreeKind) -> MarkClosed {
        self.events[m.index] = Event::Open { kind };
        self.events.push(Event::Close);
        MarkClosed { index: m.index }
    }

    // TODO: This insert is O(n).
    // It could be remedied by instead setting a Option<usize> on the open event that
    // the new event is supposed to happen before and just pushing the new one onto events.
    // This makes a pseudo-linked-list. build_tree needs to follow these pointers then.
    fn open_before(&mut self, m: MarkClosed) -> MarkOpened {
        let mark = MarkOpened { index: m.index };
        self.events.insert(
            m.index,
            Event::Open {
                kind: TreeKind::ErrorTree,
            },
        );
        mark
    }

    fn advance(&mut self) {
        assert!(!self.eof());
        self.fuel.set(256);
        self.events.push(Event::Advance);
        self.pos += 1;
    }

    fn eof(&self) -> bool {
        self.pos == self.tokens.len()
    }

    fn position(&self) -> Position {
        self.tokens
            .get(self.pos)
            .expect("token to be available")
            .position
    }

    fn nth(&self, lookahead: usize) -> TokenKind {
        if self.fuel.get() == 0 {
            panic!("parser is stuck")
        }

        self.fuel.set(self.fuel.get() - 1);
        self.tokens
            .get(self.pos + lookahead)
            .map_or(Eof, |it| it.kind.clone())
    }

    fn at(&self, kind: TokenKind) -> bool {
        std::mem::discriminant(&self.nth(0)) == std::mem::discriminant(&kind)
    }

    fn eat(&mut self, kind: TokenKind) -> bool {
        if self.at(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, kind: TokenKind) {
        if self.eat(kind.clone()) {
            return;
        }

        self.errors.push(Message {
            severity: Severity::Error,
            position: self.position(),
            source_path: self.source_path.clone(),
            content: Content {
                message: format!("expected '{}'", kind),
                indicator_message: Some("here".to_string()),
                fix_hint: None,
            },
        });
    }

    fn build_tree(&mut self) -> Tree {
        let mut stack = Vec::new();
        let mut tokens = self.tokens.iter();

        assert!(matches!(self.events.pop(), Some(Event::Close)));

        for event in self.events.iter() {
            match event {
                Event::Open { kind } => stack.push(Tree {
                    kind: *kind,
                    children: Vec::new(),
                }),

                Event::Close => {
                    let tree = stack.pop().unwrap();
                    stack.last_mut().unwrap().children.push(Child::Tree(tree))
                }

                Event::Advance => {
                    let token = tokens.next().unwrap();
                    stack
                        .last_mut()
                        .unwrap()
                        .children
                        .push(Child::Token(token.clone()))
                }
            }
        }

        assert!(stack.len() == 1);
        // assert!(tokens.next().is_some_and(|t| matches!(t.kind, Eof)));

        stack.pop().unwrap()
    }

    fn parse(&mut self) -> Tree {
        expression(self);
        self.build_tree()
    }
}

pub type Input = crate::phase::lexer::Output;
pub type Output = HashMap<PathBuf, Tree>;
impl Phase<Input, Output> for Parser {
    fn new() -> Self {
        Parser::from(&PathBuf::new(), &Vec::new())
    }

    fn run(self: &mut Self, _config: &crate::config::Config, input: &Input) -> PhaseResult<Output> {
        let mut out = HashMap::new();
        let mut errs = HashMap::new();

        for (source_path, tokens) in input {
            *self = Parser::from(source_path, tokens);
            let cst = self.parse();
            if !self.errors.is_empty() {
                errs.insert(source_path.clone(), self.errors.clone());
            }

            out.insert(source_path.clone(), cst);
        }

        if !errs.is_empty() {
            PhaseResult::SoftErr(out, errs)
        } else {
            PhaseResult::Ok(out)
        }
    }
}
