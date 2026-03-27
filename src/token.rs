use std::fmt;

/// The category of a lexical token.
///
/// Most variants are unit variants — their textual content is captured by
/// `Token::lexeme`. Only `Number` carries a parsed payload because a binary
/// `f64` is a fundamentally different representation than the source text.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenKind {
    // Single-character tokens
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals
    Identifier,
    Str,
    Number(f64),

    // Keywords
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LeftParen => write!(f, "LEFT_PAREN"),
            Self::RightParen => write!(f, "RIGHT_PAREN"),
            Self::LeftBrace => write!(f, "LEFT_BRACE"),
            Self::RightBrace => write!(f, "RIGHT_BRACE"),
            Self::Comma => write!(f, "COMMA"),
            Self::Dot => write!(f, "DOT"),
            Self::Minus => write!(f, "MINUS"),
            Self::Plus => write!(f, "PLUS"),
            Self::Semicolon => write!(f, "SEMICOLON"),
            Self::Slash => write!(f, "SLASH"),
            Self::Star => write!(f, "STAR"),
            Self::Bang => write!(f, "BANG"),
            Self::BangEqual => write!(f, "BANG_EQUAL"),
            Self::Equal => write!(f, "EQUAL"),
            Self::EqualEqual => write!(f, "EQUAL_EQUAL"),
            Self::Greater => write!(f, "GREATER"),
            Self::GreaterEqual => write!(f, "GREATER_EQUAL"),
            Self::Less => write!(f, "LESS"),
            Self::LessEqual => write!(f, "LESS_EQUAL"),
            Self::Identifier => write!(f, "IDENTIFIER"),
            Self::Str => write!(f, "STRING"),
            Self::Number(_) => write!(f, "NUMBER"),
            Self::And => write!(f, "AND"),
            Self::Class => write!(f, "CLASS"),
            Self::Else => write!(f, "ELSE"),
            Self::False => write!(f, "FALSE"),
            Self::Fun => write!(f, "FUN"),
            Self::For => write!(f, "FOR"),
            Self::If => write!(f, "IF"),
            Self::Nil => write!(f, "NIL"),
            Self::Or => write!(f, "OR"),
            Self::Print => write!(f, "PRINT"),
            Self::Return => write!(f, "RETURN"),
            Self::Super => write!(f, "SUPER"),
            Self::This => write!(f, "THIS"),
            Self::True => write!(f, "TRUE"),
            Self::Var => write!(f, "VAR"),
            Self::While => write!(f, "WHILE"),
            Self::Eof => write!(f, "EOF"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token<'src> {
    pub kind: TokenKind,
    pub lexeme: &'src str,
    pub line: usize,
    pub column: usize,
    pub length: usize,
}

impl fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            TokenKind::Number(v) => {
                let s = v.to_string();
                if s.contains('.') {
                    write!(f, "{} {} {s}", self.kind, self.lexeme)
                } else {
                    write!(f, "{} {} {s}.0", self.kind, self.lexeme)
                }
            }
            TokenKind::Str => {
                let value = &self.lexeme[1..self.lexeme.len() - 1];
                write!(f, "{} {} {value}", self.kind, self.lexeme)
            }
            _ => write!(f, "{} {} null", self.kind, self.lexeme),
        }
    }
}
