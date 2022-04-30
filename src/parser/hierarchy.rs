use super::{ast, Parser};
use crate::lexer::token::{Token, TokenKind};

impl<'a> Parser<'a> {
  pub fn parse(&mut self) -> Vec<ast::Decl> {
    let mut decls = Vec::new();

    while !self.at(TokenKind::Eof) {
      let decl = self.decl();
      decls.push(decl);
    }

    decls
  }

  pub fn decl(&mut self) -> ast::Decl {
    match self.peek() {
      TokenKind::Function => {
        self.consume(TokenKind::Function);
        let mut params = Vec::new();

        let ident = self.next().unwrap_or_else(|| {
          panic!(
            "{}: tried to parse the function name but found eof",
            self.last
          )
        });

        assert_eq!(
          ident.kind,
          TokenKind::Identifier,
          "{}: expected {:?} but found {:?}",
          ident.loc,
          TokenKind::Identifier,
          ident.kind
        );

        let name = self.text(ident).to_string();

        self.consume(TokenKind::OpenParen);
        while !self.at(TokenKind::ClosedParen) {
          let param_ident = self
            .next()
            .unwrap_or_else(|| panic!("{}: tried to parse the parameter but found eof", self.last));

          assert_eq!(
            param_ident.kind,
            TokenKind::Identifier,
            "{}: expected {:?} but found {:?}",
            param_ident.loc,
            TokenKind::Identifier,
            param_ident.kind
          );

          let param_name = self.text(param_ident).to_string();
          self.consume(TokenKind::Colon);

          let param_type = self.type_();
          params.push((param_name, param_type));

          if self.at(TokenKind::Comma) {
            self.consume(TokenKind::Comma);
          }
        }
        self.consume(TokenKind::ClosedParen);

        let mut rtyp = None;
        if self.peek() != TokenKind::OpenBrace {
          rtyp = Some(self.type_());
        }

        assert!(
          self.at(TokenKind::OpenBrace),
          "{}: expected a block after function header",
          self.last,
        );

        let body = self.statement();

        ast::Decl::Function {
          name,
          params,
          body,
          rtyp,
        }
      }
      TokenKind::Struct => {
        self.consume(TokenKind::Struct);

        let mut members = Vec::new();
        let name = self.type_();

        self.consume(TokenKind::OpenBrace);
        while !self.at(TokenKind::ClosedBrace) {
          let member_ident = self.next().unwrap_or_else(|| {
            panic!(
              "{}: tried to parse the struct member but found eof",
              self.last
            )
          });

          assert_eq!(
            member_ident.kind,
            TokenKind::Identifier,
            "{} expected {:?} but found {:?}",
            member_ident.loc,
            TokenKind::Identifier,
            member_ident.kind
          );

          let member_name = self.text(member_ident).to_string();
          self.consume(TokenKind::Colon);

          let member_type = self.type_();
          members.push((member_name, member_type));

          if self.at(TokenKind::Comma) {
            self.consume(TokenKind::Comma);
          }
        }

        self.consume(TokenKind::ClosedBrace);
        ast::Decl::Struct { name, members }
      }
      kind => panic!("{}: unknown start of declaration", self.last),
    }
  }

  pub fn type_(&mut self) -> ast::Type {
    let ident = self
      .next()
      .unwrap_or_else(|| panic!("{}: tried to parse a type but found eof", self.last));

    assert_eq!(
      ident.kind,
      TokenKind::Identifier,
      "{}: expected an {:?} at the start of a type but found {:?}",
      ident.loc,
      TokenKind::Identifier,
      ident.kind
    );

    let name = self.text(ident).to_string();

    let mut generics = Vec::new();

    if self.at(TokenKind::LessThan) {
      self.consume(TokenKind::LessThan);

      while !self.at(TokenKind::GreaterThan) {
        let generic = self.type_();
        generics.push(generic);

        if self.at(TokenKind::Comma) {
          self.consume(TokenKind::Comma);
        }
      }
      self.consume(TokenKind::GreaterThan);
    }

    ast::Type { name, generics }
  }

  pub fn statement(&mut self) -> ast::Stmt {
    match self.peek() {
      TokenKind::Let => {
        self.consume(TokenKind::Let);
        let ident = self
          .next()
          .unwrap_or_else(|| panic!("{}: expected an identifier after `let`", self.last));

        assert_eq!(
          ident.kind,
          TokenKind::Identifier,
          "{}: expected identifier after `let`, but found `{:?}`",
          ident.loc,
          ident.kind
        );
        let name = self.text(ident).to_string();
        let mut typ = None;

        if self.at(TokenKind::Colon) {
          self.consume(TokenKind::Colon);
          typ = Some(self.type_());
        }

        self.consume(TokenKind::Assign);
        let value = self.expression();
        self.consume(TokenKind::SemiColon);

        ast::Stmt::Let {
          name,
          value: Box::new(value),
          typ,
        }
      }
      TokenKind::Identifier => {
        let ident = self.next().unwrap();
        let name = self.text(ident).to_string();

        if self.at(TokenKind::Assign) {
          self.consume(TokenKind::Assign);
          let value = self.expression();
          self.consume(TokenKind::SemiColon);
          ast::Stmt::Assign {
            name,
            op: None,
            value: Box::new(value),
          }
        } else {
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
          self.consume(TokenKind::SemiColon);

          ast::Stmt::Expr {
            value: Box::new(ast::Expr::Call { name, args }),
          }
        }
      }
      TokenKind::Return => {
        self.consume(TokenKind::Return);
        if self.peek() == TokenKind::SemiColon {
          self.consume(TokenKind::SemiColon);
          ast::Stmt::Return { value: None }
        } else {
          let expr = self.expression();
          self.consume(TokenKind::SemiColon);
          ast::Stmt::Return {
            value: Some(Box::new(expr)),
          }
        }
      }
      TokenKind::If => {
        self.consume(TokenKind::If);
        let condition = self.expression();

        assert!(
          self.at(TokenKind::OpenBrace),
          "Expected a block after `if` statement"
        );
        let body = Box::new(self.statement());

        let elze = if self.at(TokenKind::Else) {
          self.consume(TokenKind::Else);
          assert!(
            self.at(TokenKind::If) || self.at(TokenKind::OpenBrace),
            "Expected a block or an `if` after `else` statement"
          );
          Some(Box::new(self.statement()))
        } else {
          None
        };

        ast::Stmt::If {
          expr: Box::new(condition),
          body,
          elze,
        }
      }
      TokenKind::OpenBrace => {
        self.consume(TokenKind::OpenBrace);

        let mut body = Vec::new();

        while !self.at(TokenKind::ClosedBrace) {
          let stmt = self.statement();
          body.push(stmt);
        }

        self.consume(TokenKind::ClosedBrace);
        ast::Stmt::Block { body }
      }
      _ => {
        // Expression statement
        let expr = self.expression();
        self.consume(TokenKind::SemiColon);

        ast::Stmt::Expr {
          value: Box::new(expr),
        }
      }
    }
  }
}
