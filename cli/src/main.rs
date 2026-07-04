//! `adoc` — a command-line AsciiDoc to HTML5 converter.
//!
//! Reads AsciiDoc from a file (or standard input) and writes the rendered
//! HTML5 to standard output or to a file chosen with `-o`/`--output`.

use std::{
    fs,
    io::{self, Read, Write},
    path::PathBuf,
    process::ExitCode,
};

use clap::Parser;

/// Convert an AsciiDoc document to HTML5.
#[derive(Debug, Parser)]
#[command(name = "adoc", version, about)]
struct Cli {
    /// AsciiDoc input file. When omitted (or given as `-`), read from stdin.
    input: Option<PathBuf>,

    /// Write output to this file instead of stdout.
    #[arg(short, long, value_name = "FILE")]
    output: Option<PathBuf>,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match run(&cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("adoc: {err}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: &Cli) -> io::Result<()> {
    let source = read_input(cli.input.as_deref())?;

    let html = asciidoc_html5::convert(&source);

    write_output(cli.output.as_deref(), &html)
}

/// Reads AsciiDoc source from `path`, or from stdin when `path` is `None` or
/// the conventional `-`.
fn read_input(path: Option<&std::path::Path>) -> io::Result<String> {
    match path {
        Some(path) if path.as_os_str() != "-" => fs::read_to_string(path),
        _ => {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf)?;
            Ok(buf)
        }
    }
}

/// Writes `html` to `path`, or to stdout when `path` is `None`.
fn write_output(path: Option<&std::path::Path>, html: &str) -> io::Result<()> {
    match path {
        Some(path) => fs::write(path, html),
        None => io::stdout().write_all(html.as_bytes()),
    }
}
