use clap::Parser as _;

use crate::{run, tests::sdd::*, Cli};

track_file!("docs/modules/generate-html/pages/index.adoc");

// This crate's "Generate HTML" page, tracked from the CLI. Its prose is
// descriptive documentation and so non-normative, but the `adoc` invocations it
// shows are verified here: the output file carries the `.html` extension (see
// `output_file_extension_is_html`), converting a file derives the
// `my-document.html` output name (see `converts_and_derives_output_file`), and
// `-o -` writes the HTML to standard output for previewing (see
// `writes_to_stdout_with_dash`). The `asciidoc-html5` crate verifies the
// API-level conversion, and the sdd tool merges the two crates' coverage. The
// "XHTML is not supported" section states a deliberate non-feature and is
// non-normative in both crates.

non_normative!(
    r#"
= Generate HTML from AsciiDoc
:navtitle: Generate HTML
:description: How asciidoc-html5 converts AsciiDoc to HTML5, the only output format it produces.

HTML5 is the only output format `asciidoc-html5` produces.
Whether you use the `adoc` command or the Rust API, converting an AsciiDoc
document gives you back a complete, standalone HTML5 document.
This page explains how to generate that HTML5 and how the renderer relates to
Asciidoctor's `html5` backend.

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
"#
);

// The horizontal list's output-extension term. With no `-o`, `adoc` derives the
// output file name from the input by swapping its extension for `.html`, so its
// output always carries the `.html` extension the page documents.
#[test]
fn output_file_extension_is_html() {
    verifies!(
        r#"
Output file extension:: _.html_
"#
    );

    let source = "= Hello\n\nWorld.";
    let path = std::env::temp_dir().join(format!(
        "adoc-generate-html-ext-{}.adoc",
        std::process::id()
    ));
    let derived = path.with_extension("html");
    std::fs::write(&path, source).expect("write temp input");

    let cli = Cli::parse_from(["adoc", path.to_str().expect("temp path is UTF-8")]);
    let mut stdout = Vec::new();
    run(&cli, &mut stdout).expect("adoc converts the file");
    let wrote_derived = derived.exists();
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_file(&derived);

    assert!(stdout.is_empty(), "adoc wrote to stdout on success");
    assert_eq!(
        derived.extension().and_then(|e| e.to_str()),
        Some("html"),
        "derived output name should carry the .html extension"
    );
    assert!(
        wrote_derived,
        "adoc did not create the derived .html output file"
    );
}

non_normative!(
    r#"
Compatibility target:: Asciidoctor's default `html5` backend

[NOTE]
====
Like Asciidoctor, `asciidoc-html5` embeds its
xref:default-stylesheet.adoc[default stylesheet] into the standalone HTML5 it
generates, so the converted document is styled without any external files. It
also adds the `<link>` that loads the web fonts the stylesheet prefers.
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
"#
);

// Running `adoc my-document.adoc` with no `-o` converts the file silently and
// derives the output name `my-document.html` from the input.
#[test]
fn converts_and_derives_output_file() {
    // Run the command. Nothing is printed on success.
    verifies!(
        r#"
 $ adoc my-document.adoc

"#
    );

    non_normative!(
        r#"
Since HTML5 is the only output `adoc` produces, you don't need to select a
converter.
--

. `adoc` prints nothing to the terminal on success. Type `ls` to list the
directory.
+
--
"#
    );

    // Listing the directory shows the derived _my-document.html_ file, whose name
    // `adoc` takes from the input by swapping the extension.
    verifies!(
        r#"
 $ ls
 my-document.adoc  my-document.html

You should see a new file named [.path]_my-document.html_. `adoc` derives the
output file name from the input, replacing the _.adoc_ extension with _.html_.
"#
    );

    // Drive `adoc <name>.adoc`: no `-o`, so the output name is derived by
    // replacing `.adoc` with `.html`, the HTML lands there, and nothing prints.
    let source = "= Hello\n\nWorld.";
    let path = std::env::temp_dir().join(format!("adoc-generate-html-{}.adoc", std::process::id()));
    let derived = path.with_extension("html");
    std::fs::write(&path, source).expect("write temp input");

    let cli = Cli::parse_from(["adoc", path.to_str().expect("temp path is UTF-8")]);
    let mut stdout = Vec::new();
    run(&cli, &mut stdout).expect("adoc converts the file");

    // Snapshot the results and clean up before asserting, so a failing
    // assertion doesn't leak the temp files.
    let wrote_derived = derived.exists();
    let html = std::fs::read_to_string(&derived).unwrap_or_default();
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_file(&derived);

    assert!(stdout.is_empty(), "adoc wrote to stdout on success");
    assert!(wrote_derived, "adoc did not create the derived output file");
    assert!(html.starts_with("<!DOCTYPE html>"));
    assert!(html.contains("<title>Hello</title>"));
    assert!(html.trim_end().ends_with("</body>\n</html>"));
}

non_normative!(
    r#"
--

. Open [.path]_my-document.html_ in your web browser. The converted document is
a complete, standalone HTML5 document you can publish.

To preview the HTML5 in the terminal instead of writing a file, pass `-o -` to
write to standard output and pipe it into a terminal browser:

"#
);

// The preview command pipes `adoc my-document.adoc -o -` into a terminal
// browser. The `-o -` half writes the rendered HTML to standard output, which
// this test drives directly.
#[test]
fn writes_to_stdout_with_dash() {
    verifies!(
        r#"
 $ adoc my-document.adoc -o - | w3m -T text/html

"#
    );

    let source = "= Hello\n\nWorld.";
    let path = std::env::temp_dir().join(format!(
        "adoc-generate-html-stdout-{}.adoc",
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
== Generate the HTML5 from the API

The Rust API produces the same HTML5. Convert a file on disk with `convert_file`:

[,rust]
----
let html = asciidoc_html5::convert_file("my-document.adoc")?;
----

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
