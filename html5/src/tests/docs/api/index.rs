use asciidoc_parser::Parser;

use crate::{convert, convert_document, convert_file, tests::sdd::*};

track_file!("docs/modules/api/pages/index.adoc");

// This crate's own "Process AsciiDoc Using the API" page. The prose is
// descriptive documentation, tracked as non-normative; the Rust snippets it
// shows are verified by the tests below. The page is entirely about the
// `asciidoc_html5` API, so — unlike the introduction and get-started pages — it
// is tracked only from this crate.

non_normative!(
    r#"
= Process AsciiDoc Using the API
:navtitle: Using the API
:description: How to convert and render AsciiDoc with the asciidoc_html5 Rust API, including the load and convert steps and the library's entry points.

The `asciidoc-html5` library exposes a small Rust API for converting AsciiDoc to
HTML5, both in one call and by rendering a document you have already parsed. Add
the crate to your `Cargo.toml` and call it from Rust to embed AsciiDoc conversion
in a larger tool.

[NOTE]
====
The prose on this page is non-normative documentation. The API calls it shows are
normative: they are verified against the implementation, so the documented
behavior is guaranteed.
====

== When to use the API

If all you need to do is convert an AsciiDoc file to HTML5, the
xref:ROOT:convert-your-first-file.adoc[`adoc` CLI] will suit your needs. The API is for
embedding conversion in a Rust program, where you want to keep the result in
memory, inspect the parsed document, or make conversion one step of a larger
pipeline rather than shelling out to a separate process.

== The load and convert steps

Converting AsciiDoc happens in two steps:

load:: The AsciiDoc source is parsed into a document model -- an in-memory tree of
the document's elements. `asciidoc-html5` relies on `asciidoc-parser` for this
step, which returns an `asciidoc_parser::Document`.

convert:: The document model is rendered to a complete HTML5 document.

You can run both steps together with `convert` (for a string) or `convert_file`
(for a file), or run them separately by parsing with `asciidoc-parser` and then
rendering the resulting document with `convert_document`.

"#
);

// The load/convert steps: `convert` runs both together, and parsing followed by
// `convert_document` runs them separately. Both paths agree for the same
// source.
#[test]
fn steps_together_and_separately() {
    verifies!(
        r#"
Run the two steps together with `convert`:

[,rust]
----
let html = asciidoc_html5::convert("= Hello\n\nWorld.");
----

Run them separately when you want to inspect or transform the document between
parsing and rendering:

[,rust]
----
let doc = asciidoc_parser::Parser::default().parse("= Hello\n\nWorld.");
// inspect or transform `doc` here
let html = asciidoc_html5::convert_document(&doc);
----

Both paths produce the same HTML5 for the same source.
"#
    );

    let source = "= Hello\n\nWorld.";

    // Together: `convert` loads and converts the string in one call.
    let together = convert(source);

    // Separately: parse into a document, then render it with `convert_document`.
    let doc = Parser::default().parse(source);
    let separately = convert_document(&doc);

    // Both paths produce the same HTML5 for the same source.
    assert_eq!(together, separately);
    assert!(together.starts_with("<!DOCTYPE html>"));
    assert!(together.contains("<title>Hello</title>"));
}

non_normative!(
    r#"

== API entry points

The library provides three entry points. Each returns a complete, standalone
HTML5 document.

"#
);

// The three entry points, each returning a complete, standalone HTML5 document:
// `convert` from a string, `convert_file` from a file (as a `String`), and
// `convert_document` from an already-parsed document.
#[test]
fn entry_points() {
    verifies!(
        r#"
`asciidoc_html5::convert`:: parses an AsciiDoc string and renders it to HTML5.
`asciidoc_html5::convert_file`:: reads an AsciiDoc file, parses it, and renders it
to HTML5, returning the HTML as a `String`.
`asciidoc_html5::convert_document`:: renders an already-parsed
`asciidoc_parser::Document` to HTML5.
"#
    );

    let source = "= Hello\n\nWorld.";

    // `convert`: string in, complete standalone HTML5 document out.
    let from_string = convert(source);
    assert!(from_string.starts_with("<!DOCTYPE html>"));
    assert!(from_string.contains("<title>Hello</title>"));
    assert!(from_string.trim_end().ends_with("</body>\n</html>"));

    // `convert_file`: reads a file and returns the rendered HTML as a `String`.
    let path = std::env::temp_dir().join(format!(
        "asciidoc-html5-docs-api-{}.adoc",
        std::process::id()
    ));
    std::fs::write(&path, source).expect("write temp input");
    let from_file: String = convert_file(&path).expect("convert_file reads and renders");
    let _ = std::fs::remove_file(&path);
    assert_eq!(from_file, from_string);

    // `convert_document`: renders an already-parsed document to the same HTML5.
    let doc = Parser::default().parse(source);
    assert_eq!(convert_document(&doc), from_string);
}

non_normative!(
    r#"

Use `convert_file` when your source is a file on disk and `convert` when you
already hold it in memory. Reach for `convert_document` when you have parsed the
document separately -- for example, to analyze it before rendering.

[NOTE]
.Known limitation
====
Asciidoctor's Ruby API converts to several backends (HTML5, DocBook, man pages,
and more) and can register extensions, pass processing options, and write output
directly to a file. `asciidoc-html5` currently offers only HTML5 conversion
through the three entry points above; it always returns the rendered HTML as a
`String` and accepts no options. Writing the output to a file is the job of the
`adoc` CLI.
====

== Next steps

* xref:ROOT:convert-your-first-file.adoc[Convert your first file with the CLI]
* xref:ROOT:index.adoc[Introduction and API examples]
"#
);
