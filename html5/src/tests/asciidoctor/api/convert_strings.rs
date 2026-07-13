use std::fs;

use asciidoc_parser::blocks::IsBlock;

use crate::{convert, convert_document, convert_file, load, tests::sdd::*};

track_file!("ref/asciidoctor/docs/modules/api/pages/convert-strings.adoc");

// Asciidoctor's "Load and Convert Strings Using the API" page, tracked from the
// library crate. It documents the string forms of two Ruby entrypoints:
// `Asciidoctor.load`, which parses a string into an `Asciidoctor::Document`,
// and `Asciidoctor.convert`, which parses and converts a string to an output
// format. This crate has a direct analog for each: `load` parses a string into
// a `Document`, `convert` parses and converts one, and `convert_document`
// renders a document you already hold (the analog of `doc.convert`).
// `convert_file` is the file counterpart the page reaches for after
// `File.read`.
//
// The page's second half describes behavior this crate deliberately does not
// share, so it is non-normative. Asciidoctor makes an *embedded* document the
// default for string conversion and gates a standalone document behind the
// `:standalone` option; this crate has no embedded output at all — every entry
// point returns a complete, standalone HTML5 document — so the "Embedded
// output" and "Standalone output" sections, the `:standalone`/`:to_file`
// options, and the embedded-by-default statement are all non-normative. Adding
// embedded output is tracked in
// https://github.com/asciidoc-rs/asciidoc-html5/issues/68. The remaining two
// sections describe outputs this crate does not produce either: `doctype:
// 'inline'` (this renderer models only the `article` doctype) and the DocBook
// backend (HTML5 is the only backend; DocBook is not planned). The Ruby `safe:`
// option has no bearing on loading or converting here.

// The bare AsciiDoc string used throughout the page.
const SAMPLE: &str = "*This* is Asciidoctor.";

// The converted body the page shows as Asciidoctor's embedded output. This
// crate wraps the same fragment in a standalone shell rather than returning it
// on its own, so the fragment appears *within* every `convert` result.
const FRAGMENT: &str =
    "<div class=\"paragraph\">\n<p><strong>This</strong> is Asciidoctor.</p>\n</div>";

non_normative!(
    r#"
= Load and Convert Strings Using the API
:navtitle: Load and Convert Strings

This page explains how to load and convert AsciiDoc strings using the API.
A string is the bare AsciiDoc content (often the contents of a file).

"#
);

// `Asciidoctor.load` parses a string into a document model. This crate's `load`
// is the direct analog: it parses the string into a `Document` carrying the
// document's block structure.
#[test]
fn load_parses_a_string_into_a_document() {
    verifies!(
        r#"
== Load an AsciiDoc string

To parse an AsciiDoc string into a document object model, use:

[,ruby]
----
doc = Asciidoctor.load '*This* is Asciidoctor.'
----

"#
    );

    let doc = load(SAMPLE);

    // The parsed document exposes its block structure.
    assert!(doc.nested_blocks().next().is_some());
}

// Reading the source from a file and loading it. Asciidoctor reads the file
// with `File.read` and passes the resulting string to `load`; this crate does
// the same — read the file, then `load` its contents into a `Document`. The
// Ruby `safe: :safe` option does not affect loading here.
#[test]
fn a_string_read_from_a_file_can_be_loaded() {
    verifies!(
        r#"
You can also read AsciiDoc from a file and pass it to the `load` method:

[,ruby]
----
asciidoc = File.read 'document.adoc', mode: 'r:utf-8'
doc = Asciidoctor.load asciidoc, safe: :safe
----

"#
    );

    let path = std::env::temp_dir().join(format!(
        "asciidoc-html5-api-convert-strings-load-{}.adoc",
        std::process::id()
    ));
    fs::write(&path, SAMPLE).expect("write temp input");

    // Read the file, then load its contents — the two steps the snippet shows.
    let asciidoc = fs::read_to_string(&path).expect("read temp input");
    let _ = fs::remove_file(&path);

    let doc = load(&asciidoc);
    assert!(doc.nested_blocks().next().is_some());
}

// `doc.convert` renders a document you have already loaded. This crate's
// counterpart is `convert_document`, which renders the loaded `Document` to
// HTML5.
#[test]
fn a_loaded_document_is_rendered_with_convert_document() {
    verifies!(
        r#"
Once you have loaded the document, you can convert it by calling the convert method:

[,ruby]
-----
doc.convert
-----

"#
    );

    let doc = load(SAMPLE);
    let html = convert_document(&doc);
    assert!(html.contains(FRAGMENT));
}

non_normative!(
    r#"
However, if you're only interested in converting the AsciiDoc source when using the API, then it's better to use a convert entrypoint.

"#
);

// `Asciidoctor.convert` parses and converts a string directly. This crate's
// `convert` is the direct analog. The page shows Asciidoctor's *embedded*
// output — just the converted body — whereas `convert` returns a complete
// standalone document; the same body fragment appears within it, so we verify
// the fragment is present. (The embedded-vs-standalone difference is the
// subject of the non-normative sections that follow.)
#[test]
fn convert_renders_a_string_to_html() {
    verifies!(
        r#"
== Convert an AsciiDoc string

To convert the AsciiDoc string directly to HTML, use:

[,ruby]
----
puts Asciidoctor.convert '*This* is Asciidoctor.'
----

Here's the output you will see:

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

// Reading the source from a file and converting it. Asciidoctor reads the file
// with `File.read` and passes the string to `convert`; this crate's file
// counterpart, `convert_file`, reads and converts the file in one call. The
// Ruby `safe:` option has no counterpart in this call.
#[test]
fn a_string_read_from_a_file_can_be_converted() {
    verifies!(
        r#"
You can also read AsciiDoc from a file and pass it to the `convert` method:

[,ruby]
----
asciidoc = File.read 'document.adoc', mode: 'r:utf-8'
html = Asciidoctor.convert asciidoc, safe: :safe
----

"#
    );

    let path = std::env::temp_dir().join(format!(
        "asciidoc-html5-api-convert-strings-convert-{}.adoc",
        std::process::id()
    ));
    fs::write(&path, SAMPLE).expect("write temp input");

    let html = convert_file(&path).expect("convert_file reads and renders");
    let _ = fs::remove_file(&path);

    assert!(html.contains(FRAGMENT));
}

// The embedded-by-default statement. This crate always outputs a standalone
// document, so it has no embedded default to explain; the span is
// non-normative. Adding embedded output is tracked in
// https://github.com/asciidoc-rs/asciidoc-html5/issues/68.
non_normative!(
    r#"
When converting a string, Asciidoctor _does not_ output a standalone document by default.
Instead, it generates embedded output.
Let's learn why that is and how to control it.

"#
);

// The "Embedded output" section describes the embedded document this crate does
// not produce — the `:standalone` option, the pieces an embedded document
// includes, and its intended use in a template. All non-normative here (see
// issue #68).
non_normative!(
    r#"
== Embedded output

When you pass an AsciiDoc string to `Asciidoctor.convert` to convert it to a backend format, such as HTML, the `:standalone` option is `false` by default.
That means this method only returns the converted content.
This content does not include the frame around that content (i.e., the header and footer) that's included in a standalone document.
In other words, it makes an _embedded_ document.
This default was chosen to make Asciidoctor consistent with other lightweight markup processors like Markdown.

Here's what's included in an embedded document:

* The document title if the `showtitle` attribute is set
* The table of contents if the `toc` attribute is set and the value is not `preamble`
* The converted document body
* The footnotes unless the `nofootnotes` attribute is set

The author and revision information is never shown in an embedded document.
If you need to display that information, you can use attribute references such as `\{author}` and `\{revnumber}`.

The embedded document is intended to be included in a template, such as one provided by a static site generator.
That template is responsible for providing the styles and library integrations needed for the content to render properly.

"#
);

// The "Standalone output" section is framed around opting *in* to a standalone
// document with `:standalone`, and around the `:to_file` option — neither of
// which this crate has, since a standalone document is all it produces. The
// section (and its `toc::[]` note) is non-normative.
non_normative!(
    r#"
== Standalone output

You can still generate a standalone document when converting a string.
To convert from an AsciiDoc string to a standalone output document, you need to explicitly set the `:standalone` option to `true`.

[,ruby]
----
puts Asciidoctor.convert '*This* is Asciidoctor.', standalone: true
----

Now you'll get a complete HTML file.
The standalone output provides the framing around the content, which includes the styling and all the library integrations the content needs to properly render (e.g., the default stylesheet, MathJax, etc.).
If you don't set the `:standalone` option to `true`, you only get the embedded document (i.e., body content).

When the input or output is a file, the `:standalone` option is enabled by default.
Thus, to instruct Asciidoctor to write standalone HTML to a file from an AsciiDoc string, the `:to_file` option is mandatory.

[,ruby]
----
Asciidoctor.convert '*This* is Asciidoctor.', to_file: 'out.html'
----

If you want to generate embedded output when starting with a file, set the `:standalone` option to `false`.
However, most of the time you'll want to generate a standalone document when converting a file (which is why it's default).

When converting a string, the TOC is only included by default when using the `:standalone` option as shown above (whether it's enabled implicitly or explicitly).
However, you can force it to be included without the header and footer by setting the `toc` attribute with a value of `macro` and using the `toc::[]` macro in the string itself.

"#
);

// `doctype: 'inline'` returns only the inline markup. This renderer models only
// the `article` doctype (it pins `doctype` to `article`), so there is no
// inline-only mode; the span is non-normative.
non_normative!(
    r#"
== Convert inline markup only

If you only want the inline markup to be returned, set the `:doctype` option to `'inline'`:

[,ruby]
----
puts Asciidoctor.convert '*This* is Asciidoctor.', doctype: 'inline'
----

In this mode, Asciidoctor will only process the first block (e.g., paragraph) in the document and ignore the rest.

"#
);

// The DocBook backend. This crate renders HTML5 only; DocBook and the other
// Asciidoctor backends are not planned, so there is no `:backend` option (nor
// the `:standalone` option paired with it here). The span is non-normative.
non_normative!(
    r#"
== Convert to DocBook

You can produce DocBook 5.0 by setting the `:backend` option to `'docbook'`.
Since embedded DocBook isn't that useful, we also enable the standalone document (i.e., header and footer) by setting the `:standalone` option to `true`.

[,ruby]
----
puts Asciidoctor.convert '*This* is Asciidoctor.', standalone: true, backend: 'docbook'
----
"#
);
