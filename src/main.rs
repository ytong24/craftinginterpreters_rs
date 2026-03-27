mod ast;
mod error;
mod parser;
mod scanner;
mod token;

use crate::error::LoxError;
use crate::parser::Parser;
use crate::scanner::Scanner;
use clap::{Parser as ClapParser, ValueEnum};
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum Mode {
    Tokenize,
    Parse,
    // Future: Evaluate, Run, etc.
}

#[derive(ClapParser)]
#[command(name = "loxrs", about = "A Lox language interpreter")]
struct Cli {
    /// Pipeline stage to stop at (tokenize, parse)
    #[arg(long, value_enum)]
    mode: Option<Mode>,

    /// Path to a Lox script to execute
    file: Option<PathBuf>,
}

fn run(source: &str, mode: Option<Mode>) -> Result<(), LoxError> {
    let scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens().map_err(LoxError::Compile)?;

    match mode {
        Some(Mode::Tokenize) => {
            for token in &tokens {
                println!("{token}");
            }
        }
        Some(Mode::Parse) => {
            let parser = Parser::new(&tokens);
            let expr = parser.parse().map_err(LoxError::Compile)?;
            println!("{expr}");
        }
        // Default: run the full pipeline (currently same as parse)
        None => {
            let parser = Parser::new(&tokens);
            let expr = parser.parse().map_err(LoxError::Compile)?;
            println!("{expr}");
        }
    }

    Ok(())
}

fn run_prompt(mode: Option<Mode>) -> io::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut line = String::new();

    loop {
        print!("> ");
        stdout.flush()?;

        line.clear();
        let bytes_read = stdin.read_line(&mut line)?;
        if bytes_read == 0 {
            break;
        }

        if let Err(e) = run(line.trim(), mode) {
            eprintln!("{e}");
        }
    }

    Ok(())
}

fn main() {
    let cli = Cli::parse();

    match cli.file {
        Some(path) => {
            let source = match std::fs::read_to_string(&path) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Could not read file '{}': {e}", path.display());
                    std::process::exit(1);
                }
            };
            if let Err(e) = run(&source, cli.mode) {
                eprintln!("{e}");
                std::process::exit(e.exit_code());
            }
        }
        None => {
            if let Err(e) = run_prompt(cli.mode) {
                eprintln!("{e}");
                std::process::exit(1);
            }
        }
    }
}
