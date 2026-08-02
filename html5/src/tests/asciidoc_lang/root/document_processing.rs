//! Coverage of the AsciiDoc language description's *Document Processing* page.
//!
//! The page explains that AsciiDoc is a writing format, not a publishing
//! format: you write the source, then an AsciiDoc processor converts it to a
//! publishable output. Most of that is descriptive prose, and the multi-backend
//! machinery it sketches (a processor with several built-in converters,
//! including one for DocBook) is broader than this HTML5-only crate, so it is
//! tracked as non-normative. Two concrete claims *are* this crate's behavior
//! and are verified: the HTML converter turns the (default) `html` backend into
//! HTML output, and the processor works as a string-in/string-out,
//! parse-then-convert pipeline.

use crate::{
    convert, convert_document, load,
    tests::{assert_html::assert_css, sdd::*},
};

track_file!("ref/asciidoc-lang/docs/modules/ROOT/pages/document-processing.adoc");

// The framing prose (write source, then convert it to publish) and the general
// description of backends and converters. This crate renders HTML5 only —
// DocBook and the other output formats named here are out of scope — so the
// multi-backend selection described in this paragraph is non-normative here;
// the one case this crate implements, the html backend, is verified just below.
non_normative!(
    r#"
= Document Processing

AsciiDoc is specifically a writing format, not a publishing format.
In other words, it's not WYSIWYG like when you write in a word processor.
Instead, what you write is the AsciiDoc source.
You then use an AsciiDoc processor, such as Asciidoctor, to convert the AsciiDoc source into a publishable format.
It's this output that you publish.

Converting the AsciiDoc source is an opportunity to interpret and embellish your content to get more out of it than what you put in.
The work of converting the AsciiDoc source to another format is handled by a converter.
While there is a strong relationship between the language and the converters, these two aspects are not explicitly coupled.

An AsciiDoc processor provides several built-in converters, including ones for making HTML and DocBook.
To activate one of these converters, you set the backend on the document (default: html).
The backend is a keyword that tells the processor which output format you want to make.
The processor then selects the converter that makes that output format.
"#
);

// This crate is the HTML converter for the default `html` backend: it turns
// AsciiDoc into HTML output. A converted document carries HTML5 markup — here,
// the paragraph's `<div class="paragraph"><p>…</p></div>` structure.
#[test]
fn the_html_converter_makes_html_output() {
    verifies!(
        r#"
For example, the HTML converter handles the html backend to make HTML output.

"#
    );

    let output = convert("= Doc\n\nHello.\n");
    assert_css(&output, "div.paragraph", 1);
    assert_css(&output, "div.paragraph > p", 1);
}

// The two steps the page describes are two distinct entry points in this
// crate's API. `load` performs step one — it parses the source string into a
// structured `Document`. `convert_document` performs step two — it hands that
// parsed document to the renderer (the converter) to produce the output.
// `convert` is the same pipeline collapsed into a single call: a string goes in
// and another string comes out, exactly as the page's "in short" summary says.
#[test]
fn the_processor_parses_a_string_then_converts_it_to_a_string() {
    verifies!(
        r#"
An AsciiDoc processor actually works in two steps.
First, it parses the AsciiDoc document.
This parsing produces a structured document that reflects the written structure and interprets all the meaningful markup.
The processor then passes this structured document to the converter to transform it into the output format.

In short, the processor accepts a string (which may be read from a file), parses it into a structure document, then produces another string (which may be written to a file).
"#
    );

    let source = "= Title\n\nA paragraph.\n";

    // Step one: parse the source string into a structured document.
    let document = load(source);

    // Step two: convert the parsed document into the output string.
    let two_step = convert_document(&document);

    // The one-call pipeline: string in, string out.
    let one_step = convert(source);

    assert_eq!(one_step, two_step);
    assert!(one_step.contains("<p>A paragraph.</p>"));
}
