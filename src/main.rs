mod error;

use std::io::{self, Write};
use std::path::PathBuf;

use clap::Parser;

use crate::error::LoxError;

#[derive(Parser)]
#[command(name = "loxrs", about = "A Lox language interpreter")]
struct Cli {
    /// Path to a Lox script to execute
    file: Option<PathBuf>,
}

fn run(_source: &str) -> Result<(), LoxError> {
    // Stub: will be replaced with scanning, parsing, and interpreting
    Ok(())
}

fn run_prompt() -> io::Result<()> {
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

        if let Err(e) = run(line.trim()) {
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
            if let Err(e) = run(&source) {
                eprintln!("{e}");
                std::process::exit(e.exit_code());
            }
        }
        None => {
            if let Err(e) = run_prompt() {
                eprintln!("{e}");
                std::process::exit(1);
            }
        }
    }
}
