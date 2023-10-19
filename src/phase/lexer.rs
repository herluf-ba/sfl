use std::{collections::HashMap, fmt::Debug, path::PathBuf};

use crate::{
    config::Config,
    language::token::{Position, Token, TokenKind},
    message::Message,
    phase::{Phase, PhaseResult},
};

#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Debug)]
pub enum LexerError {
    UnexpectedToken(char),
}

pub struct Lexer {
    is_ok: bool,
    begin: usize,
    end: usize,
    line: usize,
    column: usize,
    start_column: usize,
    source_path: PathBuf,
    characters: Vec<char>,
}

impl Lexer {
    fn new(source: (&PathBuf, &String)) -> Self {
        Self {
            is_ok: true,
            begin: 0,
            end: 0,
            line: 0,
            column: 0,
            start_column: 0,
            source_path: source.0.clone(),
            characters: source.1.to_owned().chars().collect(),
        }
    }

    fn eof(self: &Self) -> bool {
        self.end >= self.characters.len()
    }

    fn advance(self: &mut Self) -> Option<char> {
        if self.eof() {
            return None;
        }

        let c = self.characters[self.end];
        self.end += 1;
        self.column += 1;
        if c == '\n' {
            self.line += 1;
            self.column = 0;
            self.start_column = 0;
        }
        Some(c)
    }

    fn nth(self: &Self, lookahead: usize) -> char {
        let index = (self.end + lookahead).min(self.characters.len() - 1);
        self.characters[index]
    }

    fn whitespace(self: &mut Self) {
        while !self.eof() && self.nth(0).is_whitespace() {
            self.advance();
        }
    }

    fn keyword(self: &mut Self, keyword: &str) -> bool {
        if self.end - 1 + keyword.len() > self.characters.len() {
            return false;
        }

        let matches = self
            .characters
            .get(self.end - 1..self.end - 1 + keyword.len())
            .expect("characters should contain enough chars")
            .iter()
            .zip(keyword.chars())
            .all(|(&a, b)| a == b);

        if matches {
            for _ in 0..keyword.len() - 1 {
                self.advance();
            }
        }

        matches
    }

    fn lexeme(self: &mut Self) -> Option<String> {
        self.characters
            .get(self.begin..self.end)
            .map(|s| s.iter().collect::<String>())
    }

    fn number(self: &mut Self) -> TokenKind {
        while !self.eof() && self.nth(0).is_digit(10) {
            self.advance();
        }

        let lexeme = self.lexeme().expect("lexeme to be available");
        let num = lexeme.parse::<usize>().expect("lexeme to be a number");
        TokenKind::LiteralInt(num)
    }

    fn name(self: &mut Self) -> TokenKind {
        let mut p = self.nth(0);
        while !self.eof() && (p.is_alphabetic() || p == '_') {
            self.advance();
            p = self.nth(0);
        }

        let lexeme = self.lexeme().expect("lexeme to be available");
        TokenKind::Name(lexeme)
    }

    fn comment(self: &mut Self) -> TokenKind {
        let mut p = self.nth(0);
        while !self.eof() && p != '\n' {
            self.advance();
            p = self.nth(0);
        }

        let lexeme = self.lexeme().expect("lexeme to be available");
        TokenKind::Comment(lexeme)
    }

    fn position(self: &Self) -> Position {
        Position {
            begin: self.begin,
            end: self.end,
            column: self.start_column,
            line: self.line,
        }
    }

    fn next_token(self: &mut Self) -> Option<Token> {
        self.whitespace();
        if self.eof() {
            return None;
        }
        self.begin = self.end;
        self.start_column = self.column;

        let kind = match self.advance() {
            Some('+') => TokenKind::Plus,
            Some(_) if self.keyword("->") => TokenKind::Arrow,
            Some('-') => TokenKind::Minus,
            Some(':') => TokenKind::Colon,
            Some('=') => TokenKind::Equal,
            Some('(') => TokenKind::ParenL,
            Some(')') => TokenKind::ParenR,
            Some('\\') => TokenKind::Backslash,
            Some('#') => self.comment(),
            Some(_) if self.keyword("if") => TokenKind::KeywordIf,
            Some(_) if self.keyword("then") => TokenKind::KeywordThen,
            Some(_) if self.keyword("else") => TokenKind::KeywordElse,
            Some(_) if self.keyword("let") => TokenKind::KeywordLet,
            Some(_) if self.keyword("in") => TokenKind::KeywordIn,
            Some(_) if self.keyword("true") => TokenKind::LiteralBool(true),
            Some(_) if self.keyword("false") => TokenKind::LiteralBool(false),
            Some(x) if x.is_digit(10) => self.number(),
            Some(x) if x.is_alphabetic() => self.name(),
            Some(x) => {
                self.is_ok = false;
                TokenKind::Error(LexerError::UnexpectedToken(x))
            }
            None => panic!("Lexer bottomed out!"),
        };

        Some(Token {
            kind,
            source_path: self.source_path.clone(),
            position: self.position(),
        })
    }

    pub fn lex(self: &mut Self) -> Vec<Token> {
        let mut tokens = Vec::new();
        while let Some(token) = self.next_token() {
            tokens.push(token);
        }

        tokens.push(Token {
            kind: TokenKind::Eof,
            source_path: self.source_path.clone(),
            position: self.position(),
        });

        tokens
    }
}

pub type Input = HashMap<PathBuf, String>;
pub type Output = HashMap<PathBuf, Vec<Token>>;

impl Phase<Input, Output> for Lexer {
    fn new() -> Self {
        Lexer::new((&PathBuf::new(), &String::new()))
    }

    fn run(self: &mut Self, _config: &Config, input: &Input) -> PhaseResult<Output> {
        let mut out = HashMap::new();
        let mut errs = HashMap::new();

        for source in input {
            *self = Lexer::new(source);
            let tokens = self.lex();

            if !self.is_ok {
                let errors: Vec<Message> = tokens
                    .iter()
                    .filter(|t| matches!(t.kind, TokenKind::Error(_)))
                    .map(|t| t.try_into().expect("token to be an erorr"))
                    .collect();
                errs.insert(source.0.clone(), errors);
            }

            out.insert(source.0.clone(), tokens);
        }

        if !errs.is_empty() {
            PhaseResult::SoftErr(out, errs)
        } else {
            PhaseResult::Ok(out)
        }
    }
}
