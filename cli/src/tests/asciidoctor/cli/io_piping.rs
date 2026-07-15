use std::path::PathBuf;

use asciidoc_html5::{convert_with, Options, SafeMode};
use clap::Parser as _;

use crate::{input_file, output_target, run, run_with_input, tests::sdd::*, Cli, OutputTarget};

track_file!("ref/asciidoctor/docs/modules/cli/pages/io-piping.adoc");

// Asciidoctor's "Pipe Content Through the CLI" page, tracked from the CLI
// crate. `adoc` mirrors the piping half of this interface: `-` (or no input
// file) reads the source from standard input, output goes to standard output by
// default, `-o` names an output file (`-o -` names standard output explicitly),
// and `-B` supplies the base directory that a piped document's relative
// `include::` targets resolve against. Each invocation drives `adoc`'s own
// option parsing (`Cli` plus the private `input_file`/`output_target` routing),
// and the conversion and base-directory behaviors are confirmed end to end.
//
// The `docdir` attribute is honored too: Asciidoctor offers it as a second way
// to fix include resolution when piping, and `adoc` matches — an explicit
// `-a docdir=…` seeds the include base directory just as `-B` does (with `-B`
// winning when both are given), so that passage is now verified below. The
// `-e`/`--embedded` embeddable-output mode is supported and is verified below.

/// Whether `adoc` would read this invocation's source from standard input
/// rather than from a named input file.
fn reads_stdin(args: &[&str]) -> bool {
    input_file(&Cli::parse_from(args)).is_none()
}

/// Whether `adoc` would send this invocation's rendered HTML to standard
/// output.
fn goes_to_stdout(args: &[&str]) -> bool {
    matches!(output_target(&Cli::parse_from(args)), OutputTarget::Stdout)
}

/// The output file `adoc` would write this invocation's HTML to, or `None` when
/// it writes to standard output.
fn output_file(args: &[&str]) -> Option<PathBuf> {
    match output_target(&Cli::parse_from(args)) {
        OutputTarget::File(path) => Some(path),
        OutputTarget::Stdout => None,
    }
}

/// Writes `source` to a temp `.adoc` file, runs `adoc` with `args` followed by
/// that file, and returns the captured stdout bytes together with the input
/// path (so the caller can inspect any `-o` output file and clean up).
fn run_adoc(label: &str, args: &[&str], source: &str) -> (Vec<u8>, PathBuf) {
    let path = std::env::temp_dir().join(format!(
        "adoc-cli-io-piping-{label}-{}.adoc",
        std::process::id()
    ));
    std::fs::write(&path, source).expect("write temp input");

    let mut full: Vec<&str> = vec!["adoc"];
    full.extend_from_slice(args);
    let path_str = path.to_str().expect("temp path is UTF-8");
    full.push(path_str);

    let cli = Cli::parse_from(full);
    let mut stdout = Vec::new();
    run(&cli, &mut stdout).expect("adoc converts");
    (stdout, path)
}

/// Pipes `source` through `adoc`: builds a `Cli` from `args` (which select
/// standard input — an explicit `-` or an omitted input file), feeds `source`
/// in as standard input, and returns the captured stdout. This drives the real
/// stdin read path via [`run_with_input`], the injectable-reader core of `run`.
fn run_piped(args: &[&str], source: &str) -> String {
    let cli = Cli::parse_from(args);
    let mut stdin = source.as_bytes();
    let mut stdout = Vec::new();
    run_with_input(&cli, &mut stdin, &mut stdout).expect("adoc converts");
    String::from_utf8(stdout).expect("adoc output is UTF-8")
}

non_normative!(
    r#"
= Pipe Content Through the CLI

In addition to converting files, the Asciidoctor CLI can read content from standard input (STDIN) and/or write content to standard output (STDOUT).
This feature is called piping.

"#
);

// The `-` flag tells `adoc` to read the source from standard input, exactly as
// it tells Asciidoctor to. Parsed this way, `adoc` reads from stdin (there is
// no input file), and — with no `-o` — it writes the result to stdout, so `adoc
// -` sits at the end of a pipeline.
#[test]
fn the_dash_flag_reads_from_stdin() {
    verifies!(
        r#"
Using the `-` flag, you can pipe content to the `asciidoctor` command.
This flag tells Asciidoctor read the source from standard input (STDIN).
For example:

 $ echo 'content' | asciidoctor -

"#
    );

    assert!(reads_stdin(&["adoc", "-"]));
    assert!(goes_to_stdout(&["adoc", "-"]));

    // End to end, piping content through `-` reads the source from stdin and
    // writes the converted HTML to stdout.
    let html = run_piped(&["adoc", "-"], "= Doc\n\nBody.");
    assert!(html.starts_with("<!DOCTYPE html>"));
    assert!(html.contains("<p>Body.</p>"));
}

// "Any variation of STDIN will work": for `adoc`, the two spellings that select
// standard input — an explicit `-` and an omitted input file — are equivalent,
// each reading from stdin and writing to stdout.
#[test]
fn any_variation_of_stdin_works() {
    verifies!(
        r#"
NOTE: Any variation of STDIN will work.

"#
    );

    for args in [&["adoc", "-"][..], &["adoc"][..]] {
        assert!(reads_stdin(args));
        assert!(goes_to_stdout(args));

        // Both spellings convert the same piped source end to end.
        let html = run_piped(args, "= Doc\n\nBody.");
        assert!(html.contains("<p>Body.</p>"));
    }
}

// Reading from STDIN, `adoc` has no input file to derive an output name from,
// so it writes to STDOUT by default — making `adoc -` the same as spelling out
// the destination with `adoc -o - -`. End to end, converting through the stdout
// path yields a standalone HTML5 document.
#[test]
fn stdin_sends_output_to_stdout_by_default() {
    verifies!(
        r#"
This command is effectively the same as:

 $ echo 'content' | asciidoctor -o - -

When reading source from STDIN, Asciidoctor doesn't have a reference to an input file.
Therefore, it sends the converted text to standard output (STDOUT) by default.

"#
    );

    // The bare `-` and the explicit `-o - -` route identically: both to stdout.
    assert!(goes_to_stdout(&["adoc", "-"]));
    assert!(goes_to_stdout(&["adoc", "-o", "-", "-"]));

    // End to end, piping to `-` reads stdin and writes the converted HTML to
    // stdout — byte-for-byte the same as spelling out the destination `-o - -`.
    let source = "= Doc\n\nBody.";
    let bare = run_piped(&["adoc", "-"], source);
    let explicit = run_piped(&["adoc", "-o", "-", "-"], source);
    assert!(bare.starts_with("<!DOCTYPE html>"));
    assert!(bare.contains("<p>Body.</p>"));
    assert_eq!(bare, explicit);
}

// The `-o` flag redirects the full standalone document to a file instead of
// standard output. `adoc` routes `-o output.html` to that file, and end to end
// the HTML lands in the file while stdout stays empty.
#[test]
fn the_output_flag_writes_to_a_file() {
    verifies!(
        r#"
If, instead, you want to write the full document to an output file, you specify it using the `-o` flag.
For example, the following command writes a standalone HTML document to [.path]_output.html_ instead of STDOUT:

 $ echo 'content' | asciidoctor -o output.html -

"#
    );

    assert_eq!(
        output_file(&["adoc", "-o", "output.html", "-"]),
        Some(PathBuf::from("output.html"))
    );

    // End to end, `-o <file>` writes the standalone HTML to the file, not stdout.
    let out = std::env::temp_dir().join(format!(
        "adoc-cli-io-piping-outfile-{}.html",
        std::process::id()
    ));
    let out_str = out.to_str().expect("output path is UTF-8");
    let (stdout, input) = run_adoc("outfile", &["-o", out_str], "= Doc\n\nBody.");
    assert!(
        stdout.is_empty(),
        "adoc wrote to stdout instead of the file"
    );
    let html = std::fs::read_to_string(&out).expect("read output file");
    let _ = std::fs::remove_file(&input);
    let _ = std::fs::remove_file(&out);
    assert!(html.starts_with("<!DOCTYPE html>"));
    assert!(html.contains("<p>Body.</p>"));
}

// Piped input has no location on disk, so relative `include::` targets have
// nothing to resolve against. Supplying an absolute base directory with `-B`
// gives them one. `adoc` parses `-B` into `base_dir`, and, because the stdin
// path hands the read source to `convert_with` with that base directory, a
// relative include then resolves against it.
#[test]
fn the_base_dir_option_resolves_piped_includes() {
    verifies!(
        r#"
When you pipe content to the `asciidoctor` command, it no longer has a concept of where the document is located.
Therefore, relative references such as includes may not work as expected.
To resolve this problem, you should specify an absolute base directory using the `-B` option:

 $ echo 'content' | asciidoctor -B /path/to/basedir -o output.html -

"#
    );

    // `-B` parses into the base directory, in either spelling.
    assert_eq!(
        Cli::parse_from(["adoc", "-B", "/path/to/basedir", "-o", "-", "-"]).base_dir,
        Some(PathBuf::from("/path/to/basedir"))
    );
    assert_eq!(
        Cli::parse_from(["adoc", "--base-dir=/path/to/basedir", "-o", "-", "-"]).base_dir,
        Some(PathBuf::from("/path/to/basedir"))
    );

    // The stdout branch of `run` converts the piped source with
    // `convert_with(&source, &options)`, where `-B` sets `options.base_dir`.
    // Drive that same path: a piped document whose only anchor is the base
    // directory resolves a relative include sitting inside it.
    let dir =
        std::env::temp_dir().join(format!("adoc-cli-io-piping-basedir-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create base directory");
    std::fs::write(dir.join("part.adoc"), "Included body text.\n").expect("write include");

    let source = "= Doc\n\ninclude::part.adoc[]\n";
    let options = Options::new().safe_mode(SafeMode::Safe).base_dir(&dir);
    let html = convert_with(source, &options);
    let _ = std::fs::remove_dir_all(&dir);
    assert!(html.contains("Included body text."));
}

// The `docdir` alternative: `adoc` matches Asciidoctor here, treating an
// explicit `-a docdir=/abs/dir` as a second way to anchor a piped document's
// relative includes — equivalent to `-B` when it stands alone. Setting the
// attribute to a directory that holds the include resolves it end to end
// through the CLI's stdin path. Closes
// https://github.com/asciidoc-rs/asciidoc-html5/issues/73.
#[test]
fn the_docdir_attribute_resolves_piped_includes() {
    verifies!(
        r#"
Alternately, you can set an artificial document directory by passing an absolute path to the `docdir` attribute:

 $ echo 'content' | asciidoctor -a docdir=/path/to/docdir -o output.html -

Try both approaches to determine which one suits your needs better.

"#
    );

    let dir =
        std::env::temp_dir().join(format!("adoc-cli-io-piping-docdir-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create docdir");
    std::fs::write(dir.join("part.adoc"), "Included via docdir.\n").expect("write include");
    let dir_str = dir.to_str().expect("docdir path is UTF-8");

    // Drive the real stdin path: `-a docdir=<dir>` seeds the base directory, so a
    // relative include sitting inside it resolves — the same outcome `-B <dir>`
    // produces above.
    let html = run_piped(
        &[
            "adoc",
            "-S",
            "safe",
            "-a",
            &format!("docdir={dir_str}"),
            "-",
        ],
        "= Doc\n\ninclude::part.adoc[]\n",
    );
    let _ = std::fs::remove_dir_all(&dir);
    assert!(html.contains("Included via docdir."), "{html}");
}

// The `-e`/`--embedded` embeddable-output mode: `adoc -e` writes just the
// converted body, and `-a showtitle` adds the doctitle back as a leading
// `<h1>`.
#[test]
fn the_embedded_flag_produces_body_only_output() {
    verifies!(
        r#"
When piping source from STDIN to STDOUT through the `asciidoctor` command, you often just want the converted body (i.e., embeddable HTML).
To produce that variant, add the `-e` flag, short for `--embedded` (previously the `-s` flag):

 $ echo 'content' | asciidoctor -e -

Or perhaps you want to include the doctitle as well:

 $ echo -e '= Document Title\n\ncontent' | asciidoctor -e -a showtitle -
"#
    );

    // `-e` yields the converted body only — no standalone document shell, and no
    // doctitle `<h1>` unless it is asked for.
    let body = run_piped(&["adoc", "-e", "-"], "= Document Title\n\ncontent");
    assert!(!body.starts_with("<!DOCTYPE html>"));
    assert!(body.contains("<p>content</p>"));
    assert!(!body.contains("<h1>"));

    // Adding `-a showtitle` includes the doctitle as a leading `<h1>`, still
    // without the standalone shell.
    let with_title = run_piped(
        &["adoc", "-e", "-a", "showtitle", "-"],
        "= Document Title\n\ncontent",
    );
    assert!(with_title.contains("<h1>Document Title</h1>"));
    assert!(!with_title.starts_with("<!DOCTYPE html>"));
}
