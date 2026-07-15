use std::path::PathBuf;

use asciidoc_html5::{convert_with, Options, SafeMode};
use clap::Parser as _;

use crate::{input_file, output_target, run, run_with_input, tests::sdd::*, Cli, OutputTarget};

track_file!("docs/modules/cli/pages/io-piping.adoc");

// This crate's "Pipe Content Through the CLI" page. It documents how `adoc`
// pipes: `-` (or no input file) reads standard input, output goes to standard
// output by default, `-o` names a file (`-o -` names standard output), and both
// `-B` and an explicit `-a docdir=…` supply the base directory a piped
// document's relative `include::` targets resolve against (with `-B` winning
// when both are given). Each invocation is verified through `adoc`'s own option
// parsing (`Cli` plus the private `input_file`/`output_target` routing) and,
// for conversion and include resolution, end to end. Its `-e`/`--embedded`
// embeddable-output section is verified end to end.

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
        "adoc-docs-io-piping-{label}-{}.adoc",
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
:navtitle: Pipe Content
:description: How to pipe AsciiDoc into the adoc command and its HTML5 back out through standard input and output.

The `adoc` command can read AsciiDoc from standard input (STDIN) and write the
rendered HTML5 to standard output (STDOUT), so it can sit in the middle of a
shell pipeline. This is called piping.

[NOTE]
====
The prose on this page is non-normative documentation. The `adoc` invocations it
shows are normative: they are verified against the implementation, so the
documented behavior is guaranteed.
====

"#
);

// `-` (and, equivalently, an omitted input file) reads the source from standard
// input, and — with no output file to derive a name from — `adoc` writes to
// standard output by default, so `adoc -` is the same as `adoc -o - -`.
#[test]
fn reads_stdin_and_writes_stdout_by_default() {
    verifies!(
        r#"
== Read from standard input

Pass `-` as the input file to read the source from standard input:

 $ echo 'content' | adoc -

`adoc` also reads standard input when you give no input file at all, so both
spellings pipe. Because piped input has no file name to derive an output name
from, `adoc` writes the converted HTML to standard output by default. That makes
the command above the same as naming standard output explicitly with `-o -`:

 $ echo 'content' | adoc -o - -

"#
    );

    // Both spellings of standard input read stdin and write stdout, and both
    // convert the same piped source end to end.
    for args in [&["adoc", "-"][..], &["adoc"][..]] {
        assert!(reads_stdin(args));
        assert!(goes_to_stdout(args));

        let html = run_piped(args, "= Doc\n\nBody.");
        assert!(html.starts_with("<!DOCTYPE html>"));
        assert!(html.contains("<p>Body.</p>"));
    }

    // Naming standard output explicitly with `-o -` routes the same way, and
    // piping to it produces byte-for-byte the same document as the bare `-`.
    assert!(goes_to_stdout(&["adoc", "-o", "-", "-"]));
    assert_eq!(
        run_piped(&["adoc", "-"], "= Doc\n\nBody."),
        run_piped(&["adoc", "-o", "-", "-"], "= Doc\n\nBody.")
    );
}

// `-o` names an output file, capturing the full standalone document there
// instead of on standard output.
#[test]
fn output_flag_writes_a_file() {
    verifies!(
        r#"
== Write to a file

To capture the full standalone HTML document in a file instead of standard
output, name it with `-o`:

 $ echo 'content' | adoc -o output.html -

"#
    );

    assert_eq!(
        output_file(&["adoc", "-o", "output.html", "-"]),
        Some(PathBuf::from("output.html"))
    );

    // End to end, `-o <file>` writes the standalone HTML to the file, not stdout.
    let out = std::env::temp_dir().join(format!(
        "adoc-docs-io-piping-outfile-{}.html",
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

// Piped input has no location on disk, so an absolute base directory given with
// `-B` is what a relative `include::` resolves against.
#[test]
fn base_dir_resolves_piped_includes() {
    verifies!(
        r#"
== Resolve includes when piping

Piped input has no location on disk, so relative `include::` targets have nothing
to resolve against. Give an absolute base directory with `-B` so they resolve
against it:

 $ echo 'content' | adoc -B /path/to/basedir -o output.html -

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

    // The stdout branch of `run` converts piped source with
    // `convert_with(&source, &options)`, where `-B` sets `options.base_dir`.
    // Drive that same path: a piped document resolves a relative include sitting
    // inside the base directory.
    let dir = std::env::temp_dir().join(format!(
        "adoc-docs-io-piping-basedir-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create base directory");
    std::fs::write(dir.join("part.adoc"), "Included body text.\n").expect("write include");

    let source = "= Doc\n\ninclude::part.adoc[]\n";
    let html = convert_with(
        source,
        &Options::new().safe_mode(SafeMode::Safe).base_dir(&dir),
    );
    let _ = std::fs::remove_dir_all(&dir);
    assert!(html.contains("Included body text."));
}

// `adoc` also honors `-a docdir=…` as a second way to anchor a piped document's
// relative includes, matching Asciidoctor: an explicit `docdir` seeds the base
// directory when `-B` is absent, and `-B` wins when both are given. This closes
// the divergence tracked in
// https://github.com/asciidoc-rs/asciidoc-html5/issues/73.
#[test]
fn docdir_resolves_piped_includes_and_yields_to_base_dir() {
    verifies!(
        r#"
Alternately, set the `docdir` attribute to an absolute path. `adoc` treats it as
an artificial document directory that piped includes resolve against, just as `-B`
does — matching Asciidoctor, which offers the same two routes:

 $ echo 'content' | adoc -a docdir=/path/to/docdir -o output.html -

When you pass both, `-B` wins: it sets the base directory, and an explicit
`docdir` only seeds the base directory when `-B` is absent.

"#
    );

    // Pointing `-a docdir=…` at a directory that holds the include resolves it,
    // driven end to end through the CLI's stdin path — the same route that was
    // previously anchored to the current directory, ignoring `docdir`.
    let docdir =
        std::env::temp_dir().join(format!("adoc-docs-io-piping-docdir-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&docdir);
    std::fs::create_dir_all(&docdir).expect("create docdir");
    std::fs::write(docdir.join("part.adoc"), "Body via docdir.\n").expect("write include");
    let docdir_str = docdir.to_str().expect("docdir path is UTF-8");

    let source = "= Doc\n\ninclude::part.adoc[]\n";
    let html = run_piped(
        &[
            "adoc",
            "-S",
            "safe",
            "-a",
            &format!("docdir={docdir_str}"),
            "-",
        ],
        source,
    );
    assert!(html.contains("Body via docdir."), "{html}");

    // When both are given, `-B` wins: with `-B` pointed at a directory holding a
    // different include, that copy resolves and the `docdir` copy does not.
    let base = std::env::temp_dir().join(format!(
        "adoc-docs-io-piping-docdir-base-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).expect("create base directory");
    std::fs::write(base.join("part.adoc"), "Body via base dir.\n").expect("write include");
    let base_str = base.to_str().expect("base path is UTF-8");

    let html = run_piped(
        &[
            "adoc",
            "-S",
            "safe",
            "-B",
            base_str,
            "-a",
            &format!("docdir={docdir_str}"),
            "-",
        ],
        source,
    );
    let _ = std::fs::remove_dir_all(&docdir);
    let _ = std::fs::remove_dir_all(&base);
    assert!(html.contains("Body via base dir."), "{html}");
    assert!(!html.contains("Body via docdir."), "{html}");
}

// The `-e`/`--embedded` embeddable-output mode: `adoc -e` writes just the
// converted body, and `-a showtitle` adds the doctitle back as a leading
// `<h1>`; without `-e`, output is a standalone document.
#[test]
fn the_embedded_flag_produces_body_only_output() {
    verifies!(
        r#"
== Embeddable output

When piping through `adoc`, you often just want the converted body -- embeddable
HTML to drop into a surrounding template -- rather than a standalone document. Add
the `-e` flag (short for `--embedded`) to produce that:

 $ echo 'content' | adoc -e -

Embeddable output omits the doctitle by default. To include it as a leading
`<h1>`, set the `showtitle` attribute:

 $ printf '= Document Title\n\ncontent\n' | adoc -e -a showtitle -

Without `-e`, `adoc` writes a standalone HTML5 document, matching Asciidoctor's
command, which is standalone even when piping.

"#
    );

    // `-e` yields the converted body only — no standalone shell, and no doctitle
    // `<h1>` unless it is asked for.
    let body = run_piped(&["adoc", "-e", "-"], "= Document Title\n\ncontent");
    assert!(!body.starts_with("<!DOCTYPE html>"));
    assert!(body.contains("<p>content</p>"));
    assert!(!body.contains("<h1>"));

    // `-a showtitle` includes the doctitle as a leading `<h1>`.
    let with_title = run_piped(
        &["adoc", "-e", "-a", "showtitle", "-"],
        "= Document Title\n\ncontent",
    );
    assert!(with_title.contains("<h1>Document Title</h1>"));

    // Without `-e`, the output is a standalone document.
    let standalone = run_piped(&["adoc", "-"], "= Document Title\n\ncontent");
    assert!(standalone.starts_with("<!DOCTYPE html>"));
}

non_normative!(
    r#"
You can also set the xref:cli:set-safe-mode.adoc[safe mode from the CLI], which
governs how far a piped document may reach when resolving includes.
"#
);
