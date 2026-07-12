use crate::{convert, convert_with, tests::sdd::*, Options, SafeMode};

track_file!("docs/modules/generate-html/pages/custom-stylesheet.adoc");

// This crate's "Apply a Custom Stylesheet" page, tracked from the library. It
// verifies the API (Rust) invocations the page shows: embedding a custom
// stylesheet from supplied content, linking it at its normalized web path, and
// joining a `stylesdir`. The `adoc` invocations (the "Specify" walkthrough and
// the copy/link split) are verified by the CLI crate, which reproduces the same
// page; the sdd tool merges the two by line.

non_normative!(
    r#"
= Apply a Custom Stylesheet
:navtitle: Custom Stylesheet
:description: How asciidoc-html5 applies a custom stylesheet in place of the default, embedding or linking it per the safe mode.

In place of Asciidoctor's default stylesheet, you can tell `asciidoc-html5` to
apply a custom stylesheet of your own by setting the `stylesheet` document
attribute. It is embedded, linked, copied, or disabled by the same rules as the
default stylesheet -- see xref:stylesheet-modes.adoc[Stylesheet Modes].

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

Unlike the default stylesheet, a custom stylesheet does not pull in the Google
web fonts.

"#
);

// The API embedding and linking a custom stylesheet: supplied content is
// embedded inline under a low safe mode, and the `secure` default links it at
// its normalized web path.
#[test]
fn embed_or_link_a_custom_stylesheet() {
    verifies!(
        r##"
== Embed or link a custom stylesheet

Whether the `<head>` embeds or links a custom stylesheet follows the
xref:stylesheet-modes.adoc[safe mode and `linkcss`], exactly as for the default
stylesheet. Two details are specific to a custom stylesheet.

Embedding reads the stylesheet from disk, so it resolves against a base
directory and, under a jailed safe mode (`safe` or `server`), is confined to it
-- the same rules as an `include::` target. The `adoc` command anchors the read
at the input file's directory; through the API, name the document with
`Options::input_file` (or set `Options::base_dir`).

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

    let html = convert_with(
        "= My Document\n:stylesheet: my-theme.css\n\nHello.",
        &Options::new()
            .safe_mode(SafeMode::Unsafe)
            .stylesheet_content("body { color: #ff0000; }"),
    );
    assert!(html.contains("<style>\nbody { color: #ff0000; }\n</style>"));

    let html = convert("= My Document\n:stylesheet: my-theme.css\n\nHello.");
    assert!(html.contains(r#"<link rel="stylesheet" href="./my-theme.css">"#));
}

// The `stylesdir` attribute is joined ahead of the `stylesheet` value when
// building the linked reference; a URL (or absolute path) is linked as-is.
#[test]
fn configure_the_styles_directory() {
    verifies!(
        r##"
== Configure the styles directory

When the stylesheet lives in a subdirectory, name the directory with the
`stylesdir` attribute. It is joined ahead of the `stylesheet` value both when
resolving the file to embed and when building the linked reference:

[,rust]
----
use asciidoc_html5::{convert_with, Options};

let html = convert_with(
    "= My Document\n:stylesdir: css\n:stylesheet: my-theme.css\n\nHello.",
    &Options::new().set("linkcss"),
);
assert!(html.contains(r#"<link rel="stylesheet" href="./css/my-theme.css">"#));
----

A stylesheet given as a URL (or an absolute path) is already a complete
reference, so it is linked as-is:

[,rust]
----
let html = asciidoc_html5::convert("= Doc\n:stylesheet: https://example.org/theme.css\n\nHi.");
assert!(html.contains(r#"<link rel="stylesheet" href="https://example.org/theme.css">"#));
----

"##
    );

    let html = convert_with(
        "= My Document\n:stylesdir: css\n:stylesheet: my-theme.css\n\nHello.",
        &Options::new().set("linkcss"),
    );
    assert!(html.contains(r#"<link rel="stylesheet" href="./css/my-theme.css">"#));

    let html = convert("= Doc\n:stylesheet: https://example.org/theme.css\n\nHi.");
    assert!(html.contains(r#"<link rel="stylesheet" href="https://example.org/theme.css">"#));
}

non_normative!(
    r#"
[#copy]
== Copy a linked stylesheet

When a custom stylesheet is _linked_, the file it references has to exist next
to the HTML. With `copycss` set (its default in every safe mode but `secure`),
the `adoc` command copies it into the output directory at the same `stylesdir`
web path the `<link>` uses; see
xref:stylesheet-modes.adoc#copy[Copy the stylesheet to the output directory] for
the full behavior and the `AssetWriter` API the library exposes.

You can also copy the stylesheet _from_ a location other than the one the HTML
links it under, by setting `copycss` to that path. The file is read from the
`copycss` path but still written to (and linked at) the `stylesheet` web path:

 $ adoc -a linkcss -a copycss=vendor/theme.css -a stylesheet=theme.css my-document.adoc

== Known limitations

Embedding a _remote_ stylesheet (an `http`/`https` URL) is *not planned*:
neither the library nor the `adoc` CLI reads over the network, so a remote
stylesheet can only be linked, as shown above, never fetched and inlined.
"#
);
