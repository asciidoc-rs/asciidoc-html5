//! `adoc` — a command-line AsciiDoc to HTML5 converter.
//!
//! Reads AsciiDoc from one or more files (or standard input) and writes the
//! rendered HTML5 to a file or to standard output. Given a file and no
//! `-o`/`--output`, the output file name is derived from the input by swapping
//! its extension for `.html`, matching `asciidoctor document.adoc` producing
//! `document.html`. Several files can be converted in a single invocation, each
//! to its own derived output, and a quoted glob pattern (`'*.adoc'`) is
//! expanded by `adoc` itself the way `asciidoctor` does, so it works the same
//! on every platform.

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
adoc a.adoc b.adoc              Convert several files, each to its own .html\n  \
adoc '*.adoc'                   Convert every .adoc file in the directory\n  \
adoc document.adoc -o out.html  Convert a file; write the HTML to out.html\n  \
adoc document.adoc -o -         Convert a file; write the HTML to stdout\n  \
cat document.adoc | adoc        Convert AsciiDoc from stdin; write to stdout\n  \
cat document.adoc | adoc -e     Convert stdin; write just the body (embedded)\n\n\
Exit status is 0 on success, or 1 if any input cannot be read or its output \
cannot be written."
)]
struct Cli {
    /// AsciiDoc input files or glob patterns (omit or use `-` to read stdin)
    #[arg(
        value_name = "FILE",
        long_help = "Paths to the AsciiDoc documents to convert.\n\n\
Pass several files to convert each in turn, writing each to its own output \
(derived, or the single -o target). An argument that names no existing file is \
treated as a glob pattern and expanded by adoc itself — the same portable, \
Ruby-style matching asciidoctor performs — so `'*.adoc'` converts every .adoc \
file in the directory and `'**/*.adoc'` recurses into subdirectories at any \
depth. Quote the pattern so the shell passes it through rather than expanding \
it first.\n\n\
When omitted, or given as a single dash (`-`), adoc reads the document from \
standard input instead, so it can sit at the end of a pipeline."
    )]
    inputs: Vec<PathBuf>,

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

    /// Produce embedded (body-only) output instead of a standalone document
    #[arg(
        short = 'e',
        long = "embedded",
        long_help = "Produce embedded output, the way Asciidoctor's -e option does.\n\n\
By default adoc writes a standalone HTML5 document — the full \
<!DOCTYPE>/<head>/<body> shell around the header, content, and footer. With -e \
it writes just the converted body, with no document shell, stylesheet, or \
header/footer frame, suitable for dropping into a surrounding template.\n\n\
Embedded output omits the doctitle by default; add `-a showtitle` to include it \
as a leading <h1>."
    )]
    embedded: bool,
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
/// The destination follows [`output_target_for`]: a file named by
/// `-o`/`--output`, a file whose name is derived from the input, or `stdout`.
/// Threading the
/// standard-output writer in as a parameter keeps the conversion pipeline
/// testable without spawning the binary. Reads from the process's standard
/// input; [`run_with_input`] is the same pipeline with an injectable reader, so
/// the `-`/stdin read path can be exercised in tests.
fn run(cli: &Cli, stdout: &mut dyn Write) -> io::Result<()> {
    let mut stdin = io::stdin().lock();
    run_with_input(cli, &mut stdin, stdout)
}

/// Reads the AsciiDoc input from `stdin` (when `-`/no input file) or the named
/// files, converts each, and writes the HTML5 out — the testable core of
/// [`run`].
///
/// The command's positional arguments are first resolved into a list of
/// [`InputSource`]s by [`resolve_inputs`], expanding any glob patterns the way
/// Asciidoctor does. Each source is then converted in turn: several files in
/// one invocation each produce their own output. Options shared across every
/// source (the attributes, safe mode, and standalone/embedded choice) are built
/// once; the per-source base directory and input file are layered on top for
/// each.
fn run_with_input(cli: &Cli, stdin: &mut dyn Read, stdout: &mut dyn Write) -> io::Result<()> {
    // Unlike the library's string API (embedded by default), the CLI defaults to
    // a standalone document — matching Asciidoctor's command, which writes a full
    // document even when piping STDIN to STDOUT. `-e`/`--embedded` opts into
    // body-only output. Setting the mode explicitly here also makes `-e` produce
    // embedded output when writing to a file, not just to standard output.
    let base_options = build_options(&cli.attribute)?
        .safe_mode(resolve_safe_mode(cli)?)
        .standalone(!cli.embedded);

    for source in resolve_inputs(&cli.inputs)? {
        convert_source(cli, &base_options, &source, stdin, stdout)?;
    }

    Ok(())
}

/// Converts one [`InputSource`] and writes its HTML5 to the destination
/// [`output_target_for`] picks for it.
///
/// The shared `base_options` are cloned and the source's own base directory and
/// input file are applied, so a file's top-level `include::` targets resolve
/// against its own directory and each file gets its own derived output name.
fn convert_source(
    cli: &Cli,
    base_options: &Options,
    source: &InputSource,
    stdin: &mut dyn Read,
    stdout: &mut dyn Write,
) -> io::Result<()> {
    let input = source.file();
    let options = apply_base_dir(cli, base_options.clone(), input)?;
    let source_text = read_input(input, stdin)?;

    match output_target_for(cli, input) {
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
            let html = asciidoc_html5::convert_with_writer(&source_text, &options, &mut writer)?;
            fs::write(path, html)
        }

        // Writing to standard output has no directory to copy alongside, so
        // `copycss` is inert here — again matching Asciidoctor, which skips the
        // copy unless there is an output file.
        OutputTarget::Stdout => {
            let html = asciidoc_html5::convert_with(&source_text, &options);
            stdout.write_all(html.as_bytes())
        }
    }
}

/// A single resolved source for `adoc` to convert: an on-disk file, or standard
/// input.
enum InputSource {
    /// A file named on the command line (or matched by a glob pattern).
    File(PathBuf),

    /// Standard input, selected by a lone `-` or no input argument at all.
    Stdin,
}

impl InputSource {
    /// The file this source reads from, or `None` when it reads standard input.
    fn file(&self) -> Option<&Path> {
        match self {
            InputSource::File(path) => Some(path),
            InputSource::Stdin => None,
        }
    }
}

/// Resolves the command's positional arguments into the ordered list of sources
/// to convert, mirroring how the Asciidoctor CLI treats its input arguments.
///
/// With no arguments, or a lone `-`, `adoc` reads standard input. Otherwise
/// each argument names a file to convert, except that an argument matching no
/// existing file is expanded as a glob pattern — the same portable, Ruby-style
/// matching Asciidoctor performs, so `'*.adoc'` and `'**/*.adoc'` work the same
/// on every platform regardless of what the shell would expand. A pattern that
/// matches nothing is kept as-is, so it surfaces as a missing-file error when
/// the conversion tries to read it, again matching Asciidoctor.
fn resolve_inputs(inputs: &[PathBuf]) -> io::Result<Vec<InputSource>> {
    // No input argument, or a single `-`, reads standard input — the same two
    // spellings the single-file path already treats as stdin.
    if inputs.is_empty() || (inputs.len() == 1 && inputs[0].as_os_str() == "-") {
        return Ok(vec![InputSource::Stdin]);
    }

    let mut sources = Vec::new();
    for arg in inputs {
        if arg.as_os_str() == "-" {
            sources.push(InputSource::Stdin);
        } else if arg.is_file() {
            // An argument naming an existing file is taken literally; Asciidoctor
            // only globs when the file is not found.
            sources.push(InputSource::File(arg.clone()));
        } else {
            let matches = expand_glob(arg)?;
            if matches.is_empty() {
                // No matches: keep the literal argument so the read step reports
                // it as missing, exactly as a plain misspelled filename would.
                sources.push(InputSource::File(arg.clone()));
            } else {
                sources.extend(matches.into_iter().map(InputSource::File));
            }
        }
    }

    Ok(sources)
}

/// Expands `pattern` as a glob, returning the matching files in sorted order.
///
/// Matching follows the `glob` crate, whose `*`, `?`, `**`, and `[…]` semantics
/// line up with the Ruby `Dir.glob` rules Asciidoctor uses — including `**`,
/// which spans directories at any depth (and zero depth, so `**/*.adoc` also
/// matches files in the current directory). Only files are returned; directory
/// matches are dropped so a pattern never tries to convert a directory. The
/// results are sorted so the conversion order is deterministic across
/// platforms.
///
/// # Errors
///
/// Returns an [`io::ErrorKind::InvalidInput`] error when `pattern` is not valid
/// UTF-8 or is not a valid glob pattern.
fn expand_glob(pattern: &Path) -> io::Result<Vec<PathBuf>> {
    let pattern = pattern.to_str().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("invalid input path {}: not valid UTF-8", pattern.display()),
        )
    })?;

    let paths = glob::glob(pattern)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err.to_string()))?;

    // Keep only readable file matches, discarding entries the glob crate could
    // not stat and any directories it matched.
    let mut matches: Vec<PathBuf> = paths
        .filter_map(Result::ok)
        .filter(|path| path.is_file())
        .collect();

    matches.sort();
    Ok(matches)
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

/// Records the base directory and the given `input` file on `options`,
/// mirroring Asciidoctor's `-B`/`--base-dir`.
///
/// An explicit `-B` sets the base directory. Otherwise it is left to the
/// library to derive from the input file's directory, except when the document
/// is read from standard input (`input` is `None`) — there is no file to derive
/// from, so the current directory is used, matching Asciidoctor. In every case
/// the input file (when there is one) is recorded so its top-level `include::`
/// directives resolve against its own directory.
///
/// # Errors
///
/// Returns an [`io::Error`] when the current directory is needed but cannot be
/// determined.
fn apply_base_dir(cli: &Cli, mut options: Options, input: Option<&Path>) -> io::Result<Options> {
    if let Some(dir) = &cli.base_dir {
        options = options.base_dir(dir.clone());
    } else if input.is_none() {
        options = options.base_dir(std::env::current_dir()?);
    }

    if let Some(path) = input {
        options = options.input_file(path.to_path_buf());
    }

    Ok(options)
}

/// Returns the input file path when `adoc` reads from a single real file, or
/// `None` when it reads from standard input (no input argument, or the
/// conventional `-`).
///
/// This reports the *first* input for the invocation, which is all the
/// stdin-versus-file distinction needs; the multi-file conversion loop resolves
/// each source's own file through [`resolve_inputs`]. It is a convenience for
/// the tests, which exercise `adoc`'s single-input routing.
#[cfg(test)]
fn input_file(cli: &Cli) -> Option<&Path> {
    match cli.inputs.first() {
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

/// Decides where to write the HTML5 for the invocation's first input, mirroring
/// Asciidoctor's default behavior.
///
/// A convenience over [`output_target_for`] that reads the first input through
/// [`input_file`]; the multi-file conversion loop calls [`output_target_for`]
/// once per source instead. Used by the tests, which exercise `adoc`'s
/// single-input routing.
#[cfg(test)]
fn output_target(cli: &Cli) -> OutputTarget {
    output_target_for(cli, input_file(cli))
}

/// Decides where to write the HTML5 for the source reading from `input`,
/// mirroring Asciidoctor's default behavior.
///
/// With `-o`/`--output`, the value names the destination directly, except that
/// the conventional `-` selects standard output. Without it, the output file
/// name is derived from `input` by replacing its extension with `.html` and
/// writing alongside it, so `adoc document.adoc` writes `document.html`, and
/// each file in a multi-file invocation lands in its own `.html`. When the
/// source is standard input (`input` is `None`) there is no name to derive
/// from, so the HTML5 goes to standard output.
fn output_target_for(cli: &Cli, input: Option<&Path>) -> OutputTarget {
    match cli.output.as_deref() {
        Some(path) if path.as_os_str() == "-" => OutputTarget::Stdout,
        Some(path) => OutputTarget::File(path.to_path_buf()),
        None => match input {
            Some(input) => OutputTarget::File(derive_output_path(input)),
            None => OutputTarget::Stdout,
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
