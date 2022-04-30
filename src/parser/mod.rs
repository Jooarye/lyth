pub mod ast;

mod expressions;
mod hierarchy;

use crate::lexer::token::*;
use crate::lexer::{Lexer, Location};
use std::iter::Peekable;

pub struct Parser<'a> {
    tokens: Peekable<Lexer<'a>>,
    last: Location<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(file: &'a str, input: &'a str) -> Self {
        Self {
            tokens: Lexer::new(file, input).peekable(),
            last: Location {
                file,
                line: 1,
                col: 1,
            },
        }
    }

    #[inline]
    pub fn text(&self, token: Token<'a>) -> &'a str {
        token.text
    }

    #[inline]
    pub(crate) fn peek(&mut self) -> TokenKind {
        self.tokens
            .peek()
            .map(|token| token.kind)
            .unwrap_or(TokenKind::Eof)
    }

    #[inline]
    pub(crate) fn at(&mut self, kind: TokenKind) -> bool {
        self.peek() == kind
    }

    #[inline]
    pub(crate) fn next(&mut self) -> Option<Token<'a>> {
        self.tokens.next()
    }

    pub(crate) fn consume(&mut self, expected: TokenKind) {
        let token = self
            .next()
            .expect(&format!("{}: unexpected end of file", self.last));
        assert_eq!(
            token.kind, expected,
            "{}: expected {:?} but got {:?}",
            token.loc, expected, token.kind
        );
        self.last = token.loc;
        self.last.col += token.text.len();
    }
}
