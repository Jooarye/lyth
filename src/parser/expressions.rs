use super::{ast, Parser};
use crate::lexer::token::{Token, TokenKind};

impl<'a> Parser<'a> {
  #[inline]
  pub fn expression(&mut self) -> ast::Expr {
    self.parse_expression(0)
  }

  pub fn parse_expression(&mut self, binding_power: u8) -> ast::Expr {
    let mut lhs = match self.peek() {
      lit @ TokenKind::Integer | lit @ TokenKind::Boolean | lit @ TokenKind::String => {
        let literal_text = {
          let literal_token = self.next().unwrap();
          self.text(literal_token)
        };

        let lit = match lit {
          TokenKind::Integer => ast::Lit::Integer(
            literal_text
              .parse()
              .expect(&format!("invalid integer literal: `{}`", literal_text)),
          ),
          TokenKind::Boolean => ast::Lit::Boolean(
            literal_text
              .parse()
              .expect(&format!("invalid bool literal: `{}`", literal_text)),
          ),
          TokenKind::String => {
            ast::Lit::String(literal_text[1..(literal_text.len() - 1)].to_string())
          }
          _ => unreachable!(),
        };

        ast::Expr::Literal(lit)
      }
      TokenKind::Identifier => {
        let name = {
          let ident_token = self.next().unwrap();
          self.text(ident_token).to_string()
        };

        if !self.at(TokenKind::OpenParen) {
          // plain identifier
          ast::Expr::Ident(name)
        } else {
          //  function call
          let mut args = Vec::new();
          self.consume(TokenKind::OpenParen);
          while !self.at(TokenKind::ClosedParen) {
            let arg = self.parse_expression(0);
            args.push(Box::new(arg));
            if self.at(TokenKind::Comma) {
              self.consume(TokenKind::Comma);
            }
          }
          self.consume(TokenKind::ClosedParen);
          ast::Expr::Call { name, args }
        }
      }
      TokenKind::OpenParen => {
        // There is no AST node for grouped expressions.
        // Parentheses just influence the tree structure.
        self.consume(TokenKind::OpenParen);
        let expr = self.parse_expression(0);
        self.consume(TokenKind::ClosedParen);
        expr
      }
      op @ TokenKind::Minus | op @ TokenKind::Bang => {
        self.consume(op);
        let ((), right_binding_power) = op.prefix_binding_power();
        let expr = self.parse_expression(right_binding_power);
        ast::Expr::Prefix {
          op,
          expr: Box::new(expr),
        }
      }
      kind => {
        panic!("Unknown start of expression: `{:?}`", kind);
      }
    };
    loop {
      let op = match self.peek() {
        op @ TokenKind::Plus
        | op @ TokenKind::Minus
        | op @ TokenKind::Asterisk
        | op @ TokenKind::Slash
        | op @ TokenKind::Caret
        | op @ TokenKind::Equal
        | op @ TokenKind::UnEqual
        | op @ TokenKind::And
        | op @ TokenKind::Pipe
        | op @ TokenKind::LessThan
        | op @ TokenKind::LessEqual
        | op @ TokenKind::GreaterThan
        | op @ TokenKind::GreaterEqual
        | op @ TokenKind::Bang => op,
        TokenKind::Eof => break,
        TokenKind::ClosedParen
        | TokenKind::ClosedBrace
        | TokenKind::OpenBrace
        | TokenKind::Comma
        | TokenKind::SemiColon => break,
        kind => panic!("Unknown operator: `{:?}`", kind),
      };

      if let Some((left_binding_power, ())) = op.postfix_binding_power() {
        if left_binding_power < binding_power {
          // previous operator has higher binding power than new one --> end of expression
          break;
        }

        self.consume(op);
        // no recursive call here, because we have already parsed our operand `lhs`
        lhs = ast::Expr::Postfix {
          op,
          expr: Box::new(lhs),
        };
        // parsed an operator --> go round the loop again
        continue;
      }

      if let Some((left_binding_power, right_binding_power)) = op.infix_binding_power() {
        if left_binding_power < binding_power {
          // previous operator has higher binding power than new one --> end of expression
          break;
        }

        self.consume(op);
        let rhs = self.parse_expression(right_binding_power);
        lhs = ast::Expr::Infix {
          op,
          left: Box::new(lhs),
          right: Box::new(rhs),
        };
        // parsed an operator --> go round the loop again
        continue;
      }

      break; // Not an operator --> end of expression
    }

    lhs
  }
}

trait Operator {
  /// Prefix operators bind their operand to the right.
  fn prefix_binding_power(&self) -> ((), u8);

  /// Infix operators bind two operands, lhs and rhs.
  fn infix_binding_power(&self) -> Option<(u8, u8)>;

  /// Postfix operators bind their operand to the left.
  fn postfix_binding_power(&self) -> Option<(u8, ())>;
}

impl Operator for TokenKind {
  fn prefix_binding_power(&self) -> ((), u8) {
    match self {
      TokenKind::Plus | TokenKind::Minus | TokenKind::Bang => ((), 51),
      // Prefixes are the only operators we have already seen
      // when we call this, so we know the token must be
      // one of the above
      _ => unreachable!("Not a prefix operator: {:?}", self),
    }
  }

  fn infix_binding_power(&self) -> Option<(u8, u8)> {
    let result = match self {
      TokenKind::Pipe => (1, 2),
      TokenKind::And => (3, 4),
      TokenKind::Equal | TokenKind::UnEqual => (5, 6),
      TokenKind::LessThan
      | TokenKind::GreaterThan
      | TokenKind::LessEqual
      | TokenKind::GreaterEqual => (7, 8),
      TokenKind::Caret => (9, 10),
      TokenKind::Plus | TokenKind::Minus => (11, 12),
      TokenKind::Asterisk | TokenKind::Slash | TokenKind::Percent => (13, 14),
      _ => return None,
    };
    Some(result)
  }

  fn postfix_binding_power(&self) -> Option<(u8, ())> {
    let result = match self {
      TokenKind::Bang => (101, ()),
      _ => return None,
    };
    Some(result)
  }
}
