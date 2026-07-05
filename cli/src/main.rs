//! `adoc` — a command-line AsciiDoc to HTML5 converter.
//!
//! Reads AsciiDoc from a file (or standard input) and writes the rendered
//! HTML5 to a file or to standard output. Given a file and no `-o`/`--output`,
//! the output file name is derived from the input by swapping its extension for
//! `.html`, matching `asciidoctor document.adoc` producing `document.html`.

use std::{
    fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
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
asciidoc-html5 library, and writes the resulting HTML5 to a file or to standard \
output. Given a file and no -o option, adoc derives the output file name from \
the input, replacing its extension with .html, so `adoc document.adoc` writes \
document.html. The output aims to be compatible with Asciidoctor's default html5 \
backend.",
    after_help = "Use -h for a short summary or --help for the full description.",
    after_long_help = "Examples:\n  \
adoc document.adoc              Convert a file; write the HTML to document.html\n  \
adoc document.adoc -o out.html  Convert a file; write the HTML to out.html\n  \
adoc document.adoc -o -         Convert a file; write the HTML to stdout\n  \
cat document.adoc | adoc        Convert AsciiDoc from stdin; write to stdout\n\n\
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

    /// Write HTML to this file (`-` for stdout; default: derived from input)
    #[arg(
        short,
        long,
        value_name = "FILE",
        long_help = "Path of the file to write the rendered HTML5 to.\n\n\
When omitted, adoc derives the output file name from the input file by replacing \
its extension with .html and writing alongside it. Pass a single dash (`-`) to \
write to standard output instead. When the input is read from standard input, \
there is no name to derive from, so adoc writes to standard output."
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
/// The destination follows [`output_target`]: a file named by `-o`/`--output`,
/// a file whose name is derived from the input, or `stdout`. Threading the
/// standard-output writer in as a parameter keeps the conversion pipeline
/// testable without spawning the binary.
fn run(cli: &Cli, stdout: &mut dyn Write) -> io::Result<()> {
    let source = read_input(cli.input.as_deref())?;

    let html = asciidoc_html5::convert(&source);

    match output_target(cli) {
        OutputTarget::File(path) => fs::write(path, html),
        OutputTarget::Stdout => stdout.write_all(html.as_bytes()),
    }
}

/// Where the rendered HTML5 should be written.
enum OutputTarget {
    /// A file on disk.
    File(PathBuf),

    /// Standard output.
    Stdout,
}

/// Decides where to write the HTML5, mirroring Asciidoctor's default behavior.
///
/// With `-o`/`--output`, the value names the destination directly, except that
/// the conventional `-` selects standard output. Without it, the output file
/// name is derived from the input by replacing its extension with `.html` and
/// writing alongside it, so `adoc document.adoc` writes `document.html`. When
/// the input comes from standard input there is no name to derive from, so the
/// HTML5 goes to standard output.
fn output_target(cli: &Cli) -> OutputTarget {
    match cli.output.as_deref() {
        Some(path) if path.as_os_str() == "-" => OutputTarget::Stdout,
        Some(path) => OutputTarget::File(path.to_path_buf()),
        None => match cli.input.as_deref() {
            Some(input) if input.as_os_str() != "-" => {
                OutputTarget::File(derive_output_path(input))
            }
            _ => OutputTarget::Stdout,
        },
    }
}

/// Derives the output file path from the input path by swapping its extension
/// for `.html`, matching how `asciidoctor` names its output file.
fn derive_output_path(input: &Path) -> PathBuf {
    input.with_extension("html")
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
