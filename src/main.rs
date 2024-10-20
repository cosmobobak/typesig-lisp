#![warn(clippy::all, clippy::nursery, clippy::pedantic)]

mod tokenise;
mod parse;

use std::{
    fs::File,
    io::{BufRead, BufReader, Write},
    path::PathBuf,
    time::Instant,
};

use clap::Parser;
use color_eyre::eyre::{Context, ContextCompat};
use color_eyre::Report;
use color_eyre::Result;

#[derive(Parser)]
struct Args {
    /// An input file. Replaces stdin.
    #[clap(long, short)]
    file: Option<PathBuf>,
}

struct InterpreterState {
    show_timings: bool,
}

#[allow(clippy::derivable_impls)]
impl Default for InterpreterState {
    fn default() -> Self {
        Self {
            show_timings: false,
        }
    }
}

trait Mode {
    const INTERACTIVE: bool;
}

struct Interactive;
struct NonInteractive;

impl Mode for Interactive {
    const INTERACTIVE: bool = true;
}
impl Mode for NonInteractive {
    const INTERACTIVE: bool = false;
}

const READ_FILE_PREFIX: &str = "r ";
const TOKENISE_CMD: &str = ":tokenise ";
const PARSE_CMD: &str = ":parse ";

fn repl<M, I, E>(input: I) -> Result<()>
where
    M: Mode,
    I: Iterator<Item = Result<String, E>>,
    Report: From<E>,
{
    let mut state = InterpreterState::default();
    if M::INTERACTIVE {
        println!("Welcome to Cosmo's LISP interpreter! Enter \"quit\", \"exit\", or EOF to exit.");
        print!("lisp> ");
        std::io::stdout().flush()?;
    }
    for line in input {
        let start = Instant::now();
        let line = line?;
        match line.trim() {
            "" => (),
            "quit" | "exit" => return Ok(()),
            ":timing" => state.show_timings = true,
            stripped if line.starts_with(READ_FILE_PREFIX) => {
                let file_name = stripped.strip_prefix(READ_FILE_PREFIX);
                let res = file_name
                    .context("No file name provided!")
                    .and_then(|file_name| {
                        std::fs::read_to_string(file_name)
                            .with_context(|| format!("Couldn't read {file_name}!"))
                    });
                match res {
                    Ok(contents) => println!("{contents}"),
                    Err(err) => println!("{err:#}"),
                }
            }
            stripped if line.starts_with(TOKENISE_CMD) => {
                let tokens = stripped.strip_prefix(TOKENISE_CMD)
                    .context("No text provided!")
                    .map(tokenise::tokenise);
                match tokens {
                    Ok(tokens) => println!("{:?}", tokens.collect::<Vec<_>>()),
                    Err(err) => println!("{err:#}"),
                }
            }
            stripped if line.starts_with(PARSE_CMD) => {
                let tree = stripped.strip_prefix(PARSE_CMD)
                    .context("No text provided!")
                    .map(tokenise::tokenise)
                    .map(parse::parse);
                match tree {
                    Ok(tree) => println!("{tree}"),
                    Err(err) => println!("{err:#}"),
                }
            }
            line => println!("{}", parse::parse(tokenise::tokenise(line)).stringify()),
        }

        if M::INTERACTIVE {
            if state.show_timings {
                println!("completed in {}ns", start.elapsed().as_nanos());
            }
            print!("lisp> ");
            std::io::stdout().flush()?;
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Args::parse();
    if let Some(path) = args.file {
        let file = File::open(&path)
            .with_context(|| format!("failed to open file at {}", path.display()))?;
        let reader = BufReader::new(file);
        repl::<NonInteractive, _, _>(reader.lines())?;
    } else {
        repl::<Interactive, _, _>(std::io::stdin().lines())?;
    };

    Ok(())
}
