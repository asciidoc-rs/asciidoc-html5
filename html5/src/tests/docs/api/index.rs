use crate::{
    convert, convert_document, convert_file, convert_with, load, load_file, tests::sdd::*, Options,
};

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
// docs.rs equivalent of Asciidoctor's `url-api` attribute, used for the API
// reference link under Next steps. It resolves to the crate's latest rendered
// API docs; pinning it to the released crate version (as Asciidoctor pins its
// `{release-version}`) is tracked in
// https://github.com/asciidoc-rs/asciidoc-html5/issues/32.
:url-api: https://docs.rs/asciidoc-html5

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
the document's elements. `load` (for a string) and `load_file` (for a file)
perform this step, returning an
https://docs.rs/asciidoc-parser/latest/asciidoc_parser/document/struct.Document.html[`asciidoc_parser::Document`]
(the parsing itself is handled by
https://crates.io/crates/asciidoc-parser[`asciidoc-parser`]).

convert:: The document model is rendered to a complete HTML5 document.

You can run both steps together with `convert` (for a string) or `convert_file`
(for a file), or run them separately -- `load` (or `load_file`) to parse, then
`convert_document` to render the document you get back.

"#
);

// The load/convert steps: `convert` runs both together, and `load` followed by
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
let doc = asciidoc_html5::load("= Hello\n\nWorld.");
// inspect or transform `doc` here
let html = asciidoc_html5::convert_document(&doc);
----

Both paths produce the same HTML5 for the same source.

"#
    );

    let source = "= Hello\n\nWorld.";

    // Together: `convert` loads and converts the string in one call.
    let together = convert(source);

    // Separately: `load` parses into a document, then `convert_document` renders
    // it.
    let doc = load(source);
    let separately = convert_document(&doc);

    // Both paths produce the same HTML5 for the same source.
    assert_eq!(together, separately);
    assert!(together.starts_with("<!DOCTYPE html>"));
    assert!(together.contains("<title>Hello</title>"));
}

// The five entry points: `convert`/`convert_file` parse and render to a
// `String`; `load`/`load_file` parse only, returning a `Document`; and
// `convert_document` renders an already-parsed document.
#[test]
fn entry_points() {
    verifies!(
        r#"
== API entry points

The library provides five basic entry points: a _convert_ and a _load_ function
for each input, plus `convert_document` to render a document you already hold.
Each parsing entry point also has an option-aware `_with` variant, described
below.

The convert entry points parse and render in one call, each returning a complete,
standalone HTML5 document as a `String`:

`asciidoc_html5::convert`:: parses an AsciiDoc string and renders it to HTML5.
`asciidoc_html5::convert_file`:: reads an AsciiDoc file, parses it, and renders it
to HTML5, returning the HTML as a `String`.

The load entry points parse only, returning an `asciidoc_parser::Document` you can
inspect or transform before rendering:

`asciidoc_html5::load`:: parses an AsciiDoc string into a document.
`asciidoc_html5::load_file`:: reads an AsciiDoc file and parses it into a document.

And `convert_document` renders a document you already hold:

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

    // Write the source to a temp file for the `_file` entry points.
    let path = std::env::temp_dir().join(format!(
        "asciidoc-html5-docs-api-{}.adoc",
        std::process::id()
    ));
    std::fs::write(&path, source).expect("write temp input");

    // `convert_file`: reads a file and returns the rendered HTML as a `String`.
    let from_file: String = convert_file(&path).expect("convert_file reads and renders");
    assert_eq!(from_file, from_string);

    // `load`: parses a string into a document (parse only).
    let doc = load(source);
    assert_eq!(doc.doctitle(), Some("Hello"));

    // `load_file`: reads and parses a file into the same document.
    let doc_from_file = load_file(&path).expect("load_file reads and parses");
    assert_eq!(doc_from_file.doctitle(), Some("Hello"));

    // `convert_document`: renders an already-parsed document to the same HTML5.
    assert_eq!(convert_document(&doc), from_string);

    let _ = std::fs::remove_file(&path);
}

non_normative!(
    r#"
Use the `_file` entry points when your source is a file on disk and the string
forms when you already hold it in memory. Reach for `load` (or `load_file`)
followed by `convert_document` when you want to analyze or transform the document
before rendering it.

== Supplying document attributes

To set a document attribute from outside the source -- the way Asciidoctor's
`-a name=value` option does -- build an `Options` and convert with `convert_with`
(the attribute-aware counterpart of `convert`, with `convert_file_with` for a
file on disk):

"#
);

// Supplying document attributes: `convert_with` with an `Options` seeds an
// attribute from outside the source. By default it overrides a document-header
// assignment of the same name; `attribute_default` is the soft-set form the
// document can override; `set`/`unset` toggle an attribute on or off.
#[test]
fn supplying_document_attributes() {
    verifies!(
        r#"
[,rust]
----
use asciidoc_html5::{convert_with, Options};

let opts = Options::new().attribute("webfonts", "Ubuntu+Mono:400");
let html = convert_with("= Doc\n\nBody.", &opts);
assert!(html.contains("family=Ubuntu+Mono:400"));
----

By default an attribute supplied this way _overrides_ the document: it wins over
an assignment of the same name in the document header. Use `attribute_default`
(Asciidoctor's soft-set `name=value@`) to treat your value as a fallback the
document can override instead. The `set` and `unset` methods turn an attribute on
or off, matching Asciidoctor's `name` and `name!`.

"#
    );

    // The exact example from the page.
    let opts = Options::new().attribute("webfonts", "Ubuntu+Mono:400");
    let html = convert_with("= Doc\n\nBody.", &opts);
    assert!(html.contains("family=Ubuntu+Mono:400"));

    // An override wins over a document-header assignment of the same name.
    let header = "= Doc\n:webfonts: from-header\n\nBody.";
    let overridden = convert_with(header, &Options::new().attribute("webfonts", "from-api"));
    assert!(overridden.contains("family=from-api"));
    assert!(!overridden.contains("family=from-header"));

    // A soft-set default yields to the document-header assignment instead.
    let softened = convert_with(
        header,
        &Options::new().attribute_default("webfonts", "from-api"),
    );
    assert!(softened.contains("family=from-header"));

    // `set` turns an attribute on, `unset` turns it off.
    let linked = convert_with("= Doc\n\nBody.", &Options::new().set("linkcss"));
    assert!(linked.contains("<link rel=\"stylesheet\" href=\"./asciidoctor.css\">"));
    let unfonted = convert_with("= Doc\n\nBody.", &Options::new().unset("webfonts"));
    assert!(!unfonted.contains("<link rel=\"stylesheet\" href=\"https://fonts.googleapis.com"));
}

non_normative!(
    r#"
The load entry points take the same `Options`: `load_with` and `load_file_with`
are the option-aware counterparts of `load` and `load_file`, applying these
settings at parse time and returning the document without rendering it.

[NOTE]
.Known limitation
====
Asciidoctor's Ruby API converts to several backends (HTML5, DocBook, man pages,
and more) and can register extensions and write output directly to a file.
`asciidoc-html5` currently offers only HTML5 conversion, returning the rendered
HTML as a `String`; apart from the document attributes shown above, it accepts no
processing options and supports no extensions. Writing the output to a file is
the job of the `adoc` CLI.
====

== Next steps

* xref:ROOT:convert-your-first-file.adoc[Convert your first file with the CLI]
* xref:ROOT:index.adoc[Introduction and API examples]
* {url-api}[API reference on docs.rs^]
"#
);
