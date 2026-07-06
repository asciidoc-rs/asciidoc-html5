use clap::Parser as _;

use crate::{run, tests::sdd::*, Cli};

track_file!("docs/modules/ROOT/pages/cli.adoc");

// This crate's "Process AsciiDoc Using the CLI" page. Its prose is descriptive
// documentation, tracked as non-normative; the `adoc` invocations it shows are
// verified by the tests below, which drive the command end to end.

non_normative!(
    r#"
= Process AsciiDoc Using the CLI
:navtitle: Use the CLI
:description: How to check the version of adoc, convert a file, and print help from the command line.

Once `asciidoc-html5` is installed, the command line interface (CLI) named
`adoc` is available on your PATH. This page shows how to confirm the version,
convert a file, and reach the built-in help.

[NOTE]
====
The prose on this page is non-normative documentation. The `adoc` invocations it
shows are normative: they are verified against the implementation, so the
documented behavior is guaranteed.
====

== Check the version

To confirm that the CLI is available, run:

"#
);

// The version invocations: `adoc --version`, its short form `-V`, and the
// `adoc <version>` line both print.
#[test]
fn checks_the_version() {
    verifies!(
        r#"
 $ adoc --version

"#
    );

    non_normative!(
        r#"
You can shorten the `--version` flag to `-V`:

"#
    );

    verifies!(
        r#"
 $ adoc -V

"#
    );

    non_normative!(
        r#"
Either form prints the version of `adoc` to standard output:

"#
    );

    verifies!(
        r#"
 adoc <version>

"#
    );

    non_normative!(
        r#"
Unlike `asciidoctor`, `adoc` is a native binary with no Ruby, JVM, or JavaScript
runtime, so it prints only its own version -- there is no separate
runtime-environment line.

"#
    );

    // Both `--version` and the short `-V` print `adoc <version>` and nothing
    // else. clap surfaces the request as a `DisplayVersion` "error" carrying the
    // version string.
    let long = Cli::try_parse_from(["adoc", "--version"]).expect_err("--version displays version");
    assert_eq!(long.kind(), clap::error::ErrorKind::DisplayVersion);
    assert!(long.to_string().starts_with("adoc "));

    let short = Cli::try_parse_from(["adoc", "-V"]).expect_err("-V displays version");
    assert_eq!(short.kind(), clap::error::ErrorKind::DisplayVersion);
    assert_eq!(short.to_string(), long.to_string());
}

non_normative!(
    r#"
== Convert an AsciiDoc file

To convert an `.adoc` file, pass its name to `adoc`:

"#
);

// The conversion invocations: `adoc document.adoc` derives the output name, and
// `adoc document.adoc -o out.html` writes to the named file instead.
#[test]
fn converts_a_file() {
    verifies!(
        r#"
 $ adoc document.adoc

"#
    );

    non_normative!(
        r#"
With the built-in defaults and no output option, `adoc` writes a new file in the
same directory as the input, with the same base name but the `.html` extension,
so this command produces [.path]_document.html_.

To choose the output file yourself, pass `-o` (longhand `--output`); pass `-o -`
to write the HTML5 to standard output instead:

"#
    );

    verifies!(
        r#"
 $ adoc document.adoc -o out.html

"#
    );

    // `adoc document.adoc` with no `-o` derives `document.html` alongside the
    // input.
    let source = "= Hello\n\nWorld.";
    let path = std::env::temp_dir().join(format!("adoc-docs-cli-{}.adoc", std::process::id()));
    let derived = path.with_extension("html");
    std::fs::write(&path, source).expect("write temp input");

    let cli = Cli::parse_from(["adoc", path.to_str().expect("temp path is UTF-8")]);
    let mut stdout = Vec::new();
    run(&cli, &mut stdout).expect("adoc converts the file");

    assert!(stdout.is_empty(), "adoc wrote to stdout on success");
    let html = std::fs::read_to_string(&derived).expect("read derived output file");
    assert!(html.starts_with("<!DOCTYPE html>"));
    assert!(html.contains("<title>Hello</title>"));

    // `-o out.html` names the output file explicitly, producing the same HTML5.
    let out = std::env::temp_dir().join(format!("adoc-docs-cli-out-{}.html", std::process::id()));
    let cli = Cli::parse_from([
        "adoc",
        path.to_str().expect("temp path is UTF-8"),
        "-o",
        out.to_str().expect("out path is UTF-8"),
    ]);
    let mut stdout = Vec::new();
    run(&cli, &mut stdout).expect("adoc converts to the named output file");

    assert!(stdout.is_empty(), "adoc wrote to stdout with -o set");
    let out_html = std::fs::read_to_string(&out).expect("read -o output file");
    assert_eq!(out_html, html);

    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_file(&derived);
    let _ = std::fs::remove_file(&out);
}

non_normative!(
    r#"
== Get help

The `--help` option prints the usage statement for the `adoc` command, including
its options and a few examples:

"#
);

// The help invocations: `adoc --help` and its short form `-h` both print the
// usage statement.
#[test]
fn prints_help() {
    verifies!(
        r#"
 $ adoc --help

"#
    );

    non_normative!(
        r#"
You can shorten the `--help` flag to `-h`, which prints a shorter summary:

"#
    );

    verifies!(
        r#"
 $ adoc -h

"#
    );

    non_normative!(
        r#"
[NOTE]
.Known limitations
====
The `adoc` command covers a small part of the `asciidoctor` CLI. It does not yet
accept the many attribute (`-a`) and behavior options that `asciidoctor`
provides, and its `--help` output is a single usage statement rather than the
topic-grouped help of `asciidoctor`; the `manpage` and `syntax` help topics are
not available. Printing an AsciiDoc syntax crib sheet with `--help syntax` is
tracked in https://github.com/asciidoc-rs/asciidoc-html5/issues/31[issue #31].
The short form of `--version` is `-V`, following the Rust convention, rather than
the `-v` used by `asciidoctor`.
====
"#
    );

    // clap surfaces a help request as a `DisplayHelp` "error" carrying the
    // rendered help. Both the long `--help` and short `-h` include the usage
    // statement for the `adoc` command.
    let long = Cli::try_parse_from(["adoc", "--help"]).expect_err("--help displays help");
    assert_eq!(long.kind(), clap::error::ErrorKind::DisplayHelp);
    assert!(long.to_string().contains("Usage: adoc"));

    let short = Cli::try_parse_from(["adoc", "-h"]).expect_err("-h displays help");
    assert_eq!(short.kind(), clap::error::ErrorKind::DisplayHelp);
    assert!(short.to_string().contains("Usage: adoc"));
}
