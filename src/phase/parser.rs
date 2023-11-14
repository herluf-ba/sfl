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
// file = def*
fn file(p: &mut Parser) {
    let m = p.open();

    while !p.eof() {
        if p.at(TokenKind::KeywordDef) {
            def(p)
        } else {
            p.advance_with_error(Message {
                severity: Severity::Error,
                position: p.position(),
                content: Content {
                    message: "expected a function".to_string(),
                    indicator_message: Some("here".to_string()),
                    fix_hint: Some("define a function using `def`".to_string()),
                },
                source_path: p.source_path.clone(),
            });
        }
    }

    p.close(m, File);
}

fn def(p: &mut Parser) {
    assert!(p.at(KeywordDef));
    let m = p.open();

    p.expect(KeywordDef);
    name(p);
    if p.at(ParenL) {
        params(p);
    }

    if p.eat(Colon) {
        type_expr(p);
    }

    if p.eat(CurlyL) {
        statement(p);
        p.expect(CurlyR);
    }

    p.close(m, Definition);
}

fn params(p: &mut Parser) {
    assert!(p.at(TokenKind::ParenL));
    let m = p.open();

    p.expect(TokenKind::ParenL);
    while !p.at(TokenKind::ParenR) && !p.eof() {
        if p.at(TokenKind::Name("".to_string())) {
            param(p);
        } else {
            break;
        }
    }
    p.expect(TokenKind::ParenR);

    p.close(m, Params);
}

fn param(p: &mut Parser) {
    let m = p.open();
    name(p);
    p.expect(TokenKind::Colon);
    type_expr(p);
    if !p.at(TokenKind::ParenR) {
        p.expect(TokenKind::Comma);
    }

    p.close(m, Param);
}

/////// TYPES ///////
fn type_expr(p: &mut Parser) {
    let m = p.open();
    name(p);
    p.close(m, TypeExpr);
}

//////// EXPRESSIONS /////////
fn statement(p: &mut Parser) {
    let mut lhs = expression(p);
    while p.at(Semi) {
        let m = p.open_before(lhs);
        p.close(m, Statement);
        lhs = expression(p);
    }
}

fn expression(p: &mut Parser) -> MarkClosed {
    expr_rec(p, &Eof)
}

fn expr_rec(p: &mut Parser, left: &TokenKind) -> MarkClosed {
    let mut lhs = expr_delimited(p);

    while p.at(ParenL) {
        let m = p.open_before(lhs);
        arg_list(p);
        lhs = p.close(m, Call);
    }

    loop {
        let right = p.nth(0);
        if right_binds_tighter(left, &right) {
            let m = p.open_before(lhs);
            p.advance();
            expr_rec(p, &right);
            lhs = p.close(m, Binary);
        } else {
            break lhs;
        }
    }
}

fn arg_list(p: &mut Parser) {
    assert!(p.at(ParenL));
    let m = p.open();

    p.expect(ParenL);
    while !p.at(ParenR) && !p.eof() {
        arg(p);
    }
    p.expect(ParenR);

    p.close(m, Args);
}

fn arg(p: &mut Parser) {
    let m = p.open();

    expression(p);
    if !p.at(ParenR) {
        p.expect(Comma);
    }

    p.close(m, Arg);
}

fn expr_delimited(p: &mut Parser) -> MarkClosed {
    let m = p.open();
    match p.nth(0) {
        LiteralNumber(_) | LiteralBool(_) => {
            p.advance();
            p.close(m, Literal)
        }

        TokenKind::Name(_) => {
            p.advance();
            p.close(m, TreeKind::Name)
        }

        CurlyL => {
            p.expect(CurlyL);
            statement(p);
            p.expect(CurlyR);
            p.close(m, Block)
        }

        ParenL => {
            p.expect(ParenL);
            expression(p);
            p.expect(ParenR);
            p.close(m, Expr)
        }

        _ => {
            if !p.eof() {
                p.advance();
            }
            p.close(m, ErrorTree)
        }
    }
}

fn right_binds_tighter(left: &TokenKind, right: &TokenKind) -> bool {
    fn tightness(kind: &TokenKind) -> Option<usize> {
        [
            // Precedence table
            &[Plus, Minus],
            // &[Start, Slash],
        ]
        .iter()
        .position(|l| l.contains(kind))
    }

    let Some(right_tightness) = tightness(right) else {
        return false;
    };

    let Some(left_tightness) = tightness(left) else {
        assert!(*left == Eof);
        return true;
    };

    right_tightness > left_tightness
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

    fn advance_with_error(&mut self, error: Message) {
        let m = self.open();
        self.errors.push(error);
        self.advance();
        self.close(m, ErrorTree);
    }

    fn eof(&self) -> bool {
        self.pos == self.tokens.len() - 1
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
        assert!(tokens.next().is_some_and(|t| matches!(t.kind, Eof)));

        stack.pop().unwrap()
    }

    fn parse(&mut self) -> Tree {
        file(self);
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
        let mut errs = Vec::new();

        for (source_path, tokens) in input {
            *self = Parser::from(source_path, tokens);
            let cst = self.parse();
            if !self.errors.is_empty() {
                errs.append(&mut self.errors);
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
