use std::fmt;

/// Top-level error type for the Lox interpreter.
/// Compile errors collect multiple issues; runtime errors halt immediately.
#[derive(Debug)]
pub enum LoxError {
    Compile(Vec<CompileError>),
    Runtime(RuntimeError),
}

impl LoxError {
    pub fn exit_code(&self) -> i32 {
        match self {
            LoxError::Compile(_) => 65,
            LoxError::Runtime(_) => 70,
        }
    }
}

impl fmt::Display for LoxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoxError::Compile(errors) => {
                for (i, error) in errors.iter().enumerate() {
                    if i > 0 {
                        writeln!(f)?;
                    }
                    write!(f, "{error}")?;
                }
                Ok(())
            }
            LoxError::Runtime(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for LoxError {}

// ---------------------------------------------------------------------------
// Compile errors (scanning + parsing)
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct CompileError {
    pub line: usize,
    pub kind: CompileErrorKind,
}

/// Extend this enum with new variants as the interpreter grows.
/// Scanning variants produce `Error: {message}`.
/// Parsing variants (added later) produce `Error at '{token}': {message}`.
#[derive(Debug)]
pub enum CompileErrorKind {
    // Scanning errors
    UnexpectedCharacter(char),
    UnterminatedString,
    // Parsing errors
    ParseError { at: String, message: &'static str },
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[line {}] {}", self.line, self.kind)
    }
}

impl fmt::Display for CompileErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompileErrorKind::UnexpectedCharacter(c) => {
                write!(f, "Error: Unexpected character: {c}")
            }
            CompileErrorKind::UnterminatedString => {
                write!(f, "Error: Unterminated string.")
            }
            CompileErrorKind::ParseError { at, message } => {
                write!(f, "Error at {at}: {message}")
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Runtime errors
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct RuntimeError {
    pub message: String,
    pub line: usize,
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.message)?;
        write!(f, "[line {}]", self.line)
    }
}
