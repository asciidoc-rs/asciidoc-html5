use std::path::PathBuf;

use asciidoc_html5::{convert_with, Options, SafeMode};
use clap::Parser as _;

use crate::{input_file, output_target, run, tests::sdd::*, Cli, OutputTarget};

track_file!("docs/modules/cli/pages/io-piping.adoc");

// This crate's "Pipe Content Through the CLI" page. It documents how `adoc`
// pipes: `-` (or no input file) reads standard input, output goes to standard
// output by default, `-o` names a file (`-o -` names standard output), and `-B`
// supplies the base directory a piped document's relative `include::` targets
// resolve against. Each invocation is verified through `adoc`'s own option
// parsing (`Cli` plus the private `input_file`/`output_target` routing) and,
// for conversion and include resolution, end to end.
//
// The page also documents two `adoc`-specific facts about the Asciidoctor
// features it diverges from: `-a docdir=…` sets and surfaces the `docdir`
// attribute but does not redirect include resolution (only `-B` does), and
// there is no `-e`/`--embedded` embeddable-output mode yet. The first is
// verified; the second is a known limitation with nothing to verify.

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

    // Both spellings of standard input read stdin and write stdout.
    for args in [&["adoc", "-"][..], &["adoc"][..]] {
        assert!(reads_stdin(args));
        assert!(goes_to_stdout(args));
    }

    // Naming standard output explicitly with `-o -` routes the same way.
    assert!(goes_to_stdout(&["adoc", "-o", "-", "-"]));

    // End to end, `-o -` writes the converted HTML to the captured stdout.
    let (stdout, input) = run_adoc("stdout", &["-o", "-"], "= Doc\n\nBody.");
    let _ = std::fs::remove_file(&input);
    let html = String::from_utf8(stdout).expect("adoc output is UTF-8");
    assert!(html.starts_with("<!DOCTYPE html>"));
    assert!(html.contains("<p>Body.</p>"));
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

// `adoc` accepts `-a docdir=…` and surfaces the attribute to the document, but
// — unlike Asciidoctor — the `docdir` value does not redirect include
// resolution; only the base directory (`-B`) does. This divergence is tracked
// in https://github.com/asciidoc-rs/asciidoc-html5/issues/73.
#[test]
fn docdir_is_surfaced_but_does_not_redirect_includes() {
    verifies!(
        r#"
[NOTE]
====
Asciidoctor also lets you set an artificial `docdir` attribute to fix include
resolution when piping. `adoc` accepts `-a docdir=…` and surfaces the attribute
to the document, but it does not redirect include resolution — use `-B` for that.
====

"#
    );

    // `-a docdir=…` sets the attribute, and the document sees the value.
    let (stdout, input) = run_adoc(
        "docdir",
        &["-a", "docdir=/artificial/dir", "-o", "-"],
        "= Doc\n\ndir={docdir}\n",
    );
    let _ = std::fs::remove_file(&input);
    let html = String::from_utf8(stdout).expect("adoc output is UTF-8");
    assert!(html.contains("dir=/artificial/dir"));

    // But `docdir` does not redirect includes: with `docdir` pointed at a
    // directory that holds the include, yet a base directory that does not, the
    // relative include stays unresolved. (Pointing `-B` there would resolve it,
    // as verified above.)
    let target = std::env::temp_dir().join(format!(
        "adoc-docs-io-piping-docdir-target-{}",
        std::process::id()
    ));
    let base = std::env::temp_dir().join(format!(
        "adoc-docs-io-piping-docdir-base-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&target);
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&target).expect("create docdir target");
    std::fs::create_dir_all(&base).expect("create base directory");
    std::fs::write(target.join("part.adoc"), "Body via docdir.\n").expect("write include");

    let source = "= Doc\n\ninclude::part.adoc[]\n";
    let options = Options::new()
        .safe_mode(SafeMode::Safe)
        .base_dir(&base)
        .attribute("docdir", target.to_str().expect("docdir path is UTF-8"));
    let html = convert_with(source, &options);
    let _ = std::fs::remove_dir_all(&target);
    let _ = std::fs::remove_dir_all(&base);
    assert!(!html.contains("Body via docdir."));
}

// The `-e`/`--embedded` embeddable-output mode is a known limitation: `adoc`
// always writes a standalone document, so this closing section — and the
// cross-reference after it — carry no rule to verify. Tracked in
// https://github.com/asciidoc-rs/asciidoc-html5/issues/71.
non_normative!(
    r#"
== Known limitation: no embeddable-output mode

Asciidoctor's `-e`/`--embedded` flag produces just the converted body (embeddable
HTML) rather than a standalone document. `adoc` does not support this mode yet; it
always writes a standalone HTML5 document. This is a known limitation to be lifted
in a future release.

You can also set the xref:cli:set-safe-mode.adoc[safe mode from the CLI], which
governs how far a piped document may reach when resolving includes.
"#
);
