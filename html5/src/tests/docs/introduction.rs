use std::fs;

use crate::{convert, convert_file, tests::sdd::*};

track_file!("docs/modules/ROOT/pages/index.adoc");

// The introduction page is descriptive documentation, not a specification, so
// its entire content is non-normative. The behavior it advertises — the
// baseline simplest-case conversion in the "Basic usage" section — is exercised
// by the tests below.
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
This page is descriptive documentation, not a specification: its content is
non-normative. The normative rules this renderer follows come from the AsciiDoc
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

a|
 $ adoc document.adoc

a|
[,rust]
----
let html =
    asciidoc_html5::convert_file("document.adoc")?;
----

|Reads `document.adoc` and writes the rendered HTML5 to standard output.
|Reads `document.adoc` and returns the rendered HTML5 as a `String`.
|===

In the simplest case, you give an AsciiDoc document to `asciidoc-html5` and it
gives you back a complete HTML5 document you can publish. To render AsciiDoc you
already hold in memory, pass it to `asciidoc_html5::convert` instead of reading
from a file.

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

// The baseline simplest case from the "Basic usage" section: hand an AsciiDoc
// document to the API and get back a complete, standalone HTML5 document.
#[test]
fn convert_renders_a_complete_html5_document() {
    let html = convert("= Hello\n\nWorld.");

    assert!(html.starts_with("<!DOCTYPE html>"));
    assert!(html.contains("<title>Hello</title>"));
    assert!(html.contains("<div class=\"paragraph\">\n<p>World.</p>\n</div>"));
    assert!(html.trim_end().ends_with("</body>\n</html>"));
}

// The file-based API shown in the page's "Basic usage" table: `convert_file`
// reads a document from disk and renders it exactly as `convert` would.
#[test]
fn convert_file_reads_and_renders_a_document() {
    let source = "= Hello\n\nWorld.";
    let path = std::env::temp_dir().join(format!(
        "asciidoc-html5-introduction-{}.adoc",
        std::process::id()
    ));
    fs::write(&path, source).expect("write temp input");

    let html = convert_file(&path).expect("convert_file reads and renders");
    let _ = fs::remove_file(&path);

    assert_eq!(html, convert(source));
    assert!(html.contains("<title>Hello</title>"));
}
