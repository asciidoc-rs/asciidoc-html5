use std::fs;

use asciidoc_parser::blocks::IsBlock;

use crate::{convert, convert_document, convert_file, convert_with, load, tests::sdd::*, Options};

track_file!("docs/modules/api/pages/convert-strings.adoc");

// This crate's own "Load and Convert Strings Using the API" page. The prose is
// descriptive documentation, tracked as non-normative; the Rust snippets it
// shows are verified by the tests below, each driving the same API the snippet
// demonstrates against the page's sample string. The page is entirely about the
// `asciidoc_html5` (and `asciidoc_parser`) API, so — like the other API pages —
// it is tracked only from this crate.

// The bare AsciiDoc string used throughout the page.
const SAMPLE: &str = "*This* is Asciidoctor.";

// The converted body the page shows. The string entry points `convert` and
// `convert_document` return exactly this embedded fragment; `convert_file`
// returns a standalone document with the fragment nested inside it.
const FRAGMENT: &str =
    "<div class=\"paragraph\">\n<p><strong>This</strong> is Asciidoctor.</p>\n</div>";

non_normative!(
    r#"
= Load and Convert AsciiDoc Strings Using the API
:navtitle: Load and Convert Strings
:description: How to load an AsciiDoc string into a document model and convert an AsciiDoc string to HTML5 with the asciidoc_html5 Rust API.

This page explains how to load and convert AsciiDoc strings using the API. A
string is the bare AsciiDoc content -- often the contents of a file you have
already read into memory.

[NOTE]
====
The prose on this page is non-normative documentation. The API calls it shows are
normative: they are verified against the implementation, so the documented
behavior is guaranteed.
====

"#
);

// `load` parses the string into a document model with the document's block
// structure.
#[test]
fn load_parses_a_string_into_a_document() {
    verifies!(
        r#"
== Load an AsciiDoc string

Loading parses the source into a document model without converting it.
`asciidoc-html5` relies on https://crates.io/crates/asciidoc-parser[`asciidoc-parser`]
for this step, which returns an
https://docs.rs/asciidoc-parser/latest/asciidoc_parser/document/struct.Document.html[`asciidoc_parser::Document`]
carrying the document's full block structure.

To parse an AsciiDoc string into that document model, call `load`:

[,rust]
----
let doc = asciidoc_html5::load("*This* is Asciidoctor.");
----

"#
    );

    let doc = load(SAMPLE);
    assert!(doc.nested_blocks().next().is_some());
}

// The loaded document is inspectable — here by counting its top-level blocks
// through `nested_blocks`.
#[test]
fn the_loaded_document_can_be_inspected() {
    verifies!(
        r#"
Using the `doc` value, you can inspect the parsed document -- for example, count
its top-level blocks:

[,rust]
----
use asciidoc_parser::blocks::IsBlock;

assert!(doc.nested_blocks().next().is_some());
----

"#
    );

    let doc = load(SAMPLE);
    assert!(doc.nested_blocks().next().is_some());
}

non_normative!(
    r#"
If you hold the source in a file rather than in memory, read the file and pass
its contents to `load`, or call `load_file` to read and parse it in one step.

"#
);

// `convert_document` renders a document you already hold to HTML5.
#[test]
fn convert_document_renders_a_loaded_document() {
    verifies!(
        r#"
Once you have a loaded document, render it to HTML5 with `convert_document`:

[,rust]
----
let html = asciidoc_html5::convert_document(&doc);
----

"#
    );

    let doc = load(SAMPLE);
    assert!(convert_document(&doc).contains(FRAGMENT));
}

non_normative!(
    r#"
However, if you're only interested in converting the AsciiDoc source, then it's
better to use the `convert` entry point.

"#
);

// `convert` parses and renders the string in one call; the rendered HTML
// contains the converted body fragment the page shows.
#[test]
fn convert_renders_a_string_to_html() {
    verifies!(
        r#"
== Convert an AsciiDoc string

Converting parses and renders the source in a single step. To convert an
AsciiDoc string directly to HTML5, call `convert`:

[,rust]
----
let html = asciidoc_html5::convert("*This* is Asciidoctor.");
----

The rendered HTML contains the converted body:

[,html]
----
<div class="paragraph">
<p><strong>This</strong> is Asciidoctor.</p>
</div>
----

"#
    );

    let html = convert(SAMPLE);
    assert!(html.contains(FRAGMENT));
}

// `convert_file` reads, parses, and renders a file to HTML5 in one call.
#[test]
fn convert_file_reads_and_renders_a_file() {
    verifies!(
        r#"
If your source is in a file, call `convert_file`, which reads the file, parses
it, and returns the rendered HTML5 as a `String`:

[,rust]
----
let html = asciidoc_html5::convert_file("document.adoc")?;
----

"#
    );

    let path = std::env::temp_dir().join(format!(
        "asciidoc-html5-docs-convert-strings-{}.adoc",
        std::process::id()
    ));
    fs::write(&path, SAMPLE).expect("write temp input");

    let html = convert_file(&path).expect("convert_file reads and renders");
    let _ = fs::remove_file(&path);

    assert!(html.contains(FRAGMENT));
}

// The output mode follows the entry point: the string entry points return
// embedded (body-only) output, `convert_file` returns a standalone document,
// and `Options::standalone`/`embedded` choose the mode explicitly.
#[test]
fn embedded_and_standalone_output() {
    verifies!(
        r#"
== Embedded and standalone output

Matching Asciidoctor, the output mode follows the entry point. The string entry
points -- `convert` and `convert_document` -- return _embedded_ output: just the
converted body shown above, with no `+++<!DOCTYPE>+++`, `<head>`, stylesheet, or
header/footer frame. This is what you want when the HTML is destined for a
surrounding template. The file entry point `convert_file` returns a _standalone_
document instead: a complete `+++<!DOCTYPE html>+++` file with `<html>`, a
`<head>`, and a `<body>` whose content mirrors Asciidoctor's default `html5`
backend.

To choose the mode explicitly, build an `Options` and convert with `convert_with`:
`Options::standalone(true)` returns a complete document from a string, and
`Options::embedded(true)` returns body-only output from a file. When the document
has a title, embedded output includes its `<h1>` only if the `showtitle`
attribute is set.

[,rust]
----
use asciidoc_html5::{convert_with, Options};

let opts = Options::new().standalone(true);
let html = convert_with("*This* is Asciidoctor.", &opts);
assert!(html.starts_with("<!DOCTYPE html>"));
----

"#
    );

    // The string entry points return embedded output: exactly the fragment.
    let embedded = convert(SAMPLE);
    assert_eq!(embedded.trim_end(), FRAGMENT);
    assert!(!embedded.starts_with("<!DOCTYPE html>"));

    // `convert_file` returns a standalone document with the fragment inside.
    let path = std::env::temp_dir().join(format!(
        "asciidoc-html5-docs-convert-strings-modes-{}.adoc",
        std::process::id()
    ));
    fs::write(&path, SAMPLE).expect("write temp input");
    let standalone_file = convert_file(&path).expect("convert_file reads and renders");
    let _ = fs::remove_file(&path);
    assert!(standalone_file.starts_with("<!DOCTYPE html>"));
    assert!(standalone_file.contains(FRAGMENT));

    // The exact example from the page: `standalone(true)` forces a full document.
    let opts = Options::new().standalone(true);
    let html = convert_with(SAMPLE, &opts);
    assert!(html.starts_with("<!DOCTYPE html>"));

    // `embedded(true)` forces body-only output for a file, too.
    let forced_embedded = convert_with(SAMPLE, &Options::new().embedded(true));
    assert!(!forced_embedded.starts_with("<!DOCTYPE html>"));

    // With a title, embedded output shows the doctitle `<h1>` only under
    // `showtitle`.
    let with_title = convert_with("= Doc\n\nBody.", &Options::new().set("showtitle"));
    assert!(with_title.contains("<h1>Doc</h1>"));
    assert!(!convert("= Doc\n\nBody.").contains("<h1>"));
}

non_normative!(
    r#"
[NOTE]
.Known limitation
====
Asciidoctor can also return the inline markup only (its `doctype: 'inline'`
option) and convert to other backends such as DocBook. This renderer models only
the `article` doctype and produces HTML5 only, so inline-only output is not
available, and DocBook and the other Asciidoctor backends are not planned.
====

That covers the basics of loading and converting AsciiDoc strings using the API.
"#
);
