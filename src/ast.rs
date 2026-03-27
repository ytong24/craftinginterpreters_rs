use std::fmt;

// Lox BNF:
/*
    expression     → literal
                | unary
                | binary
                | grouping ;

    literal        → NUMBER | STRING | "true" | "false" | "nil" ;
    grouping       → "(" expression ")" ;
    unary          → ( "-" | "!" ) expression ;
    binary         → expression operator expression ;
    operator       → "==" | "!=" | "<" | "<=" | ">" | ">="
                | "+"  | "-"  | "*" | "/" ;
*/

// The AST is a *semantic* representation — the parser validates structure at the
// boundary, and downstream consumers (interpreter, etc.) trust the types. Adding
// a new variant to Expr forces exhaustive-match updates everywhere it's consumed,
// so the compiler guides you through every site that needs handling.

/// A Lox expression tree node.
///
/// Recursive children are `Box`ed because Rust enums are sized inline — without
/// indirection, `Expr` would be infinitely large.
#[derive(Debug)]
pub enum Expr<'src> {
    Literal(LiteralValue<'src>),
    Unary {
        operator: UnaryOp,
        // Only the line is stored — the book's runtime error format is `[line N]`,
        // so column/length aren't needed.
        operator_line: usize,
        operand: Box<Expr<'src>>,
    },
    Binary {
        left: Box<Expr<'src>>,
        operator: BinaryOp,
        operator_line: usize,
        right: Box<Expr<'src>>,
    },
    Grouping(Box<Expr<'src>>),
}

// Dedicated operator enums instead of reusing Token — the parser converts
// TokenKind → operator enum during construction. If a token doesn't map to a
// valid operator, the parser rejects it. This makes the AST correct by
// construction: every operator position holds a valid operator, and consumers
// get exhaustive matching with no `_ => unreachable!()` fallback arms.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Negate, // -
    Not,    // !
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Add,
    Subtract,
    Multiply,
    Divide,
}

/// A Lox literal value.
///
/// `True`, `False`, and `Nil` are separate variants rather than `Bool(bool)` +
/// `Nil` — the grammar treats them as distinct terminals, and collapsing them
/// would just add a bool that every match arm immediately unpacks.
///
/// `Str` borrows from the source string — the scanner's lexeme includes quotes
/// (`"hello"`), but `&lexeme[1..len-1]` is still a valid slice into the original
/// source. Zero allocations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LiteralValue<'src> {
    Number(f64),
    Str(&'src str),
    True,
    False,
    Nil,
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Negate => write!(f, "-"),
            Self::Not => write!(f, "!"),
        }
    }
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Equal => write!(f, "=="),
            Self::NotEqual => write!(f, "!="),
            Self::Less => write!(f, "<"),
            Self::LessEqual => write!(f, "<="),
            Self::Greater => write!(f, ">"),
            Self::GreaterEqual => write!(f, ">="),
            Self::Add => write!(f, "+"),
            Self::Subtract => write!(f, "-"),
            Self::Multiply => write!(f, "*"),
            Self::Divide => write!(f, "/"),
        }
    }
}

impl fmt::Display for LiteralValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number(v) => {
                let s = v.to_string();
                if s.contains('.') {
                    write!(f, "{s}")
                } else {
                    write!(f, "{s}.0")
                }
            }
            Self::Str(s) => write!(f, "{s}"),
            Self::True => write!(f, "true"),
            Self::False => write!(f, "false"),
            Self::Nil => write!(f, "nil"),
        }
    }
}

impl fmt::Display for Expr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Literal(v) => write!(f, "{v}"),
            Self::Unary {
                operator, operand, ..
            } => {
                write!(f, "({operator} {operand})")
            }
            Self::Binary {
                left,
                operator,
                right,
                ..
            } => {
                write!(f, "({operator} {left} {right})")
            }
            Self::Grouping(expr) => write!(f, "(group {expr})"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_nested_expression() {
        // -123 * (45.67)
        let ast = Expr::Binary {
            left: Box::new(Expr::Unary {
                operator: UnaryOp::Negate,
                operator_line: 1,
                operand: Box::new(Expr::Literal(LiteralValue::Number(123.0))),
            }),
            operator: BinaryOp::Multiply,
            operator_line: 1,
            right: Box::new(Expr::Grouping(Box::new(Expr::Literal(
                LiteralValue::Number(45.67),
            )))),
        };
        assert_eq!(ast.to_string(), "(* (- 123.0) (group 45.67))");
    }
}
