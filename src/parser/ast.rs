use crate::lexer::token::TokenKind;
use std::fmt::{Display, Formatter, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum Decl {
  Function {
    name: String,
    params: Vec<(String, Type)>,
    body: Stmt,
    rtyp: Option<Type>,
  },
  Struct {
    name: Type,
    members: Vec<(String, Type)>,
  },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Type {
  pub name: String,
  pub generics: Vec<Type>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
  Let {
    name: String,
    typ: Option<Type>,
    value: Box<Expr>,
  },
  Assign {
    name: String,
    op: Option<TokenKind>,
    value: Box<Expr>,
  },
  If {
    expr: Box<Expr>,
    body: Box<Stmt>,
    elze: Option<Box<Stmt>>,
  },
  Block {
    body: Vec<Stmt>,
  },
  Expr {
    value: Box<Expr>,
  },
  Return {
    value: Option<Box<Expr>>,
  },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
  Literal(Lit),
  Ident(String),
  Call {
    name: String,
    args: Vec<Box<Expr>>,
  },
  Prefix {
    op: TokenKind,
    expr: Box<Expr>,
  },
  Infix {
    op: TokenKind,
    left: Box<Expr>,
    right: Box<Expr>,
  },
  Postfix {
    op: TokenKind,
    expr: Box<Expr>,
  },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Lit {
  Integer(usize),
  String(String),
  Boolean(bool),
}

impl Display for Expr {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result {
    match self {
      Expr::Literal(lit) => write!(f, "{}", lit),
      Expr::Ident(name) => write!(f, "{}", name),
      Expr::Call { name, args } => {
        write!(f, "{}(", name)?;
        for arg in args {
          write!(f, "{},", arg)?;
        }
        write!(f, ")")
      }
      Expr::Prefix { op, expr } => write!(f, "({:?} {})", op, expr),
      Expr::Infix { op, left, right } => write!(f, "({} {:?} {})", left, op, right),
      Expr::Postfix { op, expr } => write!(f, "({} {:?})", expr, op),
    }
  }
}

impl Display for Lit {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result {
    match self {
      Lit::Integer(i) => write!(f, "{}", i),
      Lit::Boolean(b) => write!(f, "{}", b),
      Lit::String(s) => write!(f, r#""{}""#, s),
    }
  }
}
