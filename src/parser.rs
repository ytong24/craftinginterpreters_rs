// Stratified grammar — each rule calls the next-higher-precedence rule,
// encoding operator precedence directly in the recursive descent structure.
// The AST types (Expr, BinaryOp, etc.) are precedence-agnostic — precedence
// is enforced here, not in the tree.
//
// expression     → equality ;
// equality       → comparison ( ( "!=" | "==" ) comparison )* ;
// comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
// term           → factor ( ( "-" | "+" ) factor )* ;
// factor         → unary ( ( "/" | "*" ) unary )* ;
// unary          → ( "!" | "-" ) unary | primary ;
// primary        → NUMBER | STRING | "true" | "false" | "nil"
//                | "(" expression ")" ;

use crate::ast::{BinaryOp, Expr, LiteralValue, UnaryOp};
use crate::error::{CompileError, CompileErrorKind};
use crate::token::{Token, TokenKind};

pub struct Parser<'src> {
    tokens: &'src [Token<'src>],
    current: usize,
}

impl<'src> Parser<'src> {
    // --- Helpers ---

    fn peek(&self) -> &Token<'src> {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token<'src> {
        &self.tokens[self.current - 1]
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek().kind, TokenKind::Eof)
    }

    fn advance(&mut self) -> &Token<'src> {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn consume(
        &mut self,
        kind: TokenKind,
        message: &'static str,
    ) -> Result<&Token<'src>, CompileError> {
        if std::mem::discriminant(&self.peek().kind) == std::mem::discriminant(&kind) {
            Ok(self.advance())
        } else {
            Err(self.error_at_current(message))
        }
    }

    fn error_at_current(&self, message: &'static str) -> CompileError {
        let token = self.peek();
        let at = if matches!(token.kind, TokenKind::Eof) {
            "end".to_string()
        } else {
            format!("'{}'", token.lexeme)
        };
        CompileError {
            line: token.line,
            kind: CompileErrorKind::ParseError { at, message },
        }
    }

    // --- Grammar rules (ascending precedence) ---

    fn primary(&mut self) -> Result<Expr<'src>, CompileError> {
        match self.peek().kind {
            TokenKind::Number(value) => {
                self.advance();
                Ok(Expr::Literal(LiteralValue::Number(value)))
            }
            TokenKind::Str => {
                self.advance();
                let lexeme = self.previous().lexeme;
                // Strip surrounding quotes — lexeme is `"hello"`, we want `hello`.
                // This is a sub-slice of the original source, so zero allocations.
                let value = &lexeme[1..lexeme.len() - 1];
                Ok(Expr::Literal(LiteralValue::Str(value)))
            }
            TokenKind::True => {
                self.advance();
                Ok(Expr::Literal(LiteralValue::True))
            }
            TokenKind::False => {
                self.advance();
                Ok(Expr::Literal(LiteralValue::False))
            }
            TokenKind::Nil => {
                self.advance();
                Ok(Expr::Literal(LiteralValue::Nil))
            }
            TokenKind::LeftParen => {
                self.advance();
                let expr = self.expression()?;
                self.consume(TokenKind::RightParen, "Expect ')' after expression.")?;
                Ok(Expr::Grouping(Box::new(expr)))
            }
            _ => Err(self.error_at_current("Expect expression.")),
        }
    }

    fn unary(&mut self) -> Result<Expr<'src>, CompileError> {
        if matches!(self.peek().kind, TokenKind::Bang | TokenKind::Minus) {
            self.advance();
            let op_token = self.previous();
            let operator = match op_token.kind {
                TokenKind::Bang => UnaryOp::Not,
                TokenKind::Minus => UnaryOp::Negate,
                _ => unreachable!(),
            };
            let operator_line = op_token.line;
            let operand = self.unary()?;
            return Ok(Expr::Unary {
                operator,
                operator_line,
                operand: Box::new(operand),
            });
        }

        self.primary()
    }

    fn factor(&mut self) -> Result<Expr<'src>, CompileError> {
        let mut expr = self.unary()?;

        while matches!(self.peek().kind, TokenKind::Slash | TokenKind::Star) {
            self.advance();
            let op_token = self.previous();
            let operator = match op_token.kind {
                TokenKind::Slash => BinaryOp::Divide,
                TokenKind::Star => BinaryOp::Multiply,
                _ => unreachable!(),
            };
            let operator_line = op_token.line;
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                operator_line,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr<'src>, CompileError> {
        let mut expr = self.factor()?;

        while matches!(self.peek().kind, TokenKind::Minus | TokenKind::Plus) {
            self.advance();
            let op_token = self.previous();
            let operator = match op_token.kind {
                TokenKind::Minus => BinaryOp::Subtract,
                TokenKind::Plus => BinaryOp::Add,
                _ => unreachable!(),
            };
            let operator_line = op_token.line;
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                operator_line,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr<'src>, CompileError> {
        let mut expr = self.term()?;

        while matches!(
            self.peek().kind,
            TokenKind::Greater | TokenKind::GreaterEqual | TokenKind::Less | TokenKind::LessEqual
        ) {
            self.advance();
            let op_token = self.previous();
            let operator = match op_token.kind {
                TokenKind::Greater => BinaryOp::Greater,
                TokenKind::GreaterEqual => BinaryOp::GreaterEqual,
                TokenKind::Less => BinaryOp::Less,
                TokenKind::LessEqual => BinaryOp::LessEqual,
                _ => unreachable!(),
            };
            let operator_line = op_token.line;
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                operator_line,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr<'src>, CompileError> {
        let mut expr = self.comparison()?;

        while matches!(
            self.peek().kind,
            TokenKind::BangEqual | TokenKind::EqualEqual
        ) {
            self.advance();
            let op_token = self.previous();
            let operator = match op_token.kind {
                TokenKind::BangEqual => BinaryOp::NotEqual,
                TokenKind::EqualEqual => BinaryOp::Equal,
                _ => unreachable!(),
            };
            let operator_line = op_token.line;
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                operator_line,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn expression(&mut self) -> Result<Expr<'src>, CompileError> {
        self.equality()
    }

    // --- Public API ---

    pub fn new(tokens: &'src [Token<'src>]) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(mut self) -> Result<Expr<'src>, Vec<CompileError>> {
        self.expression().map_err(|e| vec![e])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::Scanner;

    fn parse_source(source: &str) -> Result<String, Vec<CompileError>> {
        let scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens().unwrap();
        let parser = Parser::new(&tokens);
        parser.parse().map(|expr| expr.to_string())
    }

    #[test]
    fn literal_number() {
        assert_eq!(parse_source("42").unwrap(), "42.0");
    }

    #[test]
    fn literal_string() {
        assert_eq!(parse_source("\"hello\"").unwrap(), "hello");
    }

    #[test]
    fn literal_keywords() {
        assert_eq!(parse_source("true").unwrap(), "true");
        assert_eq!(parse_source("false").unwrap(), "false");
        assert_eq!(parse_source("nil").unwrap(), "nil");
    }

    #[test]
    fn unary_negate() {
        assert_eq!(parse_source("-1").unwrap(), "(- 1.0)");
    }

    #[test]
    fn unary_not() {
        assert_eq!(parse_source("!true").unwrap(), "(! true)");
    }

    #[test]
    fn double_unary() {
        assert_eq!(parse_source("!!true").unwrap(), "(! (! true))");
    }

    #[test]
    fn binary_precedence_mul_over_add() {
        assert_eq!(parse_source("1 + 2 * 3").unwrap(), "(+ 1.0 (* 2.0 3.0))");
    }

    #[test]
    fn binary_precedence_add_over_comparison() {
        assert_eq!(
            parse_source("1 + 2 > 3").unwrap(),
            "(> (+ 1.0 2.0) 3.0)"
        );
    }

    #[test]
    fn left_associativity() {
        assert_eq!(
            parse_source("1 - 2 - 3").unwrap(),
            "(- (- 1.0 2.0) 3.0)"
        );
    }

    #[test]
    fn grouping_overrides_precedence() {
        assert_eq!(
            parse_source("(1 + 2) * 3").unwrap(),
            "(* (group (+ 1.0 2.0)) 3.0)"
        );
    }

    #[test]
    fn nested_grouping() {
        assert_eq!(
            parse_source("(5 - (3 - 1)) + -1").unwrap(),
            "(+ (group (- 5.0 (group (- 3.0 1.0)))) (- 1.0))"
        );
    }

    #[test]
    fn error_empty_input() {
        let err = parse_source("").unwrap_err();
        assert_eq!(err.len(), 1);
        assert_eq!(err[0].to_string(), "[line 1] Error at end: Expect expression.");
    }

    #[test]
    fn error_missing_closing_paren() {
        let err = parse_source("(1 + 2").unwrap_err();
        assert_eq!(err.len(), 1);
        assert!(err[0].to_string().contains("Expect ')' after expression."));
    }
}
