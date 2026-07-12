use clap::Parser as _;

use crate::{run, tests::sdd::*, Cli};

track_file!("docs/modules/ROOT/pages/index.adoc");

non_normative!(
    r#"
= AsciiDoc HTML5
:navtitle: Introduction
:description: A brief introduction to asciidoc-html5, the Rust HTML5 renderer for AsciiDoc, and how it relates to AsciiDoc and Asciidoctor.

`asciidoc-html5` is a Rust library and command-line tool that renders
https://asciidoc.org[AsciiDoc] into HTML5. It builds on the
https://crates.io/crates/asciidoc-parser[`asciidoc-parser`] crate and aims for
output compatible with https://asciidoctor.org[Asciidoctor]'s default `html5`
backend.

[NOTE]
====
The descriptions on this page are non-normative documentation. The command-line
and API invocations it shows, on the other hand, are normative: they are
verified against the implementation, so the documented behavior is guaranteed.
The rules governing the AsciiDoc the renderer accepts come from the AsciiDoc
language and from Asciidoctor, whose `html5` backend is the compatibility oracle
for this crate.
====

== What is asciidoc-html5?

`asciidoc-html5` is a native Rust processor that parses AsciiDoc into a document
model and converts it to HTML5. Parsing is handled by the `asciidoc-parser`
crate; this project walks the parsed document and assembles the block-level HTML
structure that Asciidoctor's `html5` backend produces.

The project ships two components:

`asciidoc-html5`:: the renderer library, which other Rust tools can embed to
convert AsciiDoc to HTML5 as one step of a larger pipeline.

`adoc`:: a thin command-line front end over the library that reads AsciiDoc and
writes HTML5.

Unlike Asciidoctor, which is written in Ruby, `asciidoc-html5` is written in
Rust and needs no Ruby, JVM, or JavaScript runtime.

== Basic usage

`asciidoc-html5` provides two interfaces for converting AsciiDoc documents: a
CLI named `adoc` and a Rust API in the `asciidoc_html5` crate. The following
table gives you an idea of how to use these interfaces.

|===
^|CLI ^|API

"#
);

// The "Basic usage" section, verified from the CLI side.
#[test]
fn basic_usage_converts_a_document_file() {
    // The CLI column of the table: `adoc document.adoc`.
    verifies!(
        r#"
a|
 $ adoc document.adoc

"#
    );

    // The API column of the table (verified by the `asciidoc-html5` crate).
    non_normative!(
        r#"
a|
[,rust]
----
let html =
    asciidoc_html5::convert_file("document.adoc")?;
----
"#
    );

    // The CLI output description: writes the HTML to the derived output file.
    verifies!(
        r#"

|Reads `document.adoc` and writes the rendered HTML5 to _document.html_.
"#
    );

    // Drive the exact command shown on the page — `adoc document.adoc` — and
    // check that a complete HTML5 document is written to the derived output file
    // (input name with its extension swapped for `.html`), as the CLI column of
    // the table describes.
    let source = "= Hello\n\nWorld.";
    let path = std::env::temp_dir().join(format!("adoc-introduction-{}.adoc", std::process::id()));
    let derived = path.with_extension("html");
    std::fs::write(&path, source).expect("write temp input");

    let cli = Cli::parse_from(["adoc", path.to_str().expect("temp path is UTF-8")]);
    let mut stdout = Vec::new();
    run(&cli, &mut stdout).expect("adoc converts the file");

    assert!(stdout.is_empty(), "adoc wrote to stdout on success");
    let html = std::fs::read_to_string(&derived).expect("read derived output file");
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_file(&derived);

    assert!(html.starts_with("<!DOCTYPE html>"));
    assert!(html.contains("<title>Hello</title>"));
    assert!(html.contains("<p>World.</p>"));
}

non_normative!(
    r#"
|Reads `document.adoc` and returns the rendered HTML5 as a `String`.
|===

In the simplest case, you give an AsciiDoc document to `asciidoc-html5` and it
gives you back a complete HTML5 document you can publish.

"#
);

// The `adoc --help` invocation shown under "Basic usage".
#[test]
fn help_lists_usage_examples() {
    verifies!(
        r#"
Pass `--help` to the CLI to see every option:

 $ adoc --help
"#
    );

    // `adoc --help` renders the long help. clap reports a help request as a
    // `DisplayHelp` "error" whose message is the rendered help text, which must
    // carry the usage examples wired up on the command.
    let err = Cli::try_parse_from(["adoc", "--help"]).expect_err("--help displays help");
    assert_eq!(err.kind(), clap::error::ErrorKind::DisplayHelp);

    let help = err.to_string();
    assert!(help.contains("Examples:"));
    assert!(help.contains("adoc document.adoc"));
}

non_normative!(
    r#"

== API examples

The Rust API's three conversion entry points each return a complete, standalone
HTML5 document. The file-based `convert_file` shown above is the most common; the
other two are `convert`, for AsciiDoc you already hold in memory, and
`convert_document`, for a document you have already parsed.

Convert AsciiDoc held in memory with `convert`:

[,rust]
----
let html = asciidoc_html5::convert("= Hello\n\nWorld.");
----

To load a document without converting it — say, to inspect or transform it first
— parse it with `load` and render the result with `convert_document`:

[,rust]
----
let doc = asciidoc_html5::load("= Hello\n\nWorld.");
let html = asciidoc_html5::convert_document(&doc);
----

== Relationship to AsciiDoc and Asciidoctor

AsciiDoc is the language; `asciidoc-html5` is one of its processors.

You compose documents using the AsciiDoc language, a concise text-based writing
format. AsciiDoc is not itself a publishing format, so a processor is needed to
convert the source into something you can publish. `asciidoc-html5` is such a
processor: it reads AsciiDoc source and converts it to HTML5.

Asciidoctor is the reference implementation of the AsciiDoc language. This crate
treats Asciidoctor's default `html5` backend as its compatibility target, so
that a given document renders the same whether it is processed by Asciidoctor or
by `asciidoc-html5`. Where the two differ, Asciidoctor is treated as correct
unless the difference is a documented limitation of this crate or of
`asciidoc-parser`.
"#
);
