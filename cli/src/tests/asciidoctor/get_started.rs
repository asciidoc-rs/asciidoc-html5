use clap::Parser as _;

use crate::{run, tests::sdd::*, Cli};

track_file!("ref/asciidoctor/docs/modules/get-started/pages/index.adoc");

// Asciidoctor's "Convert Your First File" tutorial, tracked from the CLI crate.
// Most of it is tutorial prose — installing Asciidoctor, an included example
// document, a screenshot, and the embedded default stylesheet — with no
// rendering rule to verify, so it is non-normative here. The testable part is
// the CLI workflow: `asciidoctor my-document.adoc` converts the file and writes
// its output to _my-document.html_, a name derived from the input. This crate's
// `adoc` command mirrors that derivation (see `converts_and_derives_output_file`).
// The `asciidoc-html5` crate verifies the conversion via `convert_file`, and the
// sdd tool merges the two crates' coverage.

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

// The core of the tutorial: `asciidoctor my-document.adoc` converts the file
// silently and, with no `-o`, derives the output file name _my-document.html_
// from the input. This crate's `adoc` command mirrors that behavior, which this
// test drives end to end.
#[test]
fn converts_and_derives_output_file() {
    // Step 4: run the command. Nothing is printed on success.
    verifies!(
        r#"
 $ asciidoctor my-document.adoc
"#
    );

    non_normative!(
        r#"

As long as the document didn't contain any syntax errors, you won't see any messages printed to your terminal.
--

. Type `ls` to list the files in the directory.
+
--
"#
    );

    // Step 5: listing the directory shows the derived _my-document.html_ file,
    // whose name Asciidoctor takes from the input document.
    verifies!(
        r#"
 $ ls
 my-document.adoc  my-document.html

You should see a new file named [.path]_my-document.html_.
Asciidoctor derives the name of the output file from the name of the input document.
"#
    );

    // Drive the closest equivalent of `asciidoctor my-document.adoc`: hand the
    // `adoc` CLI a `<name>.adoc` file with no `-o`, and confirm it derives
    // `<name>.html` (as `asciidoctor` derives `my-document.html`), writes a
    // complete HTML5 document there, and prints nothing.
    let source = "= Hello\n\nWorld.";
    let path = std::env::temp_dir().join(format!("adoc-get-started-{}.adoc", std::process::id()));
    let derived = path.with_extension("html");
    std::fs::write(&path, source).expect("write temp input");

    let cli = Cli::parse_from(["adoc", path.to_str().expect("temp path is UTF-8")]);
    let mut stdout = Vec::new();
    run(&cli, &mut stdout).expect("adoc converts the file");

    assert!(stdout.is_empty(), "adoc wrote to stdout on success");
    // The output landed in the derived `.html` file, not anywhere else.
    assert_eq!(derived, path.with_extension("html"));
    let html = std::fs::read_to_string(&derived).expect("read derived output file");
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_file(&derived);

    assert!(html.starts_with("<!DOCTYPE html>"));
    assert!(html.contains("<title>Hello</title>"));
    assert!(html.trim_end().ends_with("</body>\n</html>"));
}

non_normative!(
    r#"
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
