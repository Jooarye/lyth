use super::Location;

#[derive(Debug, Hash, Copy, Clone)]
pub enum TokenKind {
  // Literals
  Identifier,
  String,
  Integer,
  Boolean,

  // Arithmetic
  Plus,
  Minus,
  Asterisk,
  Slash,
  Percent,
  And,
  Pipe,
  Caret,
  Bang,
  Tilde,
  Assign,

  // Comparision
  Equal,
  UnEqual,
  LessThan,
  GreaterThan,
  LessEqual,
  GreaterEqual,

  // Punctuation
  Dot,
  Comma,
  Colon,
  SemiColon,
  OpenParen,
  ClosedParen,
  OpenBrace,
  ClosedBrace,

  // Keywords
  Struct,
  Function,
  Let,
  For,
  Loop,
  Break,
  Continue,
  Return,

  // Extension Keywords
  Inline,
}

pub struct Token<'a> {
  pub kind: TokenKind,
  pub text: &'a str,
  pub loc: Location<'a>,
}

impl<'a> Token<'a> {
  pub fn new(kind: TokenKind, text: &'a str, loc: Location<'a>) -> Self {
    Self { kind, text, loc }
  }
}
