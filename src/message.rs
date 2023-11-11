use colored::Colorize;
use std::{collections::HashMap, path::PathBuf};

use crate::language::token::Position;

#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Debug)]
pub enum Severity {
    #[allow(dead_code)]
    Hint,
    #[allow(dead_code)]
    Warning,
    Error,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct Content {
    pub message: String,
    pub indicator_message: Option<String>,
    pub fix_hint: Option<String>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct Message {
    pub severity: Severity,
    pub position: Position,
    pub content: Content,
    pub source_path: PathBuf,
}

impl Message {
    pub fn format(self: &Self, source: &str) -> String {
        let line_num_width = format!("{}", self.position.line).len();
        let inset = line_num_width + 3;

        let severity = match self.severity {
            Severity::Error => "error".bright_red(),
            Severity::Warning => "warning".yellow(),
            Severity::Hint => "hint".bright_blue(),
        };
        let message = format!("{}: {}", severity, self.content.message).bold();

        let path = self.source_path.as_path();
        let path = if path.starts_with("./") {
            path.strip_prefix("./").unwrap()
        } else {
            path
        };

        let position = format!(
            "{} {}:{}:{}",
            " -->".bold().bright_blue(),
            path.to_str().expect("to be able to display path"),
            self.position.line + 1,
            self.position.column,
        );

        let source_line = source
            .lines()
            .nth(self.position.line)
            .expect("source to contain line from position");
        let line_num = format!("{} | ", self.position.line + 1)
            .bright_blue()
            .bold();
        let source_line = format!("{line_num}{source_line}");

        let indicator_message = self
            .content
            .indicator_message
            .to_owned()
            .unwrap_or(String::new());
        let indicator = format!(
            "{}{}{}",
            " ".repeat(inset + self.position.column),
            "^".repeat(self.position.end - self.position.begin),
            indicator_message,
        )
        .bold();

        format!("{message}\n{position}\n{source_line}\n{indicator}\n").to_string()
    }

    pub fn format_errors(sources: &HashMap<PathBuf, String>, errors: &Vec<Self>) -> String {
        errors
            .iter()
            .map(|error| {
                error.format(
                    sources
                        .get(&error.source_path)
                        .expect("source file to be present"),
                )
            })
            .collect::<Vec<String>>()
            .join("\n")
    }
}
