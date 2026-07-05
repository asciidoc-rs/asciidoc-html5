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
#[command(
    name = "adoc",
    version,
    about = "Convert an AsciiDoc document to HTML5.",
    long_about = "Convert an AsciiDoc document to a standalone HTML5 document.\n\n\
adoc reads AsciiDoc from a file or from standard input, renders it with the \
asciidoc-html5 library, and writes the resulting HTML5 to standard output or to \
a file. The output aims to be compatible with Asciidoctor's default html5 backend.",
    after_help = "Use -h for a short summary or --help for the full description.",
    after_long_help = "Examples:\n  \
adoc document.adoc              Convert a file and print the HTML to stdout\n  \
adoc document.adoc -o out.html  Convert a file and write the HTML to out.html\n  \
cat document.adoc | adoc        Convert AsciiDoc read from standard input\n\n\
Exit status is 0 on success, or 1 if the input cannot be read or the output \
cannot be written."
)]
struct Cli {
    /// AsciiDoc input file (omit or use `-` to read stdin)
    #[arg(
        value_name = "FILE",
        long_help = "Path to the AsciiDoc document to convert.\n\n\
When omitted, or given as a single dash (`-`), adoc reads the document from \
standard input instead, so it can sit at the end of a pipeline."
    )]
    input: Option<PathBuf>,

    /// Write HTML to this file instead of stdout
    #[arg(
        short,
        long,
        value_name = "FILE",
        long_help = "Path of the file to write the rendered HTML5 to.\n\n\
When omitted, adoc writes the HTML to standard output."
    )]
    output: Option<PathBuf>,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let mut stdout = io::stdout().lock();
    match run(&cli, &mut stdout) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("adoc: {err}");
            ExitCode::FAILURE
        }
    }
}

/// Reads the AsciiDoc input, converts it, and writes the HTML5 out.
///
/// Output goes to the file named by `-o`/`--output` when one is given, and to
/// `stdout` otherwise. Threading the standard-output writer in as a parameter
/// keeps the conversion pipeline testable without spawning the binary.
fn run(cli: &Cli, stdout: &mut dyn Write) -> io::Result<()> {
    let source = read_input(cli.input.as_deref())?;

    let html = asciidoc_html5::convert(&source);

    match cli.output.as_deref() {
        Some(path) => fs::write(path, html),
        None => stdout.write_all(html.as_bytes()),
    }
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

#[cfg(test)]
mod tests;
