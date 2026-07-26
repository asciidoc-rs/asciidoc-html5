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

use asciidoc_html5::{AssetWriter, DirAssetWriter, Document, Options, SafeMode};
use asciidoc_parser::{
    parser::SourceLine,
    warnings::{Warning, WarningType},
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
adoc a.adoc b.adoc              Convert several files, each to its own .html\n  \
adoc '*.adoc'                   Convert every .adoc file in the directory\n  \
adoc document.adoc -o out.html  Convert a file; write the HTML to out.html\n  \
adoc document.adoc -D build     Convert a file; write the HTML to build/document.html\n  \
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
there is no name to derive from, so adoc writes to standard output.\n\n\
A relative path is resolved against the current directory (or the -D destination \
directory, when one is given), not against the input file's directory."
    )]
    output: Option<PathBuf>,

    /// Write output to this directory (default: the input file's directory)
    #[arg(
        short = 'D',
        long = "destination-dir",
        value_name = "DIR",
        long_help = "Set the output directory, the way Asciidoctor's -D option does.\n\n\
Write the rendered HTML5 (and any companion files, such as a linked stylesheet) \
into this directory instead of alongside the input file. The output file keeps \
its derived name unless -o names one; a relative -o path is resolved inside this \
directory, while an absolute -o path is used unchanged. The directory is created \
if it does not exist.\n\n\
When omitted, output goes to the input file's directory, or to the location -o \
names."
    )]
    destination_dir: Option<PathBuf>,

    /// Source root for recreating input subdirectories under `-D`
    #[arg(
        short = 'R',
        long = "source-dir",
        value_name = "DIR",
        long_help = "Set the source root directory, the way Asciidoctor's -R option does.\n\n\
Names a directory that the input files live under, so that -D recreates each \
input's subdirectory structure inside the destination directory. With both set, \
converting `src/sub/index.adoc` under `-R src -D out` writes `out/sub/index.html` \
rather than flattening it to `out/index.html`. Only inputs actually located \
below the source directory are relocated this way; an input elsewhere keeps the \
plain -D behavior.\n\n\
Has no effect without -D (there is no destination directory to build the tree \
under), and none on a document read from standard input (there is no input path \
to take a subdirectory from)."
    )]
    source_dir: Option<PathBuf>,

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

    /// Select the output backend; only `html5` is supported (default: html5)
    #[arg(
        short = 'b',
        long = "backend",
        value_name = "BACKEND",
        long_help = "Select the output backend, the way Asciidoctor's -b option does.\n\n\
adoc produces only the HTML5 backend, so the sole accepted value is `html5` \
(case-insensitive), which — like omitting the option — is a no-op accepted for \
command-line compatibility, so an existing `asciidoctor -b html5 …` invocation \
runs unchanged.\n\n\
Any other backend Asciidoctor documents (`xhtml5`, `docbook5`, `manpage`, and \
the like) is not implemented here, so it is rejected with a non-zero exit rather \
than being silently ignored."
    )]
    backend: Option<String>,

    /// Select the document type; only `article` is supported (default: article)
    #[arg(
        short = 'd',
        long = "doctype",
        value_name = "DOCTYPE",
        long_help = "Select the document type, the way Asciidoctor's -d option does.\n\n\
adoc models only the `article` doctype, so the sole accepted value is `article` \
(case-insensitive), which — like omitting the option — is a no-op accepted for \
command-line compatibility, so an existing `asciidoctor -d article …` invocation \
runs unchanged.\n\n\
The other doctypes Asciidoctor defines (`book`, `manpage`, and `inline`) are not \
implemented here, so each is rejected with a non-zero exit rather than being \
silently ignored."
    )]
    doctype: Option<String>,

    /// Auto-number section titles (sets the `sectnums` attribute)
    #[arg(
        short = 'n',
        long = "section-numbers",
        long_help = "Auto-number section titles, the way Asciidoctor's -n option does.\n\n\
Equivalent to setting the `sectnums` document attribute (`-a sectnums`): each \
section heading is prefixed with its dotted section number (`1.`, `1.1.`, …), \
nesting down to the `sectnumlevels` depth (3 by default). Numbering is off \
unless this flag, or the attribute, turns it on.\n\n\
This flag seeds `sectnums` before any -a option on the same command line, so an \
explicit `-a sectnums!` still wins and leaves the headings unnumbered."
    )]
    section_numbers: bool,

    /// Produce embedded (body-only) output instead of a standalone document
    #[arg(
        short = 'e',
        visible_short_alias = 's',
        long = "embedded",
        visible_alias = "no-header-footer",
        long_help = "Produce embedded output, the way Asciidoctor's -e option does.\n\n\
By default adoc writes a standalone HTML5 document — the full \
<!DOCTYPE>/<head>/<body> shell around the header, content, and footer. With -e \
it writes just the converted body, with no document shell, stylesheet, or \
header/footer frame, suitable for dropping into a surrounding template.\n\n\
Asciidoctor's original spelling, -s/--no-header-footer (which -e/--embedded \
superseded), is accepted as an alias, so `asciidoctor -s` and \
`asciidoctor --no-header-footer` invocations run unchanged. Note that -s is \
distinct from -S/--safe-mode.\n\n\
Embedded output omits the doctitle by default; add `-a showtitle` to include it \
as a leading <h1>."
    )]
    embedded: bool,

    /// Silence log messages, including parser warnings (default: off)
    #[arg(
        short = 'q',
        long = "quiet",
        conflicts_with = "verbose",
        long_help = "Silence log messages, the way Asciidoctor's -q option does.\n\n\
By default adoc prints the parser's warnings to standard error. With -q it \
prints nothing, whatever the document contains. A --failure-level threshold is \
still evaluated against the (now unprinted) diagnostics, so `-q \
--failure-level=WARN` can still exit non-zero on a warning without emitting any \
text.\n\n\
Cannot be combined with -v/--verbose."
    )]
    quiet: bool,

    /// Show verbose (info-level) log messages too (default: off)
    #[arg(
        short = 'v',
        long = "verbose",
        long_help = "Show verbose log messages, the way Asciidoctor's -v option does.\n\n\
By default adoc reports only warnings. With -v it also reports the lower, \
info-level diagnostics — for example a reference to a missing attribute, or a \
cross reference whose target is not defined — which are otherwise suppressed as \
likely false positives.\n\n\
Cannot be combined with -q/--quiet."
    )]
    verbose: bool,

    /// Accepted for compatibility with Asciidoctor's -w; has no effect
    #[arg(
        short = 'w',
        long = "warnings",
        long_help = "Accepted for compatibility with Asciidoctor's -w/--warnings option.\n\n\
In Asciidoctor -w turns on the Ruby interpreter's script warnings, a concern \
with no analog in this native binary, so adoc accepts the flag and does \
nothing with it. Parser warnings are printed by default regardless (silence \
them with -q); this flag does not govern them."
    )]
    warnings: bool,

    /// Minimum level that yields a non-zero exit: INFO, WARN, ERROR, or FATAL
    #[arg(
        long = "failure-level",
        value_name = "LEVEL",
        default_value = "FATAL",
        long_help = "Set the minimum log level that yields a non-zero exit code, the way \
Asciidoctor's --failure-level option does.\n\n\
Accepts `INFO`, `WARN` (`WARNING`), `ERROR`, or `FATAL` (case-insensitive). \
When a diagnostic at or above this level is emitted, adoc exits with status 1 \
after finishing the conversion. The parser's own diagnostics are warnings (with \
a handful reported at the lower info level), so `--failure-level=WARN` turns any \
warning into a failure and `--failure-level=INFO` turns any diagnostic into \
one.\n\n\
The default, `FATAL`, is effectively never reached by a parse — matching \
Asciidoctor, whose conversions succeed despite warnings unless this option \
lowers the bar."
    )]
    failure_level: String,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let mut stdin = io::stdin().lock();
    let mut stdout = io::stdout().lock();
    let mut stderr = io::stderr().lock();
    match run_with_streams(&cli, &mut stdin, &mut stdout, &mut stderr) {
        // A run that reached the configured `--failure-level` finishes the
        // conversion but exits non-zero, matching Asciidoctor.
        Ok(false) => ExitCode::SUCCESS,
        Ok(true) => ExitCode::FAILURE,
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
/// input, and prints parser warnings to the process's standard error;
/// [`run_with_streams`] is the same pipeline with those two streams injectable,
/// so the `-`/stdin read path and the warning output can be exercised in tests.
///
/// The failure-level exit code [`run_with_streams`] computes is discarded here
/// (this returns only success or the propagated I/O error); the binary's
/// entry point calls [`run_with_streams`] directly to honor it.
///
/// This convenience over [`run_with_streams`] is used by the tests, which
/// exercise conversions that emit no warnings and so need neither the injected
/// warning stream nor the exit code.
#[cfg(test)]
fn run(cli: &Cli, stdout: &mut dyn Write) -> io::Result<()> {
    let mut stdin = io::stdin().lock();
    run_with_input(cli, &mut stdin, stdout)
}

/// Reads the AsciiDoc input from `stdin` (when `-`/no input file) or the named
/// files, converts each, and writes the HTML5 out — the injectable-reader
/// counterpart of [`run`], routing warnings to the process's standard error.
///
/// Like [`run`], the failure-level exit code is discarded; use
/// [`run_with_streams`] to observe it and to capture the warning stream.
#[cfg(test)]
fn run_with_input(cli: &Cli, stdin: &mut dyn Read, stdout: &mut dyn Write) -> io::Result<()> {
    let mut stderr = io::stderr().lock();
    run_with_streams(cli, stdin, stdout, &mut stderr).map(|_| ())
}

/// Reads the AsciiDoc input from `stdin` (when `-`/no input file) or the named
/// files, converts each, writes the HTML5 to `stdout`, and prints any parser
/// warnings to `stderr` — the fully injectable core of [`run`].
///
/// The command's positional arguments are first resolved into a list of
/// [`InputSource`]s by [`resolve_inputs`], expanding any glob patterns the way
/// Asciidoctor does. Each source is then converted in turn: several files in
/// one invocation each produce their own output. Options shared across every
/// source (the attributes, safe mode, and standalone/embedded choice) are built
/// once; the per-source base directory and input file are layered on top for
/// each.
///
/// Parsing surfaces warnings (see [`Document::warnings`]); how they reach
/// `stderr`, and whether they raise the exit code, is governed by the
/// `-q`/`-v`/`--failure-level` options, gathered once into a
/// [`WarningReporter`]. Returns `true` when any source emits a diagnostic at or
/// above the configured failure level (regardless of whether it was printed),
/// so the caller exits non-zero; `false` otherwise.
fn run_with_streams(
    cli: &Cli,
    stdin: &mut dyn Read,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> io::Result<bool> {
    // Reject an unsupported backend or doctype before reading or converting
    // anything, so an `-b docbook5` or `-d book` invocation fails cleanly without
    // touching the input.
    check_backend(cli)?;
    check_doctype(cli)?;

    // Unlike the library's string API (embedded by default), the CLI defaults to
    // a standalone document — matching Asciidoctor's command, which writes a full
    // document even when piping STDIN to STDOUT. `-e`/`--embedded` opts into
    // body-only output. Setting the mode explicitly here also makes `-e` produce
    // embedded output when writing to a file, not just to standard output.
    let base_options = build_options(cli.section_numbers, &cli.attribute)?
        .safe_mode(resolve_safe_mode(cli)?)
        .standalone(!cli.embedded);

    let reporter = WarningReporter::from_cli(cli)?;

    let sources = resolve_inputs(&cli.inputs)?;

    // Every on-disk input in this invocation. A source's resolved output must not
    // name any of them — not only its own input — or converting one source would
    // truncate another (or itself) before that source is read.
    let input_paths: Vec<PathBuf> = sources
        .iter()
        .filter_map(InputSource::file)
        .map(Path::to_path_buf)
        .collect();

    // A single source reaching the failure level fails the whole invocation, but
    // every source is still converted first — matching Asciidoctor, which sets
    // its exit code from the highest severity seen across all inputs.
    let mut failure_reached = false;
    for source in &sources {
        failure_reached |= convert_source(
            cli,
            &base_options,
            source,
            &input_paths,
            &reporter,
            stdin,
            stdout,
            stderr,
        )?;
    }

    Ok(failure_reached)
}

/// Converts one [`InputSource`] and writes its HTML5 to the destination
/// [`output_target_for`] picks for it, printing the source's parser warnings to
/// `stderr` through `reporter`.
///
/// The shared `base_options` are cloned and the source's own base directory and
/// input file are applied, so a file's top-level `include::` targets resolve
/// against its own directory and each file gets its own derived output name.
///
/// Returns whether this source emitted a diagnostic at or above the reporter's
/// failure level, so the caller can raise the invocation's exit code.
#[allow(clippy::too_many_arguments)]
fn convert_source(
    cli: &Cli,
    base_options: &Options,
    source: &InputSource,
    input_paths: &[PathBuf],
    reporter: &WarningReporter,
    stdin: &mut dyn Read,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> io::Result<bool> {
    let input = source.file();
    let options = apply_base_dir(cli, base_options.clone(), input)?;
    let target = output_target_for(cli, input);

    // Refuse to write the output onto any input file in this invocation, matching
    // Asciidoctor's refusal to convert a file onto itself — and extending it
    // across a multi-file run, where one source's resolved output (named with
    // `-o` or derived, e.g. via an `outfilesuffix` landing on an input's
    // extension) could otherwise truncate a sibling input before it is read. The
    // comparison (see [`same_file`]) tests file identity, so an output that
    // aliases an input through a symlink or a hard link is caught too. Fail
    // before reading or converting.
    //
    // This check is best-effort against *accidental* clobbering, not a race-free
    // atomic write: it runs here, but the actual write happens after the input is
    // read and converted, so a concurrent process could swap the output path for
    // an alias to an input in between (the identity checked is not bound to the
    // file finally written). Matching Asciidoctor, which checks the same way, the
    // TOCTOU race is left open; closing it would mean verifying the opened output
    // handle's identity before truncating. Tracked in
    // <https://github.com/asciidoc-rs/asciidoc-html5/issues/170>.
    if let OutputTarget::File(path) = &target {
        if input_paths.iter().any(|input| same_file(path, input)) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "input file and output file cannot be the same",
            ));
        }
    }

    let source_text = read_input(input, stdin)?;

    // Load the document once so its warnings can be surfaced to `stderr`, then
    // render that same parse — rather than parsing a second time — to `stdout` or
    // the output file. The reporter both prints the warnings (subject to
    // `-q`/`-v`) and reports whether any reached the failure level.
    let document = asciidoc_html5::load_with(&source_text, &options);
    let failure_reached = reporter.report(&document, warning_document_name(input), stderr)?;

    match target {
        OutputTarget::File(path) => {
            let dir = output_dir(&path);

            // Create the output directory if it is missing, matching Asciidoctor,
            // which makes the destination directory (named by `-D`, or embedded
            // in an `-o` path like `build/out.html`) before writing to it.
            fs::create_dir_all(&dir)?;

            // Write any companion stylesheet (`copycss`) into the output file's
            // directory, so a linked stylesheet lands next to the HTML that
            // references it — matching Asciidoctor, which copies only when
            // converting to a file. The guard keeps the copy from clobbering the
            // output file itself when the two paths coincide.
            let mut writer = OutputGuard {
                inner: DirAssetWriter::new(dir),
                output: path.clone(),
            };
            let html =
                asciidoc_html5::convert_document_with_writer(&document, &options, &mut writer)?;
            fs::write(path, html)?;
        }

        // Writing to standard output has no directory to copy alongside, so
        // `copycss` is inert here — again matching Asciidoctor, which skips the
        // copy unless there is an output file.
        OutputTarget::Stdout => {
            let html = asciidoc_html5::convert_document_with(&document, &options);
            stdout.write_all(html.as_bytes())?;
        }
    }

    Ok(failure_reached)
}

/// The document name to attach to a warning's location line: the input file's
/// path, or the conventional `<stdin>` when the document was read from standard
/// input — matching how Asciidoctor labels the source of a diagnostic.
fn warning_document_name(input: Option<&Path>) -> String {
    match input {
        Some(path) => path.display().to_string(),
        None => "<stdin>".to_string(),
    }
}

/// A single resolved source for `adoc` to convert: an on-disk file, or standard
/// input.
#[derive(Debug)]
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
///
/// Standard input is read only when `-` is the *sole* argument. A `-` mixed in
/// with other inputs is an extra argument — Asciidoctor warns about it and
/// ignores it rather than reading standard input a second time (which would
/// yield an empty document at end-of-stream), so `adoc` does the same.
fn resolve_inputs(inputs: &[PathBuf]) -> io::Result<Vec<InputSource>> {
    // No input argument, or a single `-`, reads standard input — the same two
    // spellings the single-file path already treats as stdin.
    if inputs.is_empty() || (inputs.len() == 1 && inputs[0].as_os_str() == "-") {
        return Ok(vec![InputSource::Stdin]);
    }

    let mut sources = Vec::new();
    for arg in inputs {
        if arg.as_os_str() == "-" {
            // A `-` here is not the sole argument, so it is an extra one: warn
            // and skip it, matching Asciidoctor, rather than reading standard
            // input again.
            eprintln!(
                "adoc: ignoring extra '-' argument; standard input is read only \
                 when it is the only input"
            );
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

/// Whether `a` and `b` name the same file on disk.
///
/// When both paths exist, compares their file identity — device and inode — so
/// two entries pointing at the same underlying file are recognized even under
/// different path spellings, including a hard link (which shares an inode but
/// has no common path) and a symlink (whose target is stat'd). Writing through
/// either would replace that shared file's contents, so an output aliasing an
/// input this way must be refused.
///
/// For a path that does not exist yet — a derived output not written, or one
/// reached through a parent-directory symlink — there is no inode to compare,
/// so it falls back to comparing resolved paths (see [`resolve_for_compare`]),
/// and finally to the plain absolute comparison, so the check still works
/// before either file exists.
///
/// # Platform limitation
///
/// The identity comparison is Unix-only: the equivalent Windows file id
/// (`file_index`/`volume_serial_number`) is exposed by std only behind an
/// unstable feature, and the actively-maintained crates that fill the gap are
/// avoided here (see the project's dependency policy). On Windows the check
/// therefore relies on path resolution — which `canonicalize` still uses to
/// resolve symlinks and junctions, so a symlinked output is caught — but a
/// *hard-linked* output (distinct path, no symlink to resolve) is not detected,
/// so writing it could truncate the source. Closing that would take a Win32
/// `GetFileInformationByHandle` call via a fresh, maintained binding; tracked
/// in <https://github.com/asciidoc-rs/asciidoc-html5/issues/169>.
fn same_file(a: &Path, b: &Path) -> bool {
    if let (Ok(ma), Ok(mb)) = (fs::metadata(a), fs::metadata(b)) {
        if same_inode(&ma, &mb) {
            return true;
        }
    }

    if let (Some(a), Some(b)) = (resolve_for_compare(a), resolve_for_compare(b)) {
        return a == b;
    }

    match (std::path::absolute(a), std::path::absolute(b)) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    }
}

/// Whether two metadata handles refer to the same underlying file (same device
/// and inode). On Unix this catches hard links and resolved symlinks; on other
/// platforms std exposes no inode, so [`same_file`] relies on its path
/// comparison instead.
#[cfg(unix)]
fn same_inode(a: &fs::Metadata, b: &fs::Metadata) -> bool {
    use std::os::unix::fs::MetadataExt as _;

    a.dev() == b.dev() && a.ino() == b.ino()
}

#[cfg(not(unix))]
fn same_inode(_a: &fs::Metadata, _b: &fs::Metadata) -> bool {
    false
}

/// Resolves `path` to a canonical identity suitable for comparing whether two
/// paths name the same file, even when the file does not exist yet.
///
/// An existing path is canonicalized outright, following a symlink to its
/// target. A path that does not exist is resolved by canonicalizing its parent
/// directory (so parent-directory symlinks are still accounted for) and
/// re-attaching the file name; a bare file name resolves against the current
/// directory. Returns `None` when neither the path nor its parent can be
/// resolved — an unresolvable output cannot alias an existing input, so callers
/// treat that as "not the same file".
fn resolve_for_compare(path: &Path) -> Option<PathBuf> {
    if let Ok(canonical) = path.canonicalize() {
        return Some(canonical);
    }

    let name = path.file_name()?;
    let parent = path.parent().unwrap_or(Path::new(""));
    let base = if parent.as_os_str().is_empty() {
        std::env::current_dir().ok()?
    } else {
        parent.canonicalize().ok()?
    };
    Some(base.join(name))
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
/// from, so the current directory is used, matching Asciidoctor. That standard-
/// input fallback yields to an explicit `-a docdir=…`, which Asciidoctor treats
/// as a second way to anchor a piped document's relative includes and which the
/// library honors on its own; forcing the current directory here would mask it.
/// In every case the input file (when there is one) is recorded so its
/// top-level `include::` directives resolve against its own directory.
///
/// # Errors
///
/// Returns an [`io::Error`] when the current directory is needed but cannot be
/// determined.
fn apply_base_dir(cli: &Cli, mut options: Options, input: Option<&Path>) -> io::Result<Options> {
    if let Some(dir) = &cli.base_dir {
        options = options.base_dir(dir.clone());
    } else if input.is_none() && !has_explicit_docdir(cli) {
        options = options.base_dir(std::env::current_dir()?);
    }

    if let Some(path) = input {
        options = options.input_file(path.to_path_buf());
    }

    Ok(options)
}

/// Whether the `-a`/`--attribute` specs explicitly assign a `docdir` *value*
/// (`docdir=…`), as opposed to setting, unsetting, or not mentioning it.
///
/// Such a value seeds include resolution inside the library, so the standard-
/// input base-directory fallback in [`apply_base_dir`] must defer to it rather
/// than pinning the base directory to the current directory. The parsing
/// mirrors [`apply_attribute_spec`]: a trailing `@` is a soft-default marker,
/// the key is everything before the first `=`, a leading or trailing `!` on the
/// key makes it an unset (not a value), and attribute names are matched
/// case-insensitively (the library lowercases them).
fn has_explicit_docdir(cli: &Cli) -> bool {
    cli.attribute.iter().any(|spec| {
        let body = spec.strip_suffix('@').unwrap_or(spec);
        match body.split_once('=') {
            Some((key, _)) => {
                !key.starts_with('!') && !key.ends_with('!') && key.eq_ignore_ascii_case("docdir")
            }
            None => false,
        }
    })
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

/// Validates the `-b`/`--backend` selection, accepting only the HTML5 backend.
///
/// `adoc` produces solely the HTML5 backend, so `-b html5` (case-insensitive)
/// and the default (no `-b`) are accepted as a no-op, purely for command-line
/// compatibility with `asciidoctor -b html5 …`. Every other backend Asciidoctor
/// documents — `xhtml5`, `docbook5`, `manpage`, extended converters — is simply
/// not implemented here, so it is rejected rather than silently ignored, and a
/// caller expecting different output finds out immediately.
///
/// # Errors
///
/// Returns an [`io::ErrorKind::InvalidInput`] error naming the unsupported
/// backend.
fn check_backend(cli: &Cli) -> io::Result<()> {
    let Some(name) = &cli.backend else {
        return Ok(());
    };

    match name.to_lowercase().as_str() {
        "html5" => Ok(()),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("unsupported backend '{name}': adoc only produces the html5 backend"),
        )),
    }
}

/// Validates the `-d`/`--doctype` selection, accepting only the article
/// doctype.
///
/// `adoc` models solely the `article` doctype, so `-d article`
/// (case-insensitive) and the default (no `-d`) are accepted as a no-op, purely
/// for command-line compatibility with `asciidoctor -d article …`. The other
/// doctypes Asciidoctor defines — `book`, `manpage`, and `inline` — are not
/// implemented here, so each is rejected rather than silently ignored, and a
/// caller expecting different output finds out immediately.
///
/// # Errors
///
/// Returns an [`io::ErrorKind::InvalidInput`] error naming the unsupported
/// doctype.
fn check_doctype(cli: &Cli) -> io::Result<()> {
    let Some(name) = &cli.doctype else {
        return Ok(());
    };

    match name.to_lowercase().as_str() {
        "article" => Ok(()),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("unsupported doctype '{name}': adoc only produces the article doctype"),
        )),
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

/// A diagnostic's severity, ordered least to most severe, over the subset
/// `adoc` routes: the parser reports at the `Info` and `Warn` levels, and
/// `--failure-level` additionally names the `Error` and `Fatal` thresholds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum LogLevel {
    /// A low-severity, easily-a-false-positive diagnostic Asciidoctor shows
    /// only under `-v` (a missing-attribute reference, an unresolved cross
    /// reference).
    Info,

    /// The level of most parser diagnostics, shown by default.
    Warn,

    /// A threshold above `Warn`; no parse emits at this level, but
    /// `--failure-level=ERROR` names it.
    Error,

    /// The default `--failure-level`; effectively never reached by a parse.
    Fatal,
}

impl LogLevel {
    /// The label Asciidoctor's basic formatter prints for this level (`WARN`
    /// renders as `WARNING`, `FATAL` as `FAILED`).
    fn label(self) -> &'static str {
        match self {
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARNING",
            LogLevel::Error => "ERROR",
            LogLevel::Fatal => "FAILED",
        }
    }
}

/// The severity `adoc` reports a parser warning at.
///
/// Most parser diagnostics are warnings. A few — a reference to a missing
/// attribute, an `include::` dropped because an attribute in its target is
/// unset, and a cross reference whose target is not defined — are the lower,
/// info-level diagnostics Asciidoctor reports only in verbose mode, so `adoc`
/// classifies them the same way (suppressed unless `-v`). Any warning type not
/// named here is a warning; the parser's warning set is `non_exhaustive`, so a
/// newly-recognized condition defaults to `Warn` (printed by default) until it
/// is deliberately reclassified.
fn warning_severity(warning: &WarningType) -> LogLevel {
    match warning {
        WarningType::SkippingReferenceToMissingAttribute(_)
        | WarningType::IncludeDroppedDueToMissingAttribute(_)
        | WarningType::PossibleInvalidReference(_) => LogLevel::Info,
        _ => LogLevel::Warn,
    }
}

/// Gathers the `-q`/`-v`/`--failure-level` options that govern how a parse's
/// warnings are reported, so every source's warnings are routed the same way.
struct WarningReporter {
    /// Whether `-q`/`--quiet` silences all warning output.
    quiet: bool,

    /// The lowest severity to print: `Info` under `-v`/`--verbose`, otherwise
    /// `Warn`.
    display_level: LogLevel,

    /// The lowest severity that raises the exit code, from `--failure-level`.
    failure_level: LogLevel,
}

impl WarningReporter {
    /// Reads the reporting options off the parsed [`Cli`].
    ///
    /// # Errors
    ///
    /// Returns an [`io::ErrorKind::InvalidInput`] error when `--failure-level`
    /// names an unrecognized level.
    fn from_cli(cli: &Cli) -> io::Result<Self> {
        Ok(Self {
            quiet: cli.quiet,
            display_level: if cli.verbose {
                LogLevel::Info
            } else {
                LogLevel::Warn
            },
            failure_level: parse_failure_level(&cli.failure_level)?,
        })
    }

    /// Prints `document`'s warnings — those at or above the display level,
    /// unless silenced by `-q` — to `stderr`, labeling each with `doc_name` and
    /// its origin line. Returns whether any warning (printed or not) reached
    /// the failure level.
    ///
    /// # Errors
    ///
    /// Returns any [`io::Error`] raised while writing to `stderr`.
    fn report(
        &self,
        document: &Document<'_>,
        doc_name: String,
        stderr: &mut dyn Write,
    ) -> io::Result<bool> {
        let mut failure_reached = false;
        for warning in document.warnings() {
            let severity = warning_severity(&warning.warning);

            // The failure code follows every diagnostic's severity, even one `-q`
            // keeps off the screen — matching Asciidoctor, whose quiet logger
            // still records the highest severity it saw.
            if severity >= self.failure_level {
                failure_reached = true;
            }

            if self.quiet || severity < self.display_level {
                continue;
            }

            let (file, line) = warning_location(document, warning, &doc_name);
            writeln!(
                stderr,
                "adoc: {label}: {file}: line {line}: {message}",
                label = severity.label(),
                message = warning.warning,
            )?;
        }
        Ok(failure_reached)
    }
}

/// Resolves the `(file, line)` a warning should be attributed to, matching
/// Asciidoctor's `<path>: line N` diagnostic prefix.
///
/// A warning that carries its own [`origin`](Warning::origin) — one raised
/// inside include-expanded content that never appears in the document source —
/// names its file and line directly. Otherwise the warning's source span is
/// resolved through the document's source map: an `include::`d file reports
/// that file's name, and the root document (which the map leaves unnamed)
/// reports `doc_name` — the input path, or `<stdin>`.
fn warning_location(
    document: &Document<'_>,
    warning: &Warning<'_>,
    doc_name: &str,
) -> (String, usize) {
    if let Some(SourceLine(file, line)) = &warning.origin {
        let file = file.clone().unwrap_or_else(|| doc_name.to_string());
        return (file, *line);
    }

    let origin = document.origin_of(warning.source);
    let file = origin
        .file
        .map(str::to_string)
        .unwrap_or_else(|| doc_name.to_string());

    (file, origin.line)
}

/// Parses a `--failure-level` value (case-insensitive) into the [`LogLevel`]
/// threshold at or above which a diagnostic makes `adoc` exit non-zero.
///
/// Accepts `info`, `warn`/`warning`, `error`, and `fatal`, matching the set
/// Asciidoctor's `--failure-level` takes.
///
/// # Errors
///
/// Returns an [`io::ErrorKind::InvalidInput`] error when `name` is none of
/// those.
fn parse_failure_level(name: &str) -> io::Result<LogLevel> {
    match name.to_lowercase().as_str() {
        "info" => Ok(LogLevel::Info),
        "warn" | "warning" => Ok(LogLevel::Warn),
        "error" => Ok(LogLevel::Error),
        "fatal" => Ok(LogLevel::Fatal),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("invalid failure level '{name}': expected info, warn, error, or fatal"),
        )),
    }
}

/// Builds the conversion [`Options`] from the `-n`/`--section-numbers` flag and
/// the raw `-a`/`--attribute` specs, parsing each spec with
/// [`apply_attribute_spec`].
///
/// `-n` is Asciidoctor's shorthand for `-a sectnums`, so it is seeded as an
/// override *before* the `-a` specs are applied. Because a later directive for
/// the same name wins, an explicit `-a sectnums!` on the same command line
/// still overrides `-n` and leaves headings unnumbered — matching
/// `asciidoctor -n -a sectnums!`.
fn build_options(section_numbers: bool, specs: &[String]) -> io::Result<Options> {
    let mut options = Options::new();
    if section_numbers {
        options = options.set("sectnums");
    }

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
/// name is derived from `input` by replacing its extension with `.html`, so
/// `adoc document.adoc` writes `document.html`, and each file in a multi-file
/// invocation lands in its own `.html`. When the source is standard input
/// (`input` is `None`) there is no name to derive from, so the HTML5 goes to
/// standard output.
///
/// The `-D`/`--destination-dir` directory, when given, holds the output: a
/// relative `-o` path (or a derived name) is placed inside it, while an
/// absolute `-o` path is used unchanged. Without `-D`, an `-o` path is taken as
/// given (a relative one relative to the current directory) and a derived name
/// is written alongside its input. Because the destination applies to whichever
/// `input` this is called for, `-D build` with several inputs writes each
/// derived name into `build`.
///
/// `-R`/`--source-dir` refines the destination per input: when the input file
/// lives under the source root, its subdirectory beneath that root is recreated
/// inside `-D` (see [`destination_dir_for`]), so the derived name (or a
/// relative `-o`) lands in the mirrored subdirectory rather than flat in `-D`.
fn output_target_for(cli: &Cli, input: Option<&Path>) -> OutputTarget {
    let dir = destination_dir_for(cli, input);
    let dir = dir.as_deref();
    match cli.output.as_deref() {
        Some(path) if path.as_os_str() == "-" => OutputTarget::Stdout,
        Some(path) => OutputTarget::File(resolve_in_dir(dir, path)),
        None => match input {
            Some(input) => OutputTarget::File(derive_output_path(input, dir, &output_suffix(cli))),
            None => OutputTarget::Stdout,
        },
    }
}

/// The destination directory to place this `input`'s output in, applying
/// `-D`/`--destination-dir` and `-R`/`--source-dir` together, mirroring
/// Asciidoctor's `-R` behavior.
///
/// Without `-D` there is no destination directory, so the result is `None` and
/// `-R` is inert (matching Asciidoctor, which only consults the source root
/// when a destination directory is set). With `-D` but no `-R`, the destination
/// is the `-D` directory as given.
///
/// With both, and an on-disk `input` located below the `-R` source root, the
/// input's subdirectory relative to that root is appended to the `-D`
/// directory, so `-R src -D out` sends `src/sub/index.adoc` to `out/sub`. An
/// input that is not under the source root (or a document read from standard
/// input, where `input` is `None`) keeps the plain `-D` directory.
fn destination_dir_for(cli: &Cli, input: Option<&Path>) -> Option<PathBuf> {
    let dest = cli.destination_dir.as_deref()?;

    // `-R` only recreates structure when both a source root and an on-disk input
    // are present; otherwise the destination is the `-D` directory unchanged.
    match (cli.source_dir.as_deref(), input) {
        (Some(source_dir), Some(input)) => match relative_subdir(source_dir, input) {
            Some(subdir) => Some(dest.join(subdir)),
            None => Some(dest.to_path_buf()),
        },
        _ => Some(dest.to_path_buf()),
    }
}

/// The directory of `input` relative to `source_dir`, or `None` when `input` is
/// not located below `source_dir`.
///
/// Both paths are normalized to a lexical absolute form (see
/// [`normalize_lexically`]) — matching Asciidoctor's `File.expand_path`, which
/// collapses `.` and `..` purely textually without touching the filesystem —
/// before the input's parent directory is stripped of the source-root prefix.
/// Collapsing `..` first is what keeps an input like `src/../outside/doc.adoc`
/// (which resolves *out* of `src`) from being treated as living under the
/// source root. An input directly inside the source root yields an empty
/// relative path (joining it onto `-D` leaves the directory unchanged); an
/// input outside the root yields `None`.
fn relative_subdir(source_dir: &Path, input: &Path) -> Option<PathBuf> {
    let abs_source = normalize_lexically(source_dir)?;
    let abs_input = normalize_lexically(input)?;
    let abs_input_dir = abs_input.parent()?;
    abs_input_dir
        .strip_prefix(&abs_source)
        .ok()
        .map(Path::to_path_buf)
}

/// Resolves `path` to an absolute form with `.` and `..` collapsed purely
/// lexically, without consulting the filesystem — the same normalization
/// Asciidoctor's `File.expand_path` performs.
///
/// The path is first made absolute (prepending the current directory when it is
/// relative) via [`std::path::absolute`], which drops `.` components but leaves
/// `..` intact; each `..` is then resolved here by popping the preceding normal
/// component, and a `..` at the root is dropped (as `File.expand_path` treats
/// `/..` as `/`). Unlike [`Path::canonicalize`], symlinks are not resolved and
/// the path need not exist, so two spellings compare by their textual structure
/// alone.
///
/// Returns `None` only when the path cannot be made absolute (for example, an
/// empty path with no current directory available).
fn normalize_lexically(path: &Path) -> Option<PathBuf> {
    use std::path::Component;

    let absolute = std::path::absolute(path).ok()?;
    let mut normalized = Vec::new();
    for component in absolute.components() {
        // `std::path::absolute` has already dropped every `.`, so only `..`
        // needs collapsing; any other component is a root, prefix, or name to
        // keep.
        if component == Component::ParentDir {
            // Climb out of the preceding directory, but never past the root (or
            // a Windows prefix), where `..` is a no-op.
            if matches!(normalized.last(), Some(Component::Normal(_))) {
                normalized.pop();
            }
        } else {
            normalized.push(component);
        }
    }
    Some(normalized.iter().collect())
}

/// The file-name suffix `adoc` derives a default output name with, honoring an
/// `-a outfilesuffix=…` override the way Asciidoctor does (its derived output
/// name uses the `outfilesuffix` attribute). Defaults to `.html`; when the same
/// attribute is assigned more than once, the last value wins.
///
/// Only a value assignment (`outfilesuffix=.htm`) changes the suffix; a bare
/// set or an unset leaves the default, since neither yields a usable extension.
/// The parsing mirrors [`has_explicit_docdir`]: a trailing `@` soft-default
/// marker is stripped, the key is everything before the first `=`, a leading or
/// trailing `!` marks an unset, and the name is matched case-insensitively.
fn output_suffix(cli: &Cli) -> String {
    let mut suffix = String::from(".html");
    for spec in &cli.attribute {
        let body = spec.strip_suffix('@').unwrap_or(spec);
        if let Some((key, value)) = body.split_once('=') {
            if !key.starts_with('!')
                && !key.ends_with('!')
                && key.eq_ignore_ascii_case("outfilesuffix")
            {
                suffix = value.to_string();
            }
        }
    }
    suffix
}

/// Resolves an explicit `-o` output `file` against the `-D` destination `dir`:
/// a relative file lands inside the directory, while an absolute file (or the
/// absence of `-D`) is taken unchanged. So a bare relative `-o` with no `-D`
/// stays relative to the current working directory.
fn resolve_in_dir(dir: Option<&Path>, file: &Path) -> PathBuf {
    match dir {
        Some(dir) if file.is_relative() => dir.join(file),
        _ => file.to_path_buf(),
    }
}

/// Derives the default output path for `input` by swapping its extension for
/// `suffix` (`.html` by default, or an `-a outfilesuffix=…` override — see
/// [`output_suffix`]), matching how `asciidoctor` names its output file. With a
/// `-D` destination `dir`, the derived name is placed in that directory;
/// otherwise it is written alongside the input.
fn derive_output_path(input: &Path, dir: Option<&Path>, suffix: &str) -> PathBuf {
    // `outfilesuffix` carries the leading dot (`.htm`), while `with_extension`
    // wants the extension without it.
    let extension = suffix.strip_prefix('.').unwrap_or(suffix);
    let derived = input.with_extension(extension);
    match (dir, derived.file_name()) {
        (Some(dir), Some(name)) => dir.join(name),
        _ => derived,
    }
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
