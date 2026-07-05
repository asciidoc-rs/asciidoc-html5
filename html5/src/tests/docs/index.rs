use std::fs;

use asciidoc_parser::Parser;

use crate::{convert, convert_document, convert_file, tests::sdd::*};

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

"#
);

// The "Basic usage" section, verified from the API side. The CLI column is
// verified by the `adoc` crate; the sdd tool merges the two.
#[test]
fn basic_usage() {
    // Section framing: two interfaces, and the promise that the simplest
    // case yields a complete, publishable HTML5 document.
    verifies!(
        r#"
== Basic usage

`asciidoc-html5` provides two interfaces for converting AsciiDoc documents: a
CLI named `adoc` and a Rust API in the `asciidoc_html5` crate. The following
table gives you an idea of how to use these interfaces.

|===
^|CLI ^|API

"#
    );

    // The CLI column of the table (verified by the `adoc` crate).
    non_normative!(
        r#"
a|
 $ adoc document.adoc

"#
    );

    // The API column of the table: the file-based `convert_file`.
    verifies!(
        r#"
a|
[,rust]
----
let html =
    asciidoc_html5::convert_file("document.adoc")?;
----
"#
    );

    // The CLI output description (verified by the `adoc` crate).
    non_normative!(
        r#"

|Reads `document.adoc` and writes the rendered HTML5 to _document.html_.
"#
    );

    // The API output description and the simplest-case promise.
    verifies!(
        r#"
|Reads `document.adoc` and returns the rendered HTML5 as a `String`.
|===

In the simplest case, you give an AsciiDoc document to `asciidoc-html5` and it
gives you back a complete HTML5 document you can publish.

"#
    );

    // The simplest case: `convert_file` reads the document from disk and returns
    // a complete, standalone HTML5 document — the same result the in-memory
    // `convert` entry point produces for the same source.
    let source = "= Hello\n\nWorld.";
    let path = std::env::temp_dir().join(format!(
        "asciidoc-html5-introduction-basic-usage-{}.adoc",
        std::process::id()
    ));
    fs::write(&path, source).expect("write temp input");
    let html = convert_file(&path).expect("convert_file reads and renders");
    let _ = fs::remove_file(&path);

    assert_eq!(html, convert(source));
    assert!(html.starts_with("<!DOCTYPE html>"));
    assert!(html.contains("<title>Hello</title>"));
    assert!(html.trim_end().ends_with("</body>\n</html>"));
}

non_normative!(
    r#"
Pass `--help` to the CLI to see every option:

 $ adoc --help

== API examples

The Rust API exposes three entry points. The file-based `convert_file` shown
above is the most common. The other two are `convert`, for AsciiDoc you already
hold in memory, and `convert_document`, for a document you have already parsed.
Each returns a complete, standalone HTML5 document.

"#
);

// The first "API examples" entry: the sentence introducing `convert` and
// its listing.
#[test]
fn convert_renders_in_memory_asciidoc() {
    verifies!(
        r#"
Convert AsciiDoc held in memory with `convert`:

[,rust]
----
let html = asciidoc_html5::convert("= Hello\n\nWorld.");
----
"#
    );

    // `convert` renders in-memory AsciiDoc to a complete HTML5 document.
    let html = convert("= Hello\n\nWorld.");

    assert!(html.starts_with("<!DOCTYPE html>"));
    assert!(html.contains("<title>Hello</title>"));
    assert!(html.contains("<div class=\"paragraph\">\n<p>World.</p>\n</div>"));
    assert!(html.trim_end().ends_with("</body>\n</html>"));
}

non_normative!(
    r#"

"#
);

// The second "API examples" entry: the sentence introducing
// `convert_document` and its listing.
#[test]
fn convert_document_renders_a_parsed_document() {
    verifies!(
        r#"
If you already hold a parsed document — for example, to inspect or transform it
first — render it with `convert_document`:

[,rust]
----
let doc = asciidoc_parser::Parser::default().parse("= Hello\n\nWorld.");
let html = asciidoc_html5::convert_document(&doc);
----
"#
    );

    // `convert_document` renders a document that was parsed separately, giving
    // the same result as `convert` of the same source.
    let source = "= Hello\n\nWorld.";
    let doc = Parser::default().parse(source);
    let html = convert_document(&doc);

    assert_eq!(html, convert(source));
    assert!(html.contains("<title>Hello</title>"));
}

non_normative!(
    r#"

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
