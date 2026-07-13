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

use asciidoc_html5::{AssetWriter, DirAssetWriter, Options, SafeMode};
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

    /// Set a document attribute (`name`, `name=value`, or `name!` to unset)
    #[arg(
        short = 'a',
        long = "attribute",
        value_name = "NAME[=VALUE]",
        long_help = "Set a document attribute from outside the document, the way \
Asciidoctor's -a option does.\n\n\
Give `name` to set an attribute, `name=value` to set it to a value, or `name!` \
(equivalently `!name`) to unset it. By default the value supplied here overrides any assignment of the \
same name inside the document. Append `@` (for example `name=value@`) to make it \
a soft default instead, which a document assignment of the same name overrides.\n\n\
Repeat -a to set several attributes."
    )]
    attribute: Vec<String>,

    /// Base directory for the document and its resources (default: input's dir)
    #[arg(
        short = 'B',
        long = "base-dir",
        value_name = "DIR",
        long_help = "Set the base directory, the way Asciidoctor's -B option does.\n\n\
The base directory is where filesystem-relative resources are resolved from. \
Today that means `include::` targets: a relative include resolves against the \
including file's directory, and under the `safe` and `server` safe modes reads \
may not climb above the base directory (a target that tries is recovered back \
inside). Under `unsafe` there is no such restriction; under `secure` includes \
become links and are never read.\n\n\
When omitted, the base directory is the directory containing the input file, or \
the current directory when the document is read from standard input."
    )]
    base_dir: Option<PathBuf>,

    /// Set the safe mode: unsafe, safe, server, or secure (default: unsafe)
    #[arg(
        short = 'S',
        long = "safe-mode",
        value_name = "SAFE_MODE",
        long_help = "Set the safe mode level, the way Asciidoctor's -S option does.\n\n\
The safe mode controls how far a document may reach outside itself, and (as in \
Asciidoctor) whether the default stylesheet is embedded or linked. Accepts \
`unsafe`, `safe`, `server`, or `secure` (case-insensitive).\n\n\
When omitted, adoc runs in `unsafe` mode — the Asciidoctor CLI default — which \
embeds the default stylesheet. The `secure` mode links it instead. See also \
--safe."
    )]
    safe_mode: Option<String>,

    /// Set the safe mode to safe (compatibility shorthand for --safe-mode=safe)
    #[arg(
        long = "safe",
        conflicts_with = "safe_mode",
        long_help = "Set the safe mode level to `safe`.\n\n\
Provided for compatibility with the Python AsciiDoc `safe` command, and \
equivalent to --safe-mode=safe. Cannot be combined with --safe-mode."
    )]
    safe: bool,
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
/// testable without spawning the binary. Reads from the process's standard
/// input; [`run_with_input`] is the same pipeline with an injectable reader, so
/// the `-`/stdin read path can be exercised in tests.
fn run(cli: &Cli, stdout: &mut dyn Write) -> io::Result<()> {
    let mut stdin = io::stdin().lock();
    run_with_input(cli, &mut stdin, stdout)
}

/// Reads the AsciiDoc input from `stdin` (when `-`/no input file) or a named
/// file, converts it, and writes the HTML5 out — the testable core of [`run`].
fn run_with_input(cli: &Cli, stdin: &mut dyn Read, stdout: &mut dyn Write) -> io::Result<()> {
    let mut options = build_options(&cli.attribute)?.safe_mode(resolve_safe_mode(cli)?);
    options = apply_base_dir(cli, options)?;

    let source = read_input(cli.input.as_deref(), stdin)?;

    match output_target(cli) {
        OutputTarget::File(path) => {
            // Write any companion stylesheet (`copycss`) into the output file's
            // directory, so a linked stylesheet lands next to the HTML that
            // references it — matching Asciidoctor, which copies only when
            // converting to a file. The guard keeps the copy from clobbering the
            // output file itself when the two paths coincide.
            let mut writer = OutputGuard {
                inner: DirAssetWriter::new(output_dir(&path)),
                output: path.clone(),
            };
            let html = asciidoc_html5::convert_with_writer(&source, &options, &mut writer)?;
            fs::write(path, html)
        }

        // Writing to standard output has no directory to copy alongside, so
        // `copycss` is inert here — again matching Asciidoctor, which skips the
        // copy unless there is an output file.
        OutputTarget::Stdout => {
            let html = asciidoc_html5::convert_with(&source, &options);
            stdout.write_all(html.as_bytes())
        }
    }
}

/// An [`AssetWriter`] wrapping a [`DirAssetWriter`] that refuses to write a
/// companion file onto the primary output path, warning instead.
///
/// This guards the contradictory `adoc -a linkcss -o asciidoctor.css …` case,
/// where the copied stylesheet and the output HTML resolve to the same file:
/// the `-o` output must win, so the copy is skipped rather than being
/// overwritten by (or overwriting) the HTML.
struct OutputGuard {
    /// The underlying filesystem writer, rooted at the output directory.
    inner: DirAssetWriter,

    /// The primary output file the copy must not collide with.
    output: PathBuf,
}

impl AssetWriter for OutputGuard {
    fn write_asset(&mut self, path: &Path, content: &[u8]) -> io::Result<()> {
        let dest = self.inner.destination(path);
        if same_file(&dest, &self.output) {
            eprintln!(
                "adoc: not copying the stylesheet to {}: it is the output file \
                 (choose a different -o, or unset copycss)",
                dest.display()
            );
            return Ok(());
        }
        self.inner.write_asset(path, content)
    }
}

/// Whether `a` and `b` name the same file, comparing their absolute (but not
/// symlink-resolved) forms so the check works before either file exists.
fn same_file(a: &Path, b: &Path) -> bool {
    match (std::path::absolute(a), std::path::absolute(b)) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    }
}

/// The directory to root companion-file writes at for an output file `path`:
/// its parent directory, or the current directory when `path` is a bare file
/// name.
fn output_dir(path: &Path) -> PathBuf {
    match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.to_path_buf(),
        _ => PathBuf::from("."),
    }
}

/// Records the base directory and primary file on `options`, mirroring
/// Asciidoctor's `-B`/`--base-dir`.
///
/// An explicit `-B` sets the base directory. Otherwise it is left to the
/// library to derive from the input file's directory, except when the document
/// is read from standard input — there is no file to derive from, so the
/// current directory is used, matching Asciidoctor. In every case the input
/// file (when there is one) is recorded so its top-level `include::` directives
/// resolve against its own directory.
///
/// # Errors
///
/// Returns an [`io::Error`] when the current directory is needed but cannot be
/// determined.
fn apply_base_dir(cli: &Cli, mut options: Options) -> io::Result<Options> {
    if let Some(dir) = &cli.base_dir {
        options = options.base_dir(dir.clone());
    } else if input_file(cli).is_none() {
        options = options.base_dir(std::env::current_dir()?);
    }

    if let Some(path) = input_file(cli) {
        options = options.input_file(path.to_path_buf());
    }

    Ok(options)
}

/// Returns the input file path when `adoc` reads from a real file, or `None`
/// when it reads from standard input (no `input`, or the conventional `-`).
fn input_file(cli: &Cli) -> Option<&Path> {
    match cli.input.as_deref() {
        Some(path) if path.as_os_str() != "-" => Some(path),
        _ => None,
    }
}

/// Resolves the [`SafeMode`] to convert under from the CLI's safe-mode options.
///
/// `--safe-mode=MODE` names the mode explicitly; the compatibility flag
/// `--safe` selects [`SafeMode::Safe`]. With neither, `adoc` defaults to
/// [`SafeMode::Unsafe`], matching the Asciidoctor CLI (which embeds the default
/// stylesheet rather than linking it).
///
/// # Errors
///
/// Returns an [`io::ErrorKind::InvalidInput`] error when `--safe-mode` names an
/// unrecognized mode.
fn resolve_safe_mode(cli: &Cli) -> io::Result<SafeMode> {
    if let Some(name) = &cli.safe_mode {
        return parse_safe_mode(name);
    }
    if cli.safe {
        return Ok(SafeMode::Safe);
    }
    Ok(SafeMode::Unsafe)
}

/// Parses a safe-mode name (case-insensitive) into a [`SafeMode`].
///
/// # Errors
///
/// Returns an [`io::ErrorKind::InvalidInput`] error when `name` is not one of
/// `unsafe`, `safe`, `server`, or `secure`.
fn parse_safe_mode(name: &str) -> io::Result<SafeMode> {
    match name.to_lowercase().as_str() {
        "unsafe" => Ok(SafeMode::Unsafe),
        "safe" => Ok(SafeMode::Safe),
        "server" => Ok(SafeMode::Server),
        "secure" => Ok(SafeMode::Secure),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("invalid safe mode '{name}': expected unsafe, safe, server, or secure"),
        )),
    }
}

/// Builds the conversion [`Options`] from the raw `-a`/`--attribute` specs,
/// parsing each with [`apply_attribute_spec`].
fn build_options(specs: &[String]) -> io::Result<Options> {
    let mut options = Options::new();
    for spec in specs {
        options = apply_attribute_spec(options, spec)?;
    }
    Ok(options)
}

/// Parses one `-a` attribute spec and records it in `options`, mirroring
/// Asciidoctor's `-a` syntax:
///
/// - `name` sets the attribute; `name=value` sets it to a value; `name!` (or
///   `!name`) unsets it.
/// - A trailing `@` makes the assignment a soft default that a document
///   assignment of the same name overrides; without it, the value overrides the
///   document.
///
/// # Errors
///
/// Returns an [`io::ErrorKind::InvalidInput`] error when the spec has no
/// attribute name (for example, an empty string or a bare `!`).
fn apply_attribute_spec(options: Options, spec: &str) -> io::Result<Options> {
    // A trailing `@` marks a soft default the document may override; strip it
    // first so it does not become part of a value or name.
    let (body, soft) = match spec.strip_suffix('@') {
        Some(rest) => (rest, true),
        None => (spec, false),
    };

    // Split off a `=value` first, matching Asciidoctor: the key is everything
    // before the first `=` (a name cannot contain `=`), the value everything
    // after. A bare spec has no value.
    let (key, value) = match body.split_once('=') {
        Some((key, value)) => (key, Some(value)),
        None => (body, None),
    };

    // A `!` on either end of the key unsets the attribute and takes precedence
    // over any `=value` (which Asciidoctor discards). Otherwise a `=value`
    // assigns the value, and a bare key sets the attribute.
    let options = if let Some(name) = key.strip_prefix('!').or_else(|| key.strip_suffix('!')) {
        let name = validate_name(name, spec)?;
        if soft {
            options.unset_default(name)
        } else {
            options.unset(name)
        }
    } else if let Some(value) = value {
        let name = validate_name(key, spec)?;
        if soft {
            options.attribute_default(name, value)
        } else {
            options.attribute(name, value)
        }
    } else {
        let name = validate_name(key, spec)?;
        if soft {
            options.set_default(name)
        } else {
            options.set(name)
        }
    };

    Ok(options)
}

/// Returns `name` if it is a non-empty attribute name, or an
/// [`io::ErrorKind::InvalidInput`] error naming the offending `spec` otherwise.
fn validate_name<'a>(name: &'a str, spec: &str) -> io::Result<&'a str> {
    if name.is_empty() {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("invalid attribute '{spec}': missing attribute name"),
        ))
    } else {
        Ok(name)
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

/// Reads AsciiDoc source from `path`, or from `stdin` when `path` is `None` or
/// the conventional `-`.
///
/// The standard-input reader is passed in rather than taken from
/// [`io::stdin`] directly, so the stdin read path can be exercised in tests.
fn read_input(path: Option<&std::path::Path>, stdin: &mut dyn Read) -> io::Result<String> {
    match path {
        Some(path) if path.as_os_str() != "-" => fs::read_to_string(path),
        _ => {
            let mut buf = String::new();
            stdin.read_to_string(&mut buf)?;
            Ok(buf)
        }
    }
}

#[cfg(test)]
mod tests;
