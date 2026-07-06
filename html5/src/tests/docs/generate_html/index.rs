use std::fs;

use crate::{convert, convert_file, tests::sdd::*};

track_file!("docs/modules/generate-html/pages/index.adoc");

// This crate's "Generate HTML" page. It documents that HTML5 is the only output
// `asciidoc-html5` produces and walks through converting a document with `adoc`
// and with the API. Its prose is non-normative, but two API claims are verified
// here: that converting a document always yields a complete HTML5 document (see
// `html5_is_the_only_output`), and that `convert_file` reads a file from disk
// and renders it (see `converts_a_file_to_html5`). The `adoc` crate verifies
// the CLI invocations — the derived `.html` output name and the `-o -` preview
// — and the sdd tool merges the two crates' coverage. The "XHTML is not
// supported" section states a deliberate non-feature, so it carries no rule to
// verify and is non-normative in both crates.

non_normative!(
    r#"
= Generate HTML from AsciiDoc
:navtitle: Generate HTML
:description: How asciidoc-html5 converts AsciiDoc to HTML5, the only output format it produces.

"#
);

// The page's central claim: HTML5 is the only output the renderer produces, so
// converting any document yields a complete, standalone HTML5 document. The API
// takes no backend or output-format parameter — `convert` has one behavior — so
// this holds unconditionally.
#[test]
fn html5_is_the_only_output() {
    verifies!(
        r#"
HTML5 is the only output format `asciidoc-html5` produces.
Whether you use the `adoc` command or the Rust API, converting an AsciiDoc
document gives you back a complete, standalone HTML5 document.
This page explains how to generate that HTML5 and how the renderer relates to
Asciidoctor's `html5` backend.

"#
    );

    let html = convert("= Hello\n\nWorld.");
    assert!(html.starts_with("<!DOCTYPE html>"));
    assert!(html.contains("<title>Hello</title>"));
    assert!(html.contains("<p>World.</p>"));
    assert!(html.trim_end().ends_with("</body>\n</html>"));
}

non_normative!(
    r#"
[NOTE]
====
The prose on this page is non-normative documentation. The `adoc` and API
invocations it shows are normative: they are verified against the
implementation, so the documented behavior is guaranteed.
====

== Backend and converter

Asciidoctor selects an output format through a _backend_, and its default
backend, `html5`, produces HTML5. `asciidoc-html5` has no backend switch,
because HTML5 is the only format it produces. That output corresponds to
Asciidoctor's `html5` backend and aims to be compatible with it.

[horizontal]
Output format:: HTML5
Output file extension:: _.html_
Compatibility target:: Asciidoctor's default `html5` backend

[NOTE]
.Known limitation
====
Asciidoctor embeds its default stylesheet in the HTML output, so the page is
styled without any external files. This renderer does not embed a stylesheet
yet, so the converted document is currently unstyled: the output carries the
same structure Asciidoctor produces but no default theme. Embedding the default
stylesheet is tracked in
https://github.com/asciidoc-rs/asciidoc-html5/issues/27[issue #27].
====

== Generate HTML5

In this section, we'll create a sample document, then convert it with `adoc`.

=== Create and save an AsciiDoc document

. To follow along, copy the contents of the example into a new plain text file,
or use your own AsciiDoc document.
+
.my-document.adoc
[,asciidoc]
----
= The Dangers of Wolpertingers

Don't worry about gumberoos or splintercats.
Something far more fearsome plagues the days, nights, and inbetweens.
Wolpertingers.

== Origins

Wolpertingers are ravenous beasts.
----

. Make sure to save the file with the _.adoc_ file extension.

=== Convert the document to HTML5

To convert [.path]_my-document.adoc_ to HTML5 from the command line:

. Open a terminal and switch (`cd`) into the directory that contains the
document.
. Call `adoc` with the file name of the document.
+
--
 $ adoc my-document.adoc

Since HTML5 is the only output `adoc` produces, you don't need to select a
converter.
--

. `adoc` prints nothing to the terminal on success. Type `ls` to list the
directory.
+
--
 $ ls
 my-document.adoc  my-document.html

You should see a new file named [.path]_my-document.html_. `adoc` derives the
output file name from the input, replacing the _.adoc_ extension with _.html_.
--

. Open [.path]_my-document.html_ in your web browser. The converted document is
a complete, standalone HTML5 document you can publish.

To preview the HTML5 in the terminal instead of writing a file, pass `-o -` to
write to standard output and pipe it into a terminal browser:

 $ adoc my-document.adoc -o - | w3m -T text/html

== Generate the HTML5 from the API

"#
);

// The API path the page shows: `asciidoc_html5::convert_file` reads a file from
// disk and renders it to a complete HTML5 document. Drive it on the page's
// example document and confirm it matches what `convert` produces for the same
// source.
#[test]
fn converts_a_file_to_html5() {
    verifies!(
        r#"
The Rust API produces the same HTML5. Convert a file on disk with `convert_file`:

[,rust]
----
let html = asciidoc_html5::convert_file("my-document.adoc")?;
----

"#
    );

    let source = "= The Dangers of Wolpertingers\n\n\
        Don't worry about gumberoos or splintercats.\n\
        Something far more fearsome plagues the days, nights, and inbetweens.\n\
        Wolpertingers.\n\n\
        == Origins\n\n\
        Wolpertingers are ravenous beasts.\n";
    let path = std::env::temp_dir().join(format!(
        "asciidoc-html5-generate-html-{}.adoc",
        std::process::id()
    ));
    fs::write(&path, source).expect("write temp input");
    let html = convert_file(&path).expect("convert_file reads and renders");
    let _ = fs::remove_file(&path);

    assert_eq!(html, convert(source));
    assert!(html.starts_with("<!DOCTYPE html>"));
    assert!(html.contains("<title>The Dangers of Wolpertingers</title>"));
    assert!(html.trim_end().ends_with("</body>\n</html>"));
}

non_normative!(
    r#"
`convert`, for AsciiDoc you already hold in memory, and `convert_document`, for
a document you have already parsed, return the same complete HTML5 document. See
the xref:ROOT:index.adoc[introduction] for those forms.

== XHTML is not supported

Asciidoctor can emit the XML variant of HTML, called XHTML, through its `xhtml`
and `xhtml5` backends. `asciidoc-html5` does not support XHTML: it emits HTML5
syntax only, and there is no option to switch it to XHTML. If you need XHTML
output, use https://asciidoctor.org[Asciidoctor] itself.
"#
);
