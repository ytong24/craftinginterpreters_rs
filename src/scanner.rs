use crate::error::CompileError;
use crate::error::CompileErrorKind;
use crate::token::{Token, TokenKind};

fn is_alpha(s: &str) -> bool {
    s.as_bytes()
        .first()
        .is_some_and(|b| b.is_ascii_alphabetic() || *b == b'_')
}

fn is_alphanumeric(s: &str) -> bool {
    s.as_bytes()
        .first()
        .is_some_and(|b| b.is_ascii_alphanumeric() || *b == b'_')
}

fn keyword_kind(lexeme: &str) -> Option<TokenKind> {
    match lexeme {
        "and" => Some(TokenKind::And),
        "class" => Some(TokenKind::Class),
        "else" => Some(TokenKind::Else),
        "false" => Some(TokenKind::False),
        "for" => Some(TokenKind::For),
        "fun" => Some(TokenKind::Fun),
        "if" => Some(TokenKind::If),
        "nil" => Some(TokenKind::Nil),
        "or" => Some(TokenKind::Or),
        "print" => Some(TokenKind::Print),
        "return" => Some(TokenKind::Return),
        "super" => Some(TokenKind::Super),
        "this" => Some(TokenKind::This),
        "true" => Some(TokenKind::True),
        "var" => Some(TokenKind::Var),
        "while" => Some(TokenKind::While),
        _ => None,
    }
}

fn is_ascii_digit(s: &str) -> bool {
    // Safe to use `s.as_bytes().first()` because peek()/peek_next() always return either "" or a single UTF-8 character.
    // ASCII digits are single-byte (0x30–0x39), so checking the first byte is sufficient.
    // Multi-byte characters have first byte >= 0x80, which is_ascii_digit() rejects.
    s.as_bytes().first().is_some_and(|b| b.is_ascii_digit())
}

pub struct Scanner<'src> {
    source: &'src str,
    tokens: Vec<Token<'src>>,
    errors: Vec<CompileError>,
    start: usize,
    current: usize,
    line: usize,
    column: usize,
}

impl<'src> Scanner<'src> {
    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) {
        let c = self.source[self.current..].chars().next().unwrap();
        self.current += c.len_utf8();
        self.column += 1;
    }

    fn peek(&self) -> &'src str {
        if self.is_at_end() {
            return "";
        }
        let c = self.source[self.current..].chars().next().unwrap();
        &self.source[self.current..self.current + c.len_utf8()]
    }

    /// Two-character lookahead. Unlike `peek()` which returns the character at
    /// `self.current`, this returns the character *after* it — needed to check
    /// what follows a "." without consuming it.
    fn peek_next(&self) -> &'src str {
        let first = self.source[self.current..].chars().next();
        let next_start = self.current + first.map_or(0, |c| c.len_utf8());
        if next_start >= self.source.len() {
            return "";
        }
        let next = self.source[next_start..].chars().next().unwrap();
        &self.source[next_start..next_start + next.len_utf8()]
    }

    fn match_next(&mut self, expected: &str) -> bool {
        if self.peek() != expected {
            return false;
        }
        self.advance();
        true
    }

    fn lexeme(&self) -> &'src str {
        &self.source[self.start..self.current]
    }

    fn add_token(&mut self, kind: TokenKind, lexeme: &'src str) {
        self.tokens.push(Token {
            kind,
            lexeme,
            line: self.line,
            column: self.column - (self.current - self.start),
            length: self.current - self.start,
        });
    }

    fn number(&mut self) {
        while is_ascii_digit(self.peek()) {
            self.advance();
        }

        // Fractional part: consume "." only if followed by a digit.
        if self.peek() == "." && is_ascii_digit(self.peek_next()) {
            self.advance();
            while is_ascii_digit(self.peek()) {
                self.advance();
            }
        }

        let lexeme = self.lexeme();
        let value: f64 = lexeme.parse().unwrap();
        self.add_token(TokenKind::Number(value), lexeme);
    }

    fn string(&mut self) {
        // Save the token's starting position before consuming across lines.
        let start_line = self.line;
        let start_column = self.column - (self.current - self.start);

        while self.peek() != "\"" && !self.is_at_end() {
            if self.peek() == "\n" {
                self.line += 1;
                self.column = 0;
            }
            self.advance();
        }

        if self.is_at_end() {
            self.errors.push(CompileError {
                line: self.line,
                kind: CompileErrorKind::UnterminatedString,
            });
            return;
        }

        // Consume the closing "
        self.advance();

        let lexeme = self.lexeme();

        // Manual push instead of add_token() because add_token() computes the starting
        // column as `self.column - token_len`, which underflows for multi-line strings.
        // E.g. "ab\ncd" — after the closing ", self.column is 3 but the token spans 8 bytes, so 3 - 8 overflows.
        // Only strings can span lines; other tokens are safe.
        self.tokens.push(Token {
            kind: TokenKind::Str,
            lexeme,
            line: start_line,
            column: start_column,
            length: self.current - self.start,
        });
    }

    fn identifier(&mut self) {
        while is_alphanumeric(self.peek()) {
            self.advance();
        }
        let lexeme = self.lexeme();
        let kind = keyword_kind(lexeme).unwrap_or(TokenKind::Identifier);
        self.add_token(kind, lexeme);
    }

    fn scan_token(&mut self) {
        self.advance();
        let lexeme = self.lexeme();
        match lexeme {
            "(" => self.add_token(TokenKind::LeftParen, lexeme),
            ")" => self.add_token(TokenKind::RightParen, lexeme),
            "{" => self.add_token(TokenKind::LeftBrace, lexeme),
            "}" => self.add_token(TokenKind::RightBrace, lexeme),
            "," => self.add_token(TokenKind::Comma, lexeme),
            "." => self.add_token(TokenKind::Dot, lexeme),
            "-" => self.add_token(TokenKind::Minus, lexeme),
            "+" => self.add_token(TokenKind::Plus, lexeme),
            ";" => self.add_token(TokenKind::Semicolon, lexeme),
            "/" => {
                if self.match_next("/") {
                    // Line comment: consume until newline (leave '\n' for the next iteration
                    // so it still increments self.line).
                    while self.peek() != "\n" && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenKind::Slash, lexeme);
                }
            }
            "*" => self.add_token(TokenKind::Star, lexeme),
            "!" => {
                if self.match_next("=") {
                    self.add_token(TokenKind::BangEqual, self.lexeme());
                } else {
                    self.add_token(TokenKind::Bang, lexeme);
                }
            }
            "=" => {
                if self.match_next("=") {
                    self.add_token(TokenKind::EqualEqual, self.lexeme());
                } else {
                    self.add_token(TokenKind::Equal, lexeme);
                }
            }
            ">" => {
                if self.match_next("=") {
                    self.add_token(TokenKind::GreaterEqual, self.lexeme());
                } else {
                    self.add_token(TokenKind::Greater, lexeme);
                }
            }
            "<" => {
                if self.match_next("=") {
                    self.add_token(TokenKind::LessEqual, self.lexeme());
                } else {
                    self.add_token(TokenKind::Less, lexeme);
                }
            }

            "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" => self.number(),

            "\"" => self.string(),

            " " | "\r" | "\t" => {}

            "\n" => {
                self.line += 1;
                self.column = 0;
            }

            other => {
                if is_alpha(other) {
                    self.identifier();
                } else {
                    let c = other.chars().next().unwrap();
                    self.errors.push(CompileError {
                        line: self.line,
                        kind: CompileErrorKind::UnexpectedCharacter(c),
                    });
                }
            }
        }
    }

    pub fn new(source: &'src str) -> Self {
        Self {
            source,
            tokens: Vec::new(),
            errors: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
            column: 0,
        }
    }

    pub fn scan_tokens(mut self) -> Result<Vec<Token<'src>>, Vec<CompileError>> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.tokens.push(Token {
            kind: TokenKind::Eof,
            lexeme: "",
            line: self.line,
            column: self.column,
            length: 0,
        });

        if self.errors.is_empty() {
            Ok(self.tokens)
        } else {
            Err(self.errors)
        }
    }
}
