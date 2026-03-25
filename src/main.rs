use std::{io::{self, Write}, path::PathBuf};

use anyhow::{Context, Result};
use clap::Parser;

#[derive(Parser)]
#[command(name = "loxrs", about = "A Lox language interpreter")]
struct Cli {
    /// Path to a Lox script to execute
    file: Option<PathBuf>,
}

fn run(source: &str) {
    // Stub: will be replaced with scanning, parsing, and interpreting
    println!("{source}");
}

fn run_file(path: &PathBuf) -> Result<()> {
    let source = std::fs::read_to_string(path)
        .with_context(|| format!("could not read file '{path:?}'"))?;
    run(&source);
    Ok(())
}

fn run_prompt() -> Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut line = String::new();

    loop {
        print!("> ");
        stdout.flush()?;

        line.clear();
        let bytes_read = stdin.read_line(&mut line)?;
        if bytes_read == 0 {
            break; // EOF
        }

        run(line.trim());
    }

    Ok(())
}

fn main() {
    let cli = Cli::parse();

    match cli.file {
        Some(path) => {
            if let Err(e) = run_file(&path) {
                eprintln!("{e}");
                std::process::exit(1);
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
