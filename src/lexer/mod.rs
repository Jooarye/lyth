mod rules;
pub mod token;

use rules::{get_rules, Rule};
use std::fmt::{Display, Formatter, Result};
use token::*;

#[derive(Clone)]
pub struct Location<'a> {
    pub file: &'a str,
    pub line: usize,
    pub col: usize,
}

impl<'a> Display for Location<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}:{}:{}", self.file, self.line, self.col)
    }
}

pub struct Lexer<'a> {
    input: &'a str,
    loc: Location<'a>,
    rules: Vec<Rule>,
}

impl<'a> Lexer<'a> {
    pub fn new(file: &'a str, input: &'a str) -> Self {
        Self {
            input,
            rules: get_rules(),
            loc: Location {
                file,
                line: 1,
                col: 1,
            },
        }
    }

    pub fn next_token(&mut self) -> Option<Token<'a>> {
        loop {
            let next = self.input.chars().next()?;

            if !next.is_whitespace() {
                break;
            }

            if next == '\n' {
                self.loc.line += 1;
                self.loc.col = 1;
            } else {
                self.loc.col += 1;
            }

            self.input = &self.input[1..];
        }

        let (len, kind) = self
            .rules
            .iter()
            .rev()
            .filter_map(|rule| Some(((rule.matches)(self.input)?, rule.kind)))
            .max_by_key(|&(len, _)| len)?;

        let tok = Token::new(kind, &self.input[0..len], self.loc.clone());

        self.loc.col += len;
        self.input = &self.input[len..];

        Some(tok)
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}
