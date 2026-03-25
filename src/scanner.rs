use crate::error::CompileError;
use crate::error::CompileErrorKind;
use crate::token::{Token, TokenKind};

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

        // Lexeme excludes the surrounding quotes.
        let lexeme = &self.source[self.start + 1..self.current - 1];

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
            "\"" => self.string(),
            " " | "\r" | "\t" => {}
            "\n" => {
                self.line += 1;
                self.column = 0;
            }
            other => {
                let c = other.chars().next().unwrap();
                self.errors.push(CompileError {
                    line: self.line,
                    kind: CompileErrorKind::UnexpectedCharacter(c),
                });
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
