use std::fs;

use asciidoc_parser::{blocks::IsBlock, document::InterpretedValue};

use crate::{convert_file, load, load_file, tests::sdd::*};

track_file!("ref/asciidoctor/docs/modules/api/pages/convert-files.adoc");

// Asciidoctor's "Load and Convert Files Using the API" page, tracked from the
// library crate. It documents two Ruby entrypoints: `Asciidoctor.load_file`,
// which parses a file into an `Asciidoctor::Document`, and
// `Asciidoctor.convert_file`, which parses and converts a file to an output
// format. This crate has a direct analog for each: `load_file` parses a file
// into a `Document` (the analog of an `Asciidoctor::Document`; `load` is its
// string counterpart), and `convert_file` parses and converts one. The
// document-inspection calls the page shows — `doctitle`, the attributes, and
// finding paragraph blocks — map to `Document::doctitle`,
// `has_attribute`/`attribute_value`, and filtering `nested_blocks` by context.
//
// Three of the page's claims describe behavior this crate does not share, so
// they are non-normative: the CAUTION that inline content is parsed lazily
// (`asciidoc_parser` parses inline eagerly, at parse time), the `:to_file`
// option (this crate returns the HTML as a `String` and leaves file writing to
// the caller or the `adoc` CLI), and the `:backend` option (this crate renders
// HTML5 only — DocBook and the other Asciidoctor backends are not planned). The
// Ruby `safe:` option likewise has no bearing on loading or converting here.

// The page's sample document, used throughout.
const SAMPLE: &str = "= Document Title\n\nThe main content.";

/// Writes [`SAMPLE`] to a uniquely named temp file, converts it with
/// `convert_file`, and returns the rendered HTML (removing the file afterward).
fn convert_sample_file(label: &str) -> String {
    let path = std::env::temp_dir().join(format!(
        "asciidoc-html5-api-convert-files-{label}-{}.adoc",
        std::process::id()
    ));
    fs::write(&path, SAMPLE).expect("write temp input");
    let html = convert_file(&path).expect("convert_file reads and renders");
    let _ = fs::remove_file(&path);
    html
}

non_normative!(
    r#"
= Load and Convert Files Using the API
:navtitle: Load and Convert Files

This page explains how to load and convert AsciiDoc files using the API.

"#
);

// Loading parses the source into a document model. `load` is this crate's load
// step: it parses down to the block level into a `Document` (the analog of an
// `Asciidoctor::Document`) that carries the document's full block structure.
#[test]
fn loading_parses_into_a_document_model() {
    verifies!(
        r#"
== Load an AsciiDoc file

When you load AsciiDoc using the API, you're telling Asciidoctor to parse the document (down to the block level) and return an `Asciidoctor::Document` object.
This object contains the full block structure of the AsciiDoc document.

"#
    );

    let doc = load(SAMPLE);

    // The parsed document exposes its block structure.
    assert!(doc.nested_blocks().next().is_some());
}

// This crate parses inline content eagerly, so the CAUTION does not hold here.
// `asciidoc_parser` applies inline substitutions while parsing — each block
// already carries its rendered inline HTML when `parse` returns — rather than
// deferring that work to conversion. The span is therefore non-normative.
non_normative!(
    r#"
CAUTION: Loading a document currently does not parse the inline content.
That processing is deferred until the parsed document is converted.

"#
);

// `load_file` reads a file and parses its contents. This crate's `load_file` is
// the direct analog: it reads the file and parses it into a `Document`. The
// Ruby `safe: :safe` option does not affect loading here.
#[test]
fn load_file_reads_and_parses_the_source_file() {
    verifies!(
        r#"
Let's assume we're working with the following AsciiDoc document:

._document.adoc_
[,asciidoc]
----
= Document Title

The main content.
----

To parse this source file into an `Asciidoctor::Document` object, use the following API call:

[,ruby]
----
doc = Asciidoctor.load_file 'document.adoc', safe: :safe
----

"#
    );

    let path = std::env::temp_dir().join(format!(
        "asciidoc-html5-api-convert-files-load-{}.adoc",
        std::process::id()
    ));
    fs::write(&path, SAMPLE).expect("write temp input");

    // `load_file` reads and parses the file into the loaded document.
    let doc = load_file(&path).expect("load_file reads and parses");
    let _ = fs::remove_file(&path);
    assert_eq!(doc.doctitle(), Some("Document Title"));
}

// `doc.doctitle` reads the document title from the loaded document. This
// crate's counterpart is `Document::doctitle`, which returns the same title.
#[test]
fn the_loaded_document_reports_its_title() {
    verifies!(
        r#"
Using the object assigned to the `doc` variable, you can get information about the document, such as the document title.

[,ruby]
----
puts doc.doctitle
# => "Document Title"
----

"#
    );

    let doc = load(SAMPLE);
    assert_eq!(doc.doctitle(), Some("Document Title"));
}

// `pp doc.attributes` inspects the document's attributes. Here the attributes
// are reachable through `has_attribute`/`attribute_value`; the document title
// is captured as the `doctitle` attribute.
#[test]
fn the_loaded_document_exposes_its_attributes() {
    verifies!(
        r#"
You can also inspect all the document attributes:

[,ruby]
----
pp doc.attributes
----

"#
    );

    let doc = load(SAMPLE);

    assert!(doc.has_attribute("doctitle"));
    assert_eq!(
        doc.attribute_value("doctitle"),
        InterpretedValue::Value("Document Title".to_string())
    );
}

// Finding the paragraph blocks. Asciidoctor's `find_by context: :paragraph`
// selects blocks by context; this crate filters the document's `nested_blocks`
// on `resolved_context`, which is `"paragraph"` for the sample's one paragraph.
#[test]
fn paragraph_blocks_can_be_found_by_context() {
    verifies!(
        r#"
Going deeper, you can find blocks in the document, such as all the paragraph blocks, using the `find_by` method:

[,ruby]
----
puts doc.find_by context: :paragraph
# => #<Asciidoctor::Block@1001 {context: :paragraph, content_model: :simple, style: nil, lines: 1}>
----

"#
    );

    let doc = load(SAMPLE);

    let paragraphs = doc
        .nested_blocks()
        .filter(|block| block.resolved_context().as_ref() == "paragraph")
        .count();
    assert_eq!(paragraphs, 1);
}

non_normative!(
    r#"
However, if you're only interested in converting the AsciiDoc source when using the API, then it's better to use the `convert_file` entrypoint.

"#
);

// Converting parses and converts the document to an output format. Like the
// CLI, this crate produces HTML by default — in fact, HTML5 is the only backend
// it provides — so `convert_file` renders a complete HTML5 document.
#[test]
fn converting_produces_html_by_default() {
    verifies!(
        r#"
== Convert an AsciiDoc file

When you convert AsciiDoc using the API, you're telling Asciidoctor to parse and convert the document to the output format determined by the specified backend.
If you don't specify a backend, like with the CLI, Asciidoctor will produce HTML.

"#
    );

    let html = convert_sample_file("default-backend");

    assert!(html.starts_with("<!DOCTYPE html>"));
    assert!(html.contains("<title>Document Title</title>"));
}

// `convert_file` converts a source file to HTML5. This crate's `convert_file`
// reads and renders the file, returning the HTML5 document. The Ruby `safe:`
// option has no counterpart in this call.
#[test]
fn convert_file_renders_the_source_file_to_html5() {
    verifies!(
        r#"
Let's again assume we're working with the following AsciiDoc document:

._document.adoc_
[,asciidoc]
----
= Document Title

The main content.
----

To convert this source file to HTML5, use the following API call:

[,ruby]
----
Asciidoctor.convert_file 'document.adoc', safe: :safe
----

"#
    );

    let html = convert_sample_file("to-html5");

    assert!(html.contains("<title>Document Title</title>"));
    assert!(html.contains("The main content."));
}

// The `:to_file` option writes the output to a chosen file. This crate's
// `convert_file` returns the HTML as a `String` and never writes a file, so
// there is no `:to_file` counterpart: the caller writes the returned string, or
// uses the `adoc` CLI to write a derived or chosen output file. The span is
// therefore non-normative.
non_normative!(
    r#"
The command will output HTML to the file [.path]_my-sample.html_ in the same directory.
If you want Asciidoctor to output to a different file, you can specify it using the `:to_file` option:

[,ruby]
----
Asciidoctor.convert_file 'document.adoc', safe: :safe, to_file: 'out.html'
----

"#
);

// Selecting the DocBook backend. This crate renders HTML5 only; DocBook and the
// other Asciidoctor backends are not planned, so there is no `:backend` option
// (nor the `:to_file` option paired with it here). The span is non-normative.
non_normative!(
    r#"
You can convert the file to DocBook by setting the `:backend` option to `'docbook'`:

[,ruby]
----
Asciidoctor.convert_file 'document.adoc', safe: :safe, backend: 'docbook'
----

In this case, Asciidoctor will output DocBook to the file [.path]_my-sample.xml_ in the same directory.
As before, you can use the `:to_file` option to control the output file.

[,ruby]
----
Asciidoctor.convert_file 'document.adoc', safe: :safe, backend: 'docbook', to_file: 'out.html'
----

"#
);

non_normative!(
    r#"
That covers the basics of loading and converting AsciiDoc using the API.
"#
);
