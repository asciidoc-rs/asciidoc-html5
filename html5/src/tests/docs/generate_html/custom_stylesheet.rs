use crate::{convert, convert_with, tests::sdd::*, Options, SafeMode};

track_file!("docs/modules/generate-html/pages/custom-stylesheet.adoc");

// This crate's "Apply a Custom Stylesheet" page, tracked from the library. It
// documents applying a custom `stylesheet` in place of the default, embedded or
// linked per the safe mode. Every API claim it shows is verified here against
// `asciidoc_html5`: a custom stylesheet omits the web fonts, supplied content
// embeds inline, a linked custom stylesheet uses the normalized web path (with
// `stylesdir` mirrored and URIs preserved).
//
// The one `adoc` invocation (`$ adoc my-document.adoc`, which embeds the
// stylesheet read from disk) is non-normative here and verified from the CLI
// crate, whose `custom_stylesheet` tracker reproduces this same page and drives
// the binary. The sdd tool merges the two crates' coverage by line.

non_normative!(
    r#"
= Apply a Custom Stylesheet
:navtitle: Custom Stylesheet
:description: How asciidoc-html5 applies a custom stylesheet in place of the default, embedding or linking it per the safe mode.

In place of Asciidoctor's default stylesheet, you can tell `asciidoc-html5` to
apply a custom stylesheet of your own by setting the `stylesheet` document
attribute. Whether the stylesheet is _embedded_ or _linked_ follows the same
xref:ROOT:safe-modes.adoc[safe mode] rule as the
xref:default-stylesheet.adoc[default stylesheet].

[NOTE]
====
The prose on this page is non-normative documentation. The `adoc` and API
invocations it shows are normative: they are verified against the
implementation, so the documented behavior is guaranteed.
====

== Specify the custom stylesheet

Set the `stylesheet` attribute to the path of your stylesheet, relative to the
document. An empty value (the default) keeps the default stylesheet; any other
value selects a custom one.

Create a stylesheet next to your document -- say _my-theme.css_:

[,css]
----
body {
  color: #ff0000;
}
----

Then point the `stylesheet` attribute at it from the document header:

[,asciidoc]
----
= My Document
:stylesheet: my-theme.css

Hello.
----

Converting the file from the command line embeds the stylesheet's contents into
the `<head>`, so the output is self-contained:

 $ adoc my-document.adoc

"#
);

// Unlike the default stylesheet, a custom stylesheet pulls in no Google web
// fonts. (Verified with a linked custom stylesheet under the `secure` default;
// the absence of the font `<link>` holds however the stylesheet is applied.)
#[test]
fn a_custom_stylesheet_omits_the_web_fonts() {
    verifies!(
        r#"
Unlike the default stylesheet, a custom stylesheet does not pull in the Google
web fonts.

"#
    );

    let html = convert("= Doc\n:stylesheet: my-theme.css\n\nHi.");
    assert!(!html.contains("fonts.googleapis.com"));
}

non_normative!(
    r#"
== Embed or link

"#
);

// The embed-vs-link decision follows the safe mode (a mode below `secure`
// embeds; `secure`, or `linkcss`, links), and embedding reads the stylesheet
// from disk anchored at the base directory the named document establishes.
#[test]
fn embed_or_link_follows_the_safe_mode_and_reads_from_disk() {
    verifies!(
        r#"
Which form the `<head>` takes follows the xref:ROOT:safe-modes.adoc[safe mode],
exactly as for the default stylesheet:

* The `adoc` command (which runs `unsafe`) and any safe mode below `secure`
_embed_ the stylesheet's contents inline.
* The API default (`secure`), and any mode with `linkcss` set, _link_ to the
stylesheet.

Embedding reads the stylesheet from disk, so it resolves against a base
directory and, under a jailed safe mode (`safe` or `server`), is confined to it
-- the same rules as an `include::` target. The `adoc` command anchors the read
at the input file's directory; through the API, name the document with
`Options::input_file` (or set `Options::base_dir`).

"#
    );

    // A stylesheet on disk, next to the named document.
    let dir = std::env::temp_dir().join(format!("adoc-docs-embedlink-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    std::fs::write(dir.join("theme.css"), "body { color: #ff0000; }\n").expect("write css");
    let doc = dir.join("my-document.adoc");
    let source = "= My Document\n:stylesheet: theme.css\n\nHello.";

    // A safe mode below `secure` (here the jailed `server`) embeds the file's
    // contents, read from the base directory the named document anchors.
    let embedded = convert_with(
        source,
        &Options::new()
            .safe_mode(SafeMode::Server)
            .input_file(doc.clone()),
    );

    // The `secure` API default links instead, at the normalized web path.
    let linked = convert_with(source, &Options::new().input_file(doc.clone()));

    // `linkcss` links even under an embedding safe mode.
    let linkcss = convert_with(
        source,
        &Options::new()
            .safe_mode(SafeMode::Unsafe)
            .set("linkcss")
            .input_file(doc.clone()),
    );

    let _ = std::fs::remove_dir_all(&dir);

    // Below `secure`: the file's contents are embedded, not linked.
    assert!(embedded.contains("<style>\nbody { color: #ff0000; }\n</style>"));
    assert!(!embedded.contains("<link rel=\"stylesheet\""));

    // `secure` links at the normalized web path; `linkcss` does the same below it.
    assert!(linked.contains("<link rel=\"stylesheet\" href=\"./theme.css\">"));
    assert!(!linked.contains("<style>"));
    assert!(linkcss.contains("<link rel=\"stylesheet\" href=\"./theme.css\">"));
    assert!(!linkcss.contains("<style>"));
}

// Supplied content embeds the CSS inline with no file access, under an
// embedding safe mode.
#[test]
fn supplied_content_embeds_without_file_access() {
    verifies!(
        r#"
If you already hold the stylesheet's contents -- for example, from a resource
you loaded yourself -- pass them with `Options::stylesheet_content` to embed them
without any file access:

[,rust]
----
use asciidoc_html5::{convert_with, Options, SafeMode};

let html = convert_with(
    "= My Document\n:stylesheet: my-theme.css\n\nHello.",
    &Options::new()
        .safe_mode(SafeMode::Unsafe)
        .stylesheet_content("body { color: #ff0000; }"),
);
assert!(html.contains("<style>\nbody { color: #ff0000; }\n</style>"));
----

"#
    );

    let html = convert_with(
        "= My Document\n:stylesheet: my-theme.css\n\nHello.",
        &Options::new()
            .safe_mode(SafeMode::Unsafe)
            .stylesheet_content("body { color: #ff0000; }"),
    );
    assert!(html.contains("<style>\nbody { color: #ff0000; }\n</style>"));
}

// Linking works from a plain string: under the `secure` default a custom
// stylesheet is linked at its normalized web path.
#[test]
fn a_custom_stylesheet_is_linked_under_secure() {
    verifies!(
        r##"
Linking needs only the path, not the file, so it works from a plain string.
Under the `secure` default the `<head>` links to the stylesheet at its
normalized web path:

[,rust]
----
let html = asciidoc_html5::convert("= My Document\n:stylesheet: my-theme.css\n\nHello.");
assert!(html.contains(r#"<link rel="stylesheet" href="./my-theme.css">"#));
----

"##
    );

    let html = convert("= My Document\n:stylesheet: my-theme.css\n\nHello.");
    assert!(html.contains(r#"<link rel="stylesheet" href="./my-theme.css">"#));
}

non_normative!(
    r#"
== Configure the styles directory

When the stylesheet lives in a subdirectory, name the directory with the
`stylesdir` attribute. It is joined ahead of the `stylesheet` value both when
resolving the file to embed and when building the linked reference:

"#
);

// `stylesdir` is mirrored into the linked reference, ahead of the stylesheet
// file name.
#[test]
fn stylesdir_is_mirrored_into_the_linked_reference() {
    verifies!(
        r##"
[,rust]
----
use asciidoc_html5::{convert_with, Options};

let html = convert_with(
    "= My Document\n:stylesdir: css\n:stylesheet: my-theme.css\n\nHello.",
    &Options::new().set("linkcss"),
);
assert!(html.contains(r#"<link rel="stylesheet" href="./css/my-theme.css">"#));
----

"##
    );

    let html = convert_with(
        "= My Document\n:stylesdir: css\n:stylesheet: my-theme.css\n\nHello.",
        &Options::new().set("linkcss"),
    );
    assert!(html.contains(r#"<link rel="stylesheet" href="./css/my-theme.css">"#));
}

// A URL (or absolute path) stylesheet is a complete reference and is linked
// verbatim.
#[test]
fn a_url_stylesheet_is_linked_verbatim() {
    verifies!(
        r##"
A stylesheet given as a URL (or an absolute path) is already a complete
reference, so it is linked as-is:

[,rust]
----
let html = asciidoc_html5::convert("= Doc\n:stylesheet: https://example.org/theme.css\n\nHi.");
assert!(html.contains(r#"<link rel="stylesheet" href="https://example.org/theme.css">"#));
----

"##
    );

    let html = convert("= Doc\n:stylesheet: https://example.org/theme.css\n\nHi.");
    assert!(html.contains(r#"<link rel="stylesheet" href="https://example.org/theme.css">"#));
}

non_normative!(
    r#"
== Known limitations

`asciidoc-html5` produces HTML but never writes companion files. With a _linked_
custom stylesheet you are responsible for placing the stylesheet where the HTML
references it; there is no `copycss` step that copies it into an output
directory, which is tracked in
https://github.com/asciidoc-rs/asciidoc-html5/issues/39[issue #39].

Embedding a _remote_ stylesheet (an `http`/`https` URL) is *not planned*:
neither the library nor the `adoc` CLI reads over the network, so a remote
stylesheet can only be linked, as shown above, never fetched and inlined.
"#
);
