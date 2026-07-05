use std::fs;

use crate::{convert, convert_file, tests::sdd::*};

track_file!("docs/modules/ROOT/pages/convert-your-first-file.adoc");

// This crate's "Convert Your First File" walkthrough. It is descriptive
// documentation of the `adoc` CLI workflow, so its prose is tracked as
// non-normative. The one thing this library verifies is the conversion the
// walkthrough performs — `adoc my-document.adoc` renders the file to a complete,
// standalone HTML5 document — exercised through the closest library API,
// `convert_file` (see `converts_a_file_to_html5`). The `adoc` crate verifies the
// CLI's output-file derivation and its `-o -` form; the sdd tool merges the two.

non_normative!(
    r#"
= Convert Your First AsciiDoc File
:navtitle: Convert Your First File
:description: How to run the adoc command on an AsciiDoc document and convert it to HTML5.

On this page, you'll learn how to run `adoc` on an AsciiDoc document and convert
it to HTML5.

[NOTE]
====
The prose on this page is non-normative documentation. The `adoc` invocations it
shows are normative: they are verified against the implementation, so the
documented behavior is guaranteed.
====

== Generate HTML5 using the default converter

Let's convert an AsciiDoc document to HTML5 using `adoc`.

. To follow along with the steps below, copy the contents of the example into a
new plain text file, or use your own AsciiDoc document.
+
.my-document.adoc
[,asciidoc]
----
= My First Document

Converting AsciiDoc to HTML5 with adoc takes a single command.

== Getting Started

This section becomes an HTML section in the output.
----

. Make sure to save the file with the _.adoc_ file extension.
. Open a terminal and switch (`cd`) into the directory where your AsciiDoc
document is saved.
+
 $ cd directory-name

. Call `adoc` with the file name of the AsciiDoc document.
Since HTML5 is the only output `adoc` produces, you don't need to specify a
converter.
+
--
"#
);

// The conversion the walkthrough performs. `adoc my-document.adoc` renders the
// file to a complete HTML5 document; `convert_file` is the library call that
// reads a document from disk and returns exactly that.
#[test]
fn converts_a_file_to_html5() {
    verifies!(
        r#"
 $ adoc my-document.adoc
"#
    );

    // The example document the page uses — a title, a paragraph, and a section.
    // Reading it with `convert_file` yields the same complete, standalone HTML5
    // document that `convert` produces for the same source.
    let source = "= My First Document\n\n\
        Converting AsciiDoc to HTML5 with adoc takes a single command.\n\n\
        == Getting Started\n\n\
        This section becomes an HTML section in the output.\n";
    let path = std::env::temp_dir().join(format!(
        "asciidoc-html5-first-file-{}.adoc",
        std::process::id()
    ));
    fs::write(&path, source).expect("write temp input");
    let html = convert_file(&path).expect("convert_file reads and renders");
    let _ = fs::remove_file(&path);

    assert_eq!(html, convert(source));
    assert!(html.starts_with("<!DOCTYPE html>"));
    assert!(html.contains("<title>My First Document</title>"));
    assert!(html.trim_end().ends_with("</body>\n</html>"));
}

non_normative!(
    r#"

As long as the document can be read, `adoc` prints no messages to your terminal.
--

. Type `ls` to list the files in the directory.
+
--
 $ ls
 my-document.adoc  my-document.html

You should see a new file named [.path]_my-document.html_.
`adoc` derives the name of the output file from the name of the input document,
replacing the _.adoc_ extension with _.html_.
--

. Open [.path]_my-document.html_ in your web browser.
The converted document is a complete, standalone HTML5 document you can publish.

To write the HTML5 to standard output instead of a file -- for example, to pipe
it into another program -- pass `-o -`:

 $ adoc my-document.adoc -o -

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
"#
);
