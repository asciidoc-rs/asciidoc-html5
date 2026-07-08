use clap::Parser as _;

use crate::{run, tests::sdd::*, Cli};

track_file!("docs/modules/cli/pages/index.adoc");

// This crate's "Process AsciiDoc Using the CLI" page. Descriptive prose is
// tracked as non-normative; the section headings, the `adoc` invocations, and
// the claims the page makes about their behavior are verified by the tests
// below, which drive the command end to end.

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

"#
);

// The version invocations: `adoc --version` and its short form `-V` both print
// `adoc <version>` to standard output.
#[test]
fn checks_the_version() {
    verifies!(
        r#"
== Check the version

To confirm that the CLI is available, run:

 $ adoc --version

You can shorten the `--version` flag to `-V`:

 $ adoc -V

Either form prints the version of `adoc` to standard output:

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

// The conversion invocations: `adoc document.adoc` derives the output name, and
// `adoc document.adoc -o out.html` writes to the named file instead.
#[test]
fn converts_a_file() {
    verifies!(
        r#"
== Convert an AsciiDoc file

To convert an `.adoc` file, pass its name to `adoc`:

 $ adoc document.adoc

With the built-in defaults and no output option, `adoc` writes a new file in the
same directory as the input, with the same base name but the `.html` extension,
so this command produces [.path]_document.html_.

To choose the output file yourself, pass `-o` (longhand `--output`); pass `-o -`
to write the HTML5 to standard output instead:

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
    assert!(
        out.exists(),
        "-o did not create the output file at the designated path"
    );
    let out_html = std::fs::read_to_string(&out).expect("read -o output file");
    assert_eq!(out_html, html);

    // `-o -` writes the HTML5 to standard output instead of a file.
    let cli = Cli::parse_from([
        "adoc",
        path.to_str().expect("temp path is UTF-8"),
        "-o",
        "-",
    ]);
    let mut stdout = Vec::new();
    run(&cli, &mut stdout).expect("adoc writes to stdout with -o -");
    assert_eq!(String::from_utf8(stdout).expect("stdout is UTF-8"), html);

    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_file(&derived);
    let _ = std::fs::remove_file(&out);
}

// The attribute invocations: `-a` sets a document attribute (`name`,
// `name=value`, or `name!`), overriding the document by default, with a
// trailing `@` making it a soft default the document overrides instead.
#[test]
fn sets_document_attributes() {
    verifies!(
        r#"
== Set document attributes

Pass `-a` (longhand `--attribute`) to set a document attribute from the command
line, the way `asciidoctor -a` does. Give `name` to set an attribute, `name=value`
to set it to a value, or `name!` to unset it. Repeat `-a` to set more than one:

 $ adoc -a linkcss -a webfonts! document.adoc

By default the value you pass overrides any assignment of the same name inside
the document. Append `@` (for example `-a webfonts=Ubuntu+Mono:400@`) to make it
a soft default that a document assignment of the same name overrides instead.

"#
    );

    // Converts `source` through the CLI with the given extra arguments, writing
    // to stdout via `-o -`, and returns the captured HTML5.
    fn adoc(args: &[&str], source: &str) -> String {
        let path =
            std::env::temp_dir().join(format!("adoc-docs-cli-attr-{}.adoc", std::process::id()));
        std::fs::write(&path, source).expect("write temp input");

        let mut argv = vec!["adoc"];
        argv.extend_from_slice(args);
        argv.extend(["-o", "-", path.to_str().expect("temp path is UTF-8")]);

        let cli = Cli::parse_from(argv);
        let mut stdout = Vec::new();
        run(&cli, &mut stdout).expect("adoc converts with attributes");
        let _ = std::fs::remove_file(&path);
        String::from_utf8(stdout).expect("stdout is UTF-8")
    }

    // `-a linkcss -a webfonts!` sets one attribute and unsets another: the
    // stylesheet is linked rather than embedded, and the web-font link is gone.
    let html = adoc(&["-a", "linkcss", "-a", "webfonts!"], "= Doc\n\nBody.");
    assert!(html.contains("<link rel=\"stylesheet\" href=\"./asciidoctor.css\">"));
    assert!(!html.contains("<link rel=\"stylesheet\" href=\"https://fonts.googleapis.com"));

    // `-a name=value` overrides a document-header assignment of the same name.
    let header = "= Doc\n:webfonts: from-header\n\nBody.";
    let overridden = adoc(&["-a", "webfonts=from-cli"], header);
    assert!(overridden.contains("family=from-cli"));
    assert!(!overridden.contains("family=from-header"));

    // The exact soft-default example the page cites,
    // `-a webfonts=Ubuntu+Mono:400@`: its value applies when the document does
    // not assign `webfonts` itself...
    let silent = adoc(&["-a", "webfonts=Ubuntu+Mono:400@"], "= Doc\n\nBody.");
    assert!(silent.contains("family=Ubuntu+Mono:400"));

    // ...but a document-header assignment of the same name overrides it.
    let softened = adoc(&["-a", "webfonts=Ubuntu+Mono:400@"], header);
    assert!(softened.contains("family=from-header"));
    assert!(!softened.contains("family=Ubuntu+Mono:400"));
}

// The help invocations: `adoc --help` prints the usage statement, and its short
// form `-h` prints a shorter summary.
#[test]
fn prints_help() {
    verifies!(
        r#"
== Get help

The `--help` option prints the usage statement for the `adoc` command, including
its options and a few examples:

 $ adoc --help

You can shorten the `--help` flag to `-h`, which prints a shorter summary:

 $ adoc -h

"#
    );

    non_normative!(
        r#"
[NOTE]
.Known limitations
====
The `adoc` command covers a small part of the `asciidoctor` CLI. Beyond the
`-a`/`--attribute` option shown above, it does not yet accept the many behavior
options that `asciidoctor` provides, and its `--help` output is a single usage
statement rather than the topic-grouped help of `asciidoctor`; the `manpage` and
`syntax` help topics are not available. Printing an AsciiDoc syntax crib sheet with `--help syntax` is
tracked in https://github.com/asciidoc-rs/asciidoc-html5/issues/31[issue #31].
The short form of `--version` is `-V`, following the Rust convention, rather than
the `-v` used by `asciidoctor`.
====
"#
    );

    // clap surfaces a help request as a `DisplayHelp` "error" carrying the
    // rendered help. Both the long `--help` and short `-h` include the usage
    // statement for the `adoc` command, and `-h` is the shorter of the two.
    let long = Cli::try_parse_from(["adoc", "--help"]).expect_err("--help displays help");
    assert_eq!(long.kind(), clap::error::ErrorKind::DisplayHelp);
    assert!(long.to_string().contains("Usage: adoc"));

    let short = Cli::try_parse_from(["adoc", "-h"]).expect_err("-h displays help");
    assert_eq!(short.kind(), clap::error::ErrorKind::DisplayHelp);
    assert!(short.to_string().contains("Usage: adoc"));
    assert!(
        short.to_string().len() < long.to_string().len(),
        "-h summary should be shorter than --help"
    );
}
