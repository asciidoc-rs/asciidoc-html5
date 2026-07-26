use clap::Parser as _;

use crate::{check_backend, check_doctype, run_with_input, tests::sdd::*, Cli};

track_file!("docs/modules/cli/pages/options.adoc");

/// Converts a two-section document (with no `sectnums` in its own header) under
/// `args` and returns the rendered HTML, so the `-n` behavior can be driven end
/// to end through the CLI pipeline.
fn convert(args: &[&str]) -> String {
    let cli = Cli::parse_from(args);
    let mut stdin = &b"= Doc\n\n== One\n"[..];
    let mut stdout = Vec::new();
    run_with_input(&cli, &mut stdin, &mut stdout).expect("conversion succeeds");
    String::from_utf8(stdout).expect("output is UTF-8")
}

// This crate's "CLI Options" page. It is a landing page for the CLI module:
// descriptive prose, cross-references to the task-specific option pages, and
// the `adoc --help` invocation that lists the supported options. The prose and
// cross-references carry no rule to verify; the `--help` invocation is driven
// by the test below.

non_normative!(
    r#"
= CLI Options
:navtitle: CLI Options
:description: The command line options the adoc command accepts, and where each one is described.

The `adoc` command accepts a set of options that control how it reads input and
writes output. To see the full list of the options it supports, print the usage
statement:

"#
);

// The `adoc --help` invocation prints the usage statement, which lists the
// options `adoc` supports; the short `-h` flag prints a shorter summary.
#[test]
fn help_lists_the_options() {
    verifies!(
        r#"
 $ adoc --help

You can shorten the `--help` flag to `-h`.

"#
    );

    non_normative!(
        r#"
[NOTE]
====
The prose on this page is non-normative documentation. The `adoc` invocations it
shows are normative: they are verified against the implementation, so the
documented behavior is guaranteed.
====

"#
    );

    // clap surfaces a help request as a `DisplayHelp` "error" carrying the
    // rendered help. Both the long `--help` and the short `-h` include the usage
    // statement and the `Options:` section that lists the supported options, and
    // `-h` is the shorter of the two.
    let long = Cli::try_parse_from(["adoc", "--help"]).expect_err("--help displays help");
    assert_eq!(long.kind(), clap::error::ErrorKind::DisplayHelp);
    assert!(long.to_string().contains("Usage: adoc"));
    assert!(long.to_string().contains("Options:"));

    let short = Cli::try_parse_from(["adoc", "-h"]).expect_err("-h displays help");
    assert_eq!(short.kind(), clap::error::ErrorKind::DisplayHelp);
    assert!(short.to_string().contains("Usage: adoc"));
    assert!(
        short.to_string().len() < long.to_string().len(),
        "-h summary should be shorter than --help"
    );
}

// The cross-references to the task-specific option pages and the `-a` pointer
// carry no rule to verify here.
non_normative!(
    r#"
Each option is described in depth on the page for the task it serves:

* xref:output-file.adoc[Specify an Output File] covers `-o` (`--output`) to name
the output file, `-D` (`--destination-dir`) to set the output directory, and `-R`
(`--source-dir`) to recreate the input's subdirectories under it.
* xref:set-safe-mode.adoc[Set Safe Mode] covers `-S` (`--safe-mode`), `--safe`,
and `-B` (`--base-dir`).
* xref:io-piping.adoc[Pipe Content] covers reading from standard input and
writing the result to standard output.
* xref:process-multiple-files.adoc[Process Multiple Files] covers converting more
than one file in a single invocation.
* xref:control-warnings.adoc[Control Warnings] covers `-q` (`--quiet`), `-v`
(`--verbose`), `-w` (`--warnings`), and `--failure-level`.

Setting document attributes from the command line with `-a` (`--attribute`) is
shown in xref:index.adoc[Process AsciiDoc Using the CLI].

"#
);

// `-b` (`--backend`) is accepted for `asciidoctor` compatibility: `html5` (the
// only backend `adoc` produces) is a no-op, and any other backend fails rather
// than being silently ignored. Driven through `check_backend`, the gate `adoc`
// applies to the parsed `--backend` value.
#[test]
fn the_backend_option_accepts_only_html5() {
    verifies!(
        r#"
For compatibility with existing `asciidoctor` command lines, `adoc` also accepts
`-b` (`--backend`). Because it produces only the HTML5 backend, the sole accepted
value is `html5` — like `asciidoctor -b html5`, it is a no-op — while any other
backend, such as `xhtml5` or `docbook5`, exits with an error instead of being
silently ignored.

"#
    );

    // `-b html5` is accepted as a no-op, as is omitting the option entirely.
    check_backend(&Cli::parse_from(["adoc", "-b", "html5", "doc.adoc"]))
        .expect("-b html5 is accepted");
    check_backend(&Cli::parse_from(["adoc", "doc.adoc"])).expect("the default is accepted");

    // Any other backend exits with an error rather than being ignored.
    for backend in ["xhtml5", "docbook5"] {
        assert!(
            check_backend(&Cli::parse_from(["adoc", "-b", backend, "doc.adoc"])).is_err(),
            "-b {backend} is rejected"
        );
    }
}

// `-d` (`--doctype`) is accepted for `asciidoctor` compatibility: `article`
// (the only doctype `adoc` models) is a no-op, and every other doctype fails
// rather than being silently ignored. Driven through `check_doctype`, the gate
// `adoc` applies to the parsed `--doctype` value.
#[test]
fn the_doctype_option_accepts_only_article() {
    verifies!(
        r#"
Likewise, `adoc` accepts `-d` (`--doctype`). Because it models only the `article`
doctype, the sole accepted value is `article` — like `asciidoctor -d article`, it
is a no-op — while the other doctypes Asciidoctor defines, `book`, `manpage`, and
`inline`, exit with an error instead of being silently ignored.

"#
    );

    // `-d article` is accepted as a no-op, as is omitting the option entirely.
    check_doctype(&Cli::parse_from(["adoc", "-d", "article", "doc.adoc"]))
        .expect("-d article is accepted");
    check_doctype(&Cli::parse_from(["adoc", "doc.adoc"])).expect("the default is accepted");

    // Every other doctype exits with an error rather than being ignored.
    for doctype in ["book", "manpage", "inline"] {
        assert!(
            check_doctype(&Cli::parse_from(["adoc", "-d", doctype, "doc.adoc"])).is_err(),
            "-d {doctype} is rejected"
        );
    }
}

// `-n` (`--section-numbers`) sets the `sectnums` attribute, numbering section
// headings; an explicit `-a sectnums!` on the same command line overrides it.
// Both are driven end to end through the CLI's conversion pipeline.
#[test]
fn the_section_numbers_option_numbers_headings() {
    verifies!(
        r#"
To auto-number section titles, pass `-n` (`--section-numbers`), the same shorthand
`asciidoctor -n` provides for setting the `sectnums` document attribute. With it,
each section heading is prefixed with its dotted section number (`1.`, `1.1.`, …);
without it, headings are unnumbered. It is seeded before any `-a` option, so an
explicit `-a sectnums!` on the same command line still leaves the headings
unnumbered.

"#
    );

    // With `-n`, the lone section heading is numbered `1.`.
    assert!(
        convert(&["adoc", "-n", "-o", "-"]).contains(r#"<h2 id="_one">1. One</h2>"#),
        "-n numbers the heading"
    );

    // Without it, the heading is unnumbered.
    assert!(
        convert(&["adoc", "-o", "-"]).contains(r#"<h2 id="_one">One</h2>"#),
        "the heading is unnumbered by default"
    );

    // An explicit `-a sectnums!` overrides `-n`, leaving the heading unnumbered.
    assert!(
        convert(&["adoc", "-n", "-a", "sectnums!", "-o", "-"])
            .contains(r#"<h2 id="_one">One</h2>"#),
        "-a sectnums! overrides -n"
    );
}

// The known-limitations note records the scope of `adoc`'s option coverage; it
// carries no rule to verify here.
non_normative!(
    r#"
[NOTE]
.Known limitations
====
`adoc` implements a focused subset of the `asciidoctor` CLI. It does not accept
the full option catalog that `asciidoctor` documents in its man page, and it has
no `asciidoctor(1)` man page of its own, so `adoc --help` is the authoritative
list of the options it supports. Generating a man(1) page for `adoc` is tracked
in https://github.com/asciidoc-rs/asciidoc-html5/issues/94[issue #94].
====
"#
);
