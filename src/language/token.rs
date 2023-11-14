use std::{fmt::Display, path::PathBuf};

use crate::{
    message::{Content, Message, Severity},
    phase::lexer::LexerError,
};

#[derive(PartialEq, PartialOrd, Clone)]
pub struct Token {
    /// The path to the source from which this position stems
    pub source_path: PathBuf,
    pub kind: TokenKind,
    pub position: Position,
}

impl Token {
    pub fn text(&self) -> String {
        format!("{}", self.kind)
    }
}

#[derive(PartialEq, PartialOrd, Clone, Debug)]
pub enum TokenKind {
    // Values
    LiteralNumber(f64),
    LiteralBool(bool),
    Name(String),

    // Keywords
    KeywordIf,
    KeywordElse,
    KeywordLet,
    KeywordThen,
    KeywordDef,

    // Operators
    Plus,
    Minus,

    // Misc
    ParenL,
    ParenR,
    CurlyL,
    CurlyR,
    Arrow,
    Colon,
    Equal,
    Comma,
    Semi,

    // Special
    Eof,
    Error(LexerError),
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TokenKind::LiteralNumber(i) => i.to_string(),
                TokenKind::LiteralBool(i) => i.to_string(),
                TokenKind::Name(i) => i.to_string(),
                TokenKind::KeywordIf => "if".to_string(),
                TokenKind::KeywordElse => "else".to_string(),
                TokenKind::KeywordLet => "let".to_string(),
                TokenKind::Plus => "+".to_string(),
                TokenKind::Minus => "-".to_string(),
                TokenKind::ParenL => "(".to_string(),
                TokenKind::ParenR => ")".to_string(),
                TokenKind::Colon => ":".to_string(),
                TokenKind::Arrow => "->".to_string(),
                TokenKind::Equal => "=".to_string(),
                TokenKind::Eof => "<eof>".to_string(),
                TokenKind::Error(_) => "<error>".to_string(),
                TokenKind::KeywordDef => "def".to_string(),
                TokenKind::CurlyL => "{".to_string(),
                TokenKind::CurlyR => "}".to_string(),
                TokenKind::Comma => ",".to_string(),
                TokenKind::Semi => ";".to_string(),
                TokenKind::KeywordThen => "then".to_string(),
            }
        )
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Debug)]
pub struct Position {
    /// The line at where the position starts. A position can only span one line
    pub line: usize,
    /// The column where the position begins.
    pub column: usize,
    /// The absolute offset into source of where this position begins
    pub begin: usize,
    /// The absolute offset into source of where this position ends
    pub end: usize,
}

impl TryFrom<&Token> for Message {
    type Error = ();

    fn try_from(token: &Token) -> Result<Self, Self::Error> {
        if let TokenKind::Error(err) = token.kind {
            let content = match err {
                LexerError::UnexpectedToken(c) => Content {
                    message: format!("Unexpected character '{}'", c),
                    indicator_message: Some("found here".to_string()),
                    fix_hint: None,
                },
            };

            Ok(Message {
                source_path: token.source_path.clone(),
                position: token.position,
                severity: Severity::Error,
                content,
            })
        } else {
            Err(())
        }
    }
}

impl std::fmt::Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Token({:?}, {}, {})",
            self.kind, self.position.line, self.position.column
        )
    }
}
