use std::fs;

use asciidoc_parser::{blocks::IsBlock, Parser};

use crate::{convert_file, tests::sdd::*};

track_file!("docs/modules/api/pages/convert-files.adoc");

// This crate's own "Load and Convert Files Using the API" page. The prose is
// descriptive documentation, tracked as non-normative; the Rust snippets it
// shows are verified by the tests below, each driving the same API the snippet
// demonstrates against the page's sample document. The page is entirely about
// the `asciidoc_html5` (and `asciidoc_parser`) API, so — like the other API
// pages — it is tracked only from this crate.

// The page's sample document, used throughout.
const SAMPLE: &str = "= Document Title\n\nThe main content.";

/// Writes [`SAMPLE`] to a uniquely named temp file, converts it with
/// `convert_file`, and returns the rendered HTML (removing the file afterward).
fn convert_sample_file(label: &str) -> String {
    let path = std::env::temp_dir().join(format!(
        "asciidoc-html5-docs-convert-files-{label}-{}.adoc",
        std::process::id()
    ));
    fs::write(&path, SAMPLE).expect("write temp input");
    let html = convert_file(&path).expect("convert_file reads and renders");
    let _ = fs::remove_file(&path);
    html
}

non_normative!(
    r#"
= Load and Convert AsciiDoc Files Using the API
:navtitle: Load and Convert Files
:description: How to load an AsciiDoc file into a document model and convert an AsciiDoc file to HTML5 with the asciidoc_html5 Rust API.

This page explains how to load and convert AsciiDoc files using the API.

[NOTE]
====
The prose on this page is non-normative documentation. The API calls it shows are
normative: they are verified against the implementation, so the documented
behavior is guaranteed.
====

"#
);

// Loading parses the source into a document model with `asciidoc_parser`'s
// `Parser`, which returns a `Document` carrying the document's block structure.
#[test]
fn loading_returns_a_document_with_block_structure() {
    verifies!(
        r#"
== Load an AsciiDoc file

When you load AsciiDoc, you parse the document (down to the block level) into a
document model -- an in-memory tree of the document's elements. `asciidoc-html5`
relies on https://crates.io/crates/asciidoc-parser[`asciidoc-parser`] for this
step, which returns an
https://docs.rs/asciidoc-parser/latest/asciidoc_parser/document/struct.Document.html[`asciidoc_parser::Document`].
That object contains the full block structure of the AsciiDoc document.

"#
    );

    let doc = Parser::default().parse(SAMPLE);
    assert!(doc.nested_blocks().next().is_some());
}

non_normative!(
    r#"
[NOTE]
====
Asciidoctor defers inline parsing until conversion, so a freshly loaded document
has not yet processed its inline content. `asciidoc-parser` differs here: it
applies inline substitutions eagerly, while parsing, so each block already
carries its rendered inline HTML by the time `parse` returns.
====

"#
);

// Reading a file from disk and parsing its contents. There is no dedicated
// file-loading entrypoint, so the snippet reads the file and hands the source
// to `Parser::parse`; the parsed document reports the sample's title.
#[test]
fn a_file_is_read_and_parsed_into_a_document() {
    verifies!(
        r#"
Let's assume we're working with the following AsciiDoc document:

._document.adoc_
[,asciidoc]
----
= Document Title

The main content.
----

To read this source file from disk and parse it into a document model, read the
file and hand its contents to `asciidoc_parser::Parser`:

[,rust]
----
let source = std::fs::read_to_string("document.adoc")?;
let doc = asciidoc_parser::Parser::default().parse(&source);
----

"#
    );

    let path = std::env::temp_dir().join(format!(
        "asciidoc-html5-docs-convert-files-load-{}.adoc",
        std::process::id()
    ));
    fs::write(&path, SAMPLE).expect("write temp input");

    let source = fs::read_to_string(&path).expect("read temp input");
    let doc = Parser::default().parse(&source);
    let _ = fs::remove_file(&path);

    assert_eq!(doc.doctitle(), Some("Document Title"));
}

non_normative!(
    r#"
If you already hold the source in memory, pass it to `parse` directly -- there is
no separate file entry point for loading.

"#
);

// The loaded document reports its title through `Document::doctitle`.
#[test]
fn the_document_reports_its_title() {
    verifies!(
        r#"
Using the `doc` value, you can get information about the document, such as the
document title:

[,rust]
----
assert_eq!(doc.doctitle(), Some("Document Title"));
----

"#
    );

    let doc = Parser::default().parse(SAMPLE);
    assert_eq!(doc.doctitle(), Some("Document Title"));
}

// The document's attributes are reachable through `has_attribute`; the title is
// captured as the `doctitle` attribute.
#[test]
fn the_document_exposes_its_attributes() {
    verifies!(
        r#"
You can also inspect the document attributes:

[,rust]
----
assert!(doc.has_attribute("doctitle"));
----

"#
    );

    let doc = Parser::default().parse(SAMPLE);
    assert!(doc.has_attribute("doctitle"));
}

// Paragraph blocks are found by filtering the document's `nested_blocks` on
// their `resolved_context`, which is `"paragraph"` for the sample's one
// paragraph.
#[test]
fn paragraph_blocks_are_found_by_context() {
    verifies!(
        r#"
Going deeper, you can find blocks in the document, such as all the top-level
paragraph blocks, by filtering the document's blocks on their context:

[,rust]
----
use asciidoc_parser::blocks::IsBlock;

let paragraphs = doc
    .nested_blocks()
    .filter(|block| block.resolved_context().as_ref() == "paragraph")
    .count();
assert_eq!(paragraphs, 1);
----

"#
    );

    let doc = Parser::default().parse(SAMPLE);
    let paragraphs = doc
        .nested_blocks()
        .filter(|block| block.resolved_context().as_ref() == "paragraph")
        .count();
    assert_eq!(paragraphs, 1);
}

non_normative!(
    r#"
However, if you're only interested in converting the AsciiDoc source, then it's
better to use the `convert_file` entry point.

"#
);

// Converting produces HTML5, the only output format the library supports.
#[test]
fn converting_produces_html5() {
    verifies!(
        r#"
== Convert an AsciiDoc file

When you convert AsciiDoc, you parse and convert the document to the output
format in a single step. `asciidoc-html5` produces HTML5 -- the only output
format it supports, and the same format the `adoc` CLI produces.

"#
    );

    let html = convert_sample_file("default");
    assert!(html.starts_with("<!DOCTYPE html>"));
    assert!(html.contains("<title>Document Title</title>"));
}

// `convert_file` reads, parses, and renders the file to an HTML5 document.
#[test]
fn convert_file_renders_the_file_to_html5() {
    verifies!(
        r#"
Let's again assume we're working with the following AsciiDoc document:

._document.adoc_
[,asciidoc]
----
= Document Title

The main content.
----

To convert this source file to HTML5, call `convert_file`:

[,rust]
----
let html = asciidoc_html5::convert_file("document.adoc")?;
----

"#
    );

    let html = convert_sample_file("to-html5");
    assert!(html.contains("<title>Document Title</title>"));
    assert!(html.contains("The main content."));
}

// `convert_file` returns the HTML as a `String`; writing it to a file is the
// caller's job, done here with `fs::write`.
#[test]
fn the_returned_html_can_be_written_to_a_file() {
    verifies!(
        r#"
`convert_file` reads the file, parses it, and returns the rendered HTML5 as a
`String`. Unlike Asciidoctor, it does not write an output file: what you do with
the returned string is up to you. To write it to disk under a derived or chosen
name, use the xref:ROOT:convert-your-first-file.adoc[`adoc` CLI], or write the
string yourself:

[,rust]
----
let html = asciidoc_html5::convert_file("document.adoc")?;
std::fs::write("out.html", html)?;
----

"#
    );

    let html = convert_sample_file("write-out");

    let out = std::env::temp_dir().join(format!(
        "asciidoc-html5-docs-convert-files-out-{}.html",
        std::process::id()
    ));
    fs::write(&out, &html).expect("write output file");
    let written = fs::read_to_string(&out).expect("read output file");
    let _ = fs::remove_file(&out);

    assert_eq!(written, html);
}

non_normative!(
    r#"
[NOTE]
.Known limitation
====
Asciidoctor's `convert_file` accepts a `:to_file` option to control the output
file and a `:backend` option to select a different converter -- for example,
`backend: 'docbook'` to emit DocBook XML. `asciidoc-html5` offers neither: it
returns HTML5 as a `String`, leaving the file writing to the caller (or the
`adoc` CLI), and DocBook and the other Asciidoctor backends are not planned.
HTML5 is the only output format.
====

That covers the basics of loading and converting AsciiDoc using the API.
"#
);
