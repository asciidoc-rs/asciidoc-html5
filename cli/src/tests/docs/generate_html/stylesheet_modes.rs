use clap::Parser as _;

use crate::{run, tests::sdd::*, Cli};

track_file!("docs/modules/generate-html/pages/stylesheet-modes.adoc");

// This crate's "Stylesheet Modes" page, tracked from the CLI. It verifies the
// `adoc` invocations for each mode — embedding by default, linking with
// `linkcss`, copying the linked stylesheet next to the output, skipping the
// copy when `copycss` is unset, and disabling the stylesheet. The API (Rust)
// invocations the page also shows are verified by the `asciidoc-html5` crate,
// which reproduces the same page; the sdd tool merges the two by line.

/// Runs `adoc` with `args` and returns its standard output as a string.
fn adoc_stdout(args: &[&str]) -> String {
    let cli = Cli::parse_from(std::iter::once("adoc").chain(args.iter().copied()));
    let mut stdout = Vec::new();
    run(&cli, &mut stdout).expect("adoc converts the document");
    String::from_utf8(stdout).expect("stdout is UTF-8")
}

/// Creates a fresh temp directory named after `tag` holding a
/// `my-document.adoc` for the copy tests to convert into an output file.
fn scratch(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("adoc-clidocs-modes-{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create scratch dir");
    std::fs::write(dir.join("my-document.adoc"), "= My Document\n\nHello.").expect("write adoc");
    dir
}

non_normative!(
    r#"
= Stylesheet Modes
:navtitle: Stylesheet Modes
:description: How asciidoc-html5 embeds, links, copies, or disables the stylesheet, and the safe mode and attributes that control each mode.

The HTML converter can embed the CSS of the stylesheet directly into the HTML,
link to the stylesheet file, or disable it entirely. These modes apply whether
you use the xref:default-stylesheet.adoc[default stylesheet] or a
xref:custom-stylesheet.adoc[custom stylesheet]. This page covers the document
attributes and the xref:ROOT:safe-modes.adoc[safe mode] that control how the
stylesheet is applied.

IMPORTANT: A stylesheet is only applied when generating a standalone HTML
document, because the stylesheet goes inside the HTML `<head>` element.

[NOTE]
====
The prose on this page is non-normative documentation. The `adoc` and API
invocations it shows are normative: they are verified against the
implementation, so the documented behavior is guaranteed.
====

"#
);

// `adoc my-document.adoc` runs `unsafe`, so the stylesheet is embedded inline
// in a `<style>` element.
#[test]
fn embed_the_stylesheet() {
    verifies!(
        r#"
[#embed]
== Embed the stylesheet

When the xref:ROOT:safe-modes.adoc[safe mode] is `server` or lower, the default
behavior is to read the stylesheet, enclose its contents in a `<style>` element,
and embed it directly into the `<head>`. This makes the HTML self-contained: you
can move the file without losing its styling.

The `adoc` command runs `unsafe` by default, so converting a file from the
command line embeds the stylesheet:

 $ adoc my-document.adoc

Through the API, pass a safe mode below `secure` explicitly:

[,rust]
----
use asciidoc_html5::{convert_with, Options, SafeMode};

let html = convert_with(
    "= My Document\n\nHello.",
    &Options::new().safe_mode(SafeMode::Server),
);
assert!(html.contains("<style>"));
----

If the safe mode is `secure`, the converter <<link,links to the stylesheet>>
instead. The same two rules apply to the default and a custom stylesheet alike.

"#
    );

    let dir = scratch("embed");
    let path = dir.join("my-document.adoc");
    let path = path.to_str().expect("path is UTF-8");

    let html = adoc_stdout(&[path, "-o", "-"]);
    assert!(html.contains("<style>"));
    assert!(!html.contains("./asciidoctor.css"));

    // The alternative the sentence above notes: the same command under `secure`
    // links the stylesheet instead of embedding it.
    let secure = adoc_stdout(&[path, "-o", "-", "-S", "secure"]);
    assert!(secure.contains(r#"<link rel="stylesheet" href="./asciidoctor.css">"#));
    assert!(!secure.contains("<style>"));

    let _ = std::fs::remove_dir_all(&dir);
}

// `adoc -a linkcss` links to the default stylesheet at `./asciidoctor.css`
// instead of embedding it.
#[test]
fn link_to_the_stylesheet() {
    verifies!(
        r##"
[#link]
== Link to the stylesheet

Setting the `linkcss` attribute makes the converter link to the stylesheet with
a `<link rel="stylesheet">` element, using a relative `href`, instead of
embedding it. This is also the default under the `secure` safe mode. Linking is
useful when many documents should share one stylesheet.

The `linkcss` attribute must be set by the end of the header to be effective.
Set it in the document header, or from the API or CLI (shown here):

 $ adoc -a linkcss my-document.adoc

Since no stylesheet was specified, the converter links to the default one:

[,html]
----
<link rel="stylesheet" href="./asciidoctor.css">
----

Through the API, `secure` (the default) links, and `linkcss` forces linking
under any safe mode:

[,rust]
----
use asciidoc_html5::{convert_with, Options};

let html = convert_with("= My Document\n\nHello.", &Options::new().set("linkcss"));
assert!(html.contains(r#"<link rel="stylesheet" href="./asciidoctor.css">"#));
----

But where does that linked stylesheet file come from? Read on.

"##
    );

    let dir = scratch("link");
    let html = adoc_stdout(&[
        dir.join("my-document.adoc")
            .to_str()
            .expect("path is UTF-8"),
        "-o",
        "-",
        "-a",
        "linkcss",
    ]);
    assert!(html.contains(r#"<link rel="stylesheet" href="./asciidoctor.css">"#));
    let _ = std::fs::remove_dir_all(&dir);
}

// `adoc -a linkcss` with a real output file copies _asciidoctor.css_ into the
// output directory next to the HTML.
#[test]
fn copy_the_stylesheet_to_the_output_directory() {
    verifies!(
        r#"
[#copy]
== Copy the stylesheet to the output directory

A linked stylesheet has to exist at the referenced path for the browser to load
it. When the safe mode is `server` or lower, `linkcss` is set, and `copycss` is
set, the `adoc` command copies the stylesheet into the output directory next to
the HTML. For the default stylesheet it writes _asciidoctor.css_; for a custom
stylesheet it writes the file at the same `stylesdir` web path the `<link>`
uses. This works even when the xref:cli:index.adoc[output file] is in a
different directory from the source.

 $ adoc -a linkcss my-document.adoc

After running this command, _asciidoctor.css_ sits next to _my-document.html_:

 $ ls
 asciidoctor.css  my-document.adoc  my-document.html

****
As in Asciidoctor, embedding or linking the stylesheet is the converter's job,
but copying the file is a separate step. The `asciidoc-html5` library renders
HTML to a string and never writes files; it reports the stylesheet to copy
through an `AssetWriter` that the caller (such as `adoc`) drives. Convert with
`convert_file_with_writer` (or `convert_with_writer`) and hand it a
`DirAssetWriter` rooted at the output directory:

[,rust]
----
use asciidoc_html5::{convert_file_with_writer, DirAssetWriter, Options, SafeMode};

let mut writer = DirAssetWriter::new("output");
let html = convert_file_with_writer(
    "my-document.adoc",
    &Options::new().safe_mode(SafeMode::Safe).set("linkcss"),
    &mut writer,
)?;
----
****

If the safe mode is `secure`, the stylesheet is not copied, so the link would be
broken unless you copy the file yourself.

"#
    );

    let dir = scratch("copy");
    let _ = adoc_stdout(&[
        dir.join("my-document.adoc")
            .to_str()
            .expect("path is UTF-8"),
        "-o",
        dir.join("my-document.html")
            .to_str()
            .expect("path is UTF-8"),
        "-a",
        "linkcss",
    ]);
    assert!(dir.join("asciidoctor.css").is_file());
    let _ = std::fs::remove_dir_all(&dir);
}

// `adoc -a linkcss -a copycss!` links the stylesheet but does not copy it: no
// _asciidoctor.css_ is written to the output directory.
#[test]
fn to_copy_or_not_to_copy() {
    verifies!(
        r#"
=== To copy or not to copy

Whether the stylesheet is copied is controlled by the `copycss` attribute, which
is set by default unless the safe mode is `secure`. To prevent the copy
independent of the safe mode, unset `copycss`. It must be unset by the end of
the header to be effective -- in the document header (`:!copycss:`), or from the
API or CLI:

 $ adoc -a linkcss -a copycss! my-document.adoc

After this command, _asciidoctor.css_ is not written to the output directory.

The `copycss` attribute comes up again on the
xref:custom-stylesheet.adoc#copy[custom stylesheet page] as a way to set the
location the stylesheet is copied _from_, independent of where the HTML links
it.

"#
    );

    let dir = scratch("nocopy");
    let _ = adoc_stdout(&[
        dir.join("my-document.adoc")
            .to_str()
            .expect("path is UTF-8"),
        "-o",
        dir.join("my-document.html")
            .to_str()
            .expect("path is UTF-8"),
        "-a",
        "linkcss",
        "-a",
        "copycss!",
    ]);
    assert!(!dir.join("asciidoctor.css").exists());
    let _ = std::fs::remove_dir_all(&dir);
}

// `adoc -a stylesheet!` leaves out the stylesheet altogether: no `<style>` and
// no link.
#[test]
fn disable_the_stylesheet() {
    verifies!(
        r#"
[#disable]
== Disable the stylesheet

To leave out the stylesheet altogether, unset the `stylesheet` attribute. It is
set by default (to an empty value, which selects the default stylesheet), so
unsetting it tells the converter to apply no stylesheet at all:

 $ adoc -a stylesheet! my-document.adoc

NOTE: When the `stylesheet` attribute is unset, the `linkcss` and `copycss`
attributes are ignored.
"#
    );

    let dir = scratch("disable");
    let html = adoc_stdout(&[
        dir.join("my-document.adoc")
            .to_str()
            .expect("path is UTF-8"),
        "-o",
        "-",
        "-a",
        "stylesheet!",
    ]);
    assert!(!html.contains("<style>"));
    assert!(!html.contains("asciidoctor.css"));
    let _ = std::fs::remove_dir_all(&dir);
}
