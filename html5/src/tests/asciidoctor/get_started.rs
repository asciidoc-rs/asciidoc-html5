use std::fs;

use crate::{convert, convert_file, tests::sdd::*};

track_file!("ref/asciidoctor/docs/modules/get-started/pages/index.adoc");

// Asciidoctor's "Convert Your First File" tutorial. It walks a reader through
// running the `asciidoctor` CLI on a document and finding the derived HTML
// output. Almost all of it is CLI tutorial prose — installing Asciidoctor, an
// included example document, a screenshot, and the embedded default stylesheet
// — none of which describes a library rule this crate can verify, so it is
// non-normative here. The one thing with a counterpart in this crate is the
// conversion itself: `asciidoctor my-document.adoc` produces a complete HTML5
// document. This crate verifies that against its closest available API,
// `convert_file` (see `converts_a_file_to_html5`); the `adoc` crate verifies
// the CLI's output-file derivation, and the sdd tool merges the two.

non_normative!(
    r#"
= Convert Your First AsciiDoc File
:navtitle: Convert Your First File

Assumptions:

* [x] You've installed Asciidoctor.
* [x] You've confirmed that the Asciidoctor command line interface (CLI) is available on your PATH.

On this page, you'll learn how to run Asciidoctor on an AsciiDoc document and convert it to HTML.

== Generate HTML using the default converter

Let's generate HTML 5 using Asciidoctor's default converter and stylesheet from an AsciiDoc document.

. To follow along with the steps below, copy the contents of <<ex-my-doc>> into a new plain text file or use your own AsciiDoc document.
+
.my-document.adoc
[#ex-my-doc,asciidoc]
----
include::html-backend:example$my-document.adoc[tags=title;body]
----

. Make sure to save the file with the _.adoc_ file extension.
. Open a terminal and switch (`cd`) into the directory where your AsciiDoc document is saved.
+
 $ cd directory-name

. Call Asciidoctor with the `asciidoctor` command, followed by file name of the AsciiDoc document.
Since HTML 5 is Asciidoctor's default output, we don't need to specify a converter.
+
--
"#
);

// The conversion step: `asciidoctor my-document.adoc` reads the file and
// renders it to HTML5. This crate's closest counterpart is `convert_file`,
// which reads an AsciiDoc file from disk and returns a complete HTML5 document.
#[test]
fn converts_a_file_to_html5() {
    verifies!(
        r#"
 $ asciidoctor my-document.adoc
"#
    );

    // The `title;body` tags of the tutorial's example document — a title, a few
    // paragraphs, and a section. Reading it from disk with `convert_file` yields
    // the same complete, standalone HTML5 document that `convert` produces for
    // the same source.
    let source = "= The Dangers of Wolpertingers\n\
        :url-wolpertinger: https://en.wikipedia.org/wiki/Wolpertinger\n\n\
        Don't worry about gumberoos or splintercats.\n\
        Something far more fearsome plagues the days, nights, and inbetweens.\n\
        Wolpertingers.\n\n\
        == Origins\n\n\
        Wolpertingers are {url-wolpertinger}[ravenous beasts].\n";
    let path = std::env::temp_dir().join(format!(
        "asciidoc-html5-get-started-{}.adoc",
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

As long as the document didn't contain any syntax errors, you won't see any messages printed to your terminal.
--

. Type `ls` to list the files in the directory.
+
--
 $ ls
 my-document.adoc  my-document.html

You should see a new file named [.path]_my-document.html_.
Asciidoctor derives the name of the output file from the name of the input document.
--

. Open [.path]_my-document.html_ in your web browser.
The converted document should look like the example below.
+
--
image::html-backend:my-document.png[]

The document's text, titles, and link is styled by the default Asciidoctor stylesheet, which is embedded in the HTML output.
As a result, you could save [.path]_my-document.html_ to any computer and it will look the same.
--

TIP: Most of the examples in the general documentation use the CLI, but there are usually corresponding API examples under xref:api:index.adoc[].
"#
);
