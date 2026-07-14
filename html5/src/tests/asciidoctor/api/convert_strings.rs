use std::fs;

use asciidoc_parser::blocks::IsBlock;

use crate::{convert, convert_document, convert_file, convert_with, load, tests::sdd::*, Options};

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
// Asciidoctor makes an *embedded* document the default for string conversion
// and gates a standalone document behind the `:standalone` option. This crate
// now matches that default: the string entry points (`convert`,
// `convert_document`) return embedded, body-only output, while `convert_file`
// returns a standalone document, and `Options::standalone`/`embedded` choose
// the mode explicitly — so the embedded-by-default statement, the "Embedded
// output" intro, and the `:standalone` opt-in in "Standalone output" are
// verified below. The pieces the page describes that this crate does not (yet)
// produce stay non-normative: the TOC and footnotes an embedded document may
// include (neither is rendered yet), the `:to_file` option, `doctype: 'inline'`
// (this renderer models only the `article` doctype), and the DocBook backend
// (HTML5 is the only backend; DocBook is not planned). The Ruby `safe:` option
// has no bearing on loading or converting here.

// The bare AsciiDoc string used throughout the page.
const SAMPLE: &str = "*This* is Asciidoctor.";

// The converted body the page shows as Asciidoctor's embedded output. Matching
// Asciidoctor, this crate's string `convert` returns exactly this fragment;
// `convert_file` wraps the same fragment in a standalone shell.
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
// `convert` is the direct analog, and — matching Asciidoctor — it returns the
// same *embedded* output the page shows: just the converted body. (The
// embedded-vs-standalone default is the subject of the sections that follow.)
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

// The embedded-by-default statement — this crate now matches it: converting a
// string returns embedded output, not a standalone document.
#[test]
fn a_string_converts_to_embedded_output_by_default() {
    verifies!(
        r#"
When converting a string, Asciidoctor _does not_ output a standalone document by default.
Instead, it generates embedded output.
Let's learn why that is and how to control it.

"#
    );

    let html = convert(SAMPLE);
    assert!(html.contains(FRAGMENT));
    assert!(!html.starts_with("<!DOCTYPE html>"));
}

// The "Embedded output" intro: a string conversion returns only the converted
// content, with no header/footer frame. This crate matches it.
#[test]
fn embedded_output_returns_only_the_converted_content() {
    verifies!(
        r#"
== Embedded output

When you pass an AsciiDoc string to `Asciidoctor.convert` to convert it to a backend format, such as HTML, the `:standalone` option is `false` by default.
That means this method only returns the converted content.
This content does not include the frame around that content (i.e., the header and footer) that's included in a standalone document.
In other words, it makes an _embedded_ document.
This default was chosen to make Asciidoctor consistent with other lightweight markup processors like Markdown.

"#
    );

    let html = convert(SAMPLE);
    assert_eq!(html.trim_end(), FRAGMENT);
    assert!(!html.contains("id=\"header\""));
    assert!(!html.contains("id=\"footer\""));
}

// The rest of the "Embedded output" section lists the pieces an embedded
// document includes and its intended use in a template. The doctitle-under-
// `showtitle` and converted-body pieces are exercised elsewhere, but the TOC
// and footnotes are not rendered yet, and the author/revision and template
// prose carry no rule to test — so this span stays non-normative.
non_normative!(
    r#"
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

// The "Standalone output" section: opting *in* to a standalone document by
// setting `:standalone` to `true`. This crate matches it with
// `Options::standalone(true)`; without it, a string conversion is embedded.
#[test]
fn standalone_output_is_opt_in_for_a_string() {
    verifies!(
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

"#
    );

    // Explicitly opting in gives a complete HTML file, with the converted body
    // framed inside it.
    let standalone = convert_with(SAMPLE, &Options::new().standalone(true));
    assert!(standalone.starts_with("<!DOCTYPE html>"));
    assert!(standalone.contains(FRAGMENT));

    // Without it, a string conversion is the embedded document (body content).
    let embedded = convert(SAMPLE);
    assert_eq!(embedded.trim_end(), FRAGMENT);
    assert!(!embedded.starts_with("<!DOCTYPE html>"));
}

// The remainder of the "Standalone output" section is about writing to a file:
// the `:to_file` option, the file-input standalone default, and the `toc::[]`
// macro. `:to_file` and the TOC macro have no counterpart here (the library
// renders to a `String`; file output is the CLI's job, and the TOC is not
// rendered yet), so this span stays non-normative.
non_normative!(
    r#"
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
