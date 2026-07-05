use clap::Parser as _;

use crate::{run, tests::sdd::*, Cli};

track_file!("docs/modules/ROOT/pages/convert-your-first-file.adoc");

// This crate's "Convert Your First File" walkthrough, tracked from the CLI. Its
// prose is descriptive documentation and so non-normative, but the `adoc`
// invocations it shows are verified here: converting a file derives the
// `<name>.html` output name (see `converts_and_derives_output_file`), and `-o
// -` redirects the HTML to standard output (see `writes_to_stdout_with_dash`).
// The `asciidoc-html5` crate verifies the conversion via `convert_file`; the
// sdd tool merges the two crates' coverage of the page.

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

// Running `adoc my-document.adoc` with no `-o` converts the file silently and
// derives the output name `my-document.html` from the input.
#[test]
fn converts_and_derives_output_file() {
    // Step: run the command. Nothing is printed on success.
    verifies!(
        r#"
 $ adoc my-document.adoc
"#
    );

    non_normative!(
        r#"

As long as the document can be read, `adoc` prints no messages to your terminal.
--

. Type `ls` to list the files in the directory.
+
--
"#
    );

    // Step: listing the directory shows the derived `.html` file, whose name
    // `adoc` takes from the input by swapping the extension.
    verifies!(
        r#"
 $ ls
 my-document.adoc  my-document.html

You should see a new file named [.path]_my-document.html_.
`adoc` derives the name of the output file from the name of the input document,
replacing the _.adoc_ extension with _.html_.
"#
    );

    // Drive `adoc <name>.adoc`: no `-o`, so the output name is derived by
    // replacing `.adoc` with `.html`, the HTML lands there, and nothing prints.
    let source = "= Hello\n\nWorld.";
    let path = std::env::temp_dir().join(format!("adoc-first-file-{}.adoc", std::process::id()));
    let derived = path.with_extension("html");
    std::fs::write(&path, source).expect("write temp input");

    let cli = Cli::parse_from(["adoc", path.to_str().expect("temp path is UTF-8")]);
    let mut stdout = Vec::new();
    run(&cli, &mut stdout).expect("adoc converts the file");

    assert!(stdout.is_empty(), "adoc wrote to stdout on success");
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
The converted document is a complete, standalone HTML5 document you can publish.

To write the HTML5 to standard output instead of a file -- for example, to pipe
it into another program -- pass `-o -`:

"#
);

// The `-o -` form writes the HTML to standard output instead of a derived file.
#[test]
fn writes_to_stdout_with_dash() {
    verifies!(
        r#"
 $ adoc my-document.adoc -o -
"#
    );

    // Drive `adoc <file> -o -`: the dash forces the HTML to standard output, and
    // no derived file is written.
    let source = "= Hello\n\nWorld.";
    let path = std::env::temp_dir().join(format!(
        "adoc-first-file-stdout-{}.adoc",
        std::process::id()
    ));
    let derived = path.with_extension("html");
    std::fs::write(&path, source).expect("write temp input");

    let cli = Cli::parse_from([
        "adoc",
        path.to_str().expect("temp path is UTF-8"),
        "-o",
        "-",
    ]);
    let mut stdout = Vec::new();
    run(&cli, &mut stdout).expect("adoc converts the file");
    let wrote_derived = derived.exists();
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_file(&derived);

    assert!(!wrote_derived, "`-o -` should not write a derived file");
    let html = String::from_utf8(stdout).expect("stdout is UTF-8");
    assert!(html.starts_with("<!DOCTYPE html>"));
    assert!(html.contains("<title>Hello</title>"));
    assert!(html.trim_end().ends_with("</body>\n</html>"));
}

non_normative!(
    r#"

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
