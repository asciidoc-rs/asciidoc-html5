use crate::{tests::sdd::*, Options, SafeMode};

// These tests assert the standalone document shell, so they render in
// standalone mode explicitly. The string entry points default to embedded,
// body-only output.
fn convert(source: &str) -> String {
    crate::convert_with(source, &Options::new().standalone(true))
}

fn convert_with(source: &str, options: &Options) -> String {
    crate::convert_with(source, &options.clone().standalone(true))
}

track_file!("docs/modules/generate-html/pages/default-stylesheet.adoc");

// This crate's "Default Stylesheet" page. It documents that a converted
// document carries Asciidoctor's default stylesheet (and the web-font `<link>`
// it relies on) in the standalone HTML5 `<head>`, and how the `webfonts`
// attribute changes it. The embed/link/copy/disable modes it points to are
// covered on the "Stylesheet Modes" page; every claim here is verified against
// `asciidoc_html5`.
//
// The page is tracked from the library crate only. It shows no `adoc`
// invocation whose behavior is not already covered by the CLI crate's other
// page tests, so tracking it from both crates would only duplicate the
// full-page reproduction with no added coverage.
//
// The prose is non-normative documentation; the remote-stylesheet limitation
// describes an absent feature, so it carries no rule to verify on this page.

/// Converts `source` under a safe mode below `Secure`, so the default
/// stylesheet is embedded inline (`<style>`). The default (`Secure`) mode links
/// it, but the stylesheet *content* is identical either way, so the CSS-content
/// claim below reads it back from the embed branch.
fn embed(source: &str) -> String {
    convert_with(source, &Options::new().safe_mode(SafeMode::Unsafe))
}

non_normative!(
    r#"
= The Default Stylesheet
:navtitle: Default Stylesheet
:description: How asciidoc-html5 applies Asciidoctor's default stylesheet and how the safe mode and web fonts control it.

When you convert a document, `asciidoc-html5` produces a standalone HTML5
document that carries Asciidoctor's default stylesheet, so the result looks
presentable without any extra files or setup. This is the same stylesheet
Asciidoctor's `html5` backend uses, included verbatim, so a document converted
with `adoc` looks the same as one converted with Asciidoctor.

Whether the stylesheet is _linked_ or _embedded_ depends on the
xref:ROOT:safe-modes.adoc[safe mode], and it can be linked, copied, or disabled.
See xref:stylesheet-modes.adoc[Stylesheet Modes] for those behaviors, which
apply to the default and a xref:custom-stylesheet.adoc[custom stylesheet] alike.

[NOTE]
====
The prose on this page is non-normative documentation. The `adoc` and API
invocations it shows are normative: they are verified against the
implementation, so the documented behavior is guaranteed.
====

"#
);

// The features a stylesheet is required for each have a companion CSS class in
// the default stylesheet: `.text-center` for the built-in role,
// `list-style-type` for the `loweralpha` marker, the `grid-all` table rules,
// and the `toc2` sidebar layout. (Read them from the embed branch, where the
// stylesheet text is inline.)
#[test]
fn the_embedded_stylesheet_backs_the_documented_features() {
    verifies!(
        r#"
== Why a stylesheet?

A stylesheet is not just decoration. Several AsciiDoc features only work when a
companion CSS class exists to back them, and the default stylesheet provides
those classes:

* *Built-in roles* such as `text-center` take effect through a matching CSS
class (`.text-center`).
* *List marker styles* such as `loweralpha` are applied by the stylesheet, not
by HTML on its own.
* *Table cell borders and shading* for the frame, grid, and stripes attributes
come from the stylesheet.
* The *TOC position* (a left or right sidebar) is a page-layout change the
stylesheet makes.

Applying the default stylesheet means these features work out of the box, and
it serves as a reference for the styles any custom stylesheet must provide.

"#
    );

    let html = embed("= Doc\n\nBody.");
    assert!(html.contains(".text-center{text-align:center!important}"));
    assert!(html.contains("loweralpha{list-style-type:lower-alpha}"));
    assert!(html.contains("grid-all"));
    assert!(html.contains("toc2"));
}

// The converter adds a Google Fonts `<link>` naming the three families the
// stylesheet prefers, regardless of whether the stylesheet is linked or
// embedded.
#[test]
fn the_converter_links_the_web_fonts() {
    verifies!(
        r#"
== Web fonts

The default stylesheet prefers a consistent set of open source fonts so that a
document looks the same across platforms rather than falling back to whatever
system fonts each browser happens to use. To load them, the converter adds a
`<link>` element that pulls the fonts from Google Fonts:

Noto Serif:: body text
Open Sans:: headings
Droid Sans Mono:: monospaced phrases and verbatim blocks

"#
    );

    let html = convert("= Doc\n\nBody.");
    assert!(html.contains(
        "<link rel=\"stylesheet\" href=\"https://fonts.googleapis.com/css?family=Open+Sans:300,300italic,400,400italic,600,600italic%7CNoto+Serif:400,400italic,700,700italic%7CDroid+Sans+Mono:400,700\">"
    ));
}

// Unsetting `webfonts` in the document header (or from outside) drops the
// Google Fonts `<link>`.
#[test]
fn unsetting_webfonts_drops_the_font_link() {
    verifies!(
        r#"
== Disable or change the web fonts

Unset the `webfonts` attribute in the document header to drop the Google Fonts
`<link>`. The browser then falls back to the generic font families the
stylesheet names (for example, `sans-serif`):

[,asciidoc]
----
= My Document
:webfonts!:

Hello.
----

You can set these attributes from the document header, as shown above, or supply
them from outside the document -- with `adoc -a webfonts!` on the command line,
or `Options::unset("webfonts")` through the xref:api:index.adoc[API]. An
attribute supplied from outside overrides a document-header assignment of the
same name by default.

"#
    );

    // From the document header.
    let html = convert("= My Document\n:webfonts!:\n\nHello.");
    assert!(!html.contains("<link rel=\"stylesheet\" href=\"https://fonts.googleapis.com"));

    // From outside the document, via the API the `adoc -a` option feeds into.
    let external = convert_with("= My Document\n\nHello.", &Options::new().unset("webfonts"));
    assert!(!external.contains("<link rel=\"stylesheet\" href=\"https://fonts.googleapis.com"));

    // An external attribute overrides a document-header assignment of the same
    // name: unsetting `webfonts` from outside wins over a header value.
    let overridden = convert_with(
        "= My Document\n:webfonts: from-header\n\nHello.",
        &Options::new().unset("webfonts"),
    );
    assert!(!overridden.contains("<link rel=\"stylesheet\" href=\"https://fonts.googleapis.com"));
}

// A `webfonts` value becomes the `family` query string parameter in the
// font-loader URL.
#[test]
fn a_webfonts_value_sets_the_font_family() {
    verifies!(
        r#"
Alternatively, set `webfonts` to a value to change which fonts are loaded. The
value becomes the `family` query string parameter in the font-loader URL. For
example, to use Ubuntu Mono for monospaced text:

[,asciidoc]
----
= My Document
:webfonts: Open+Sans:300,300italic,400,400italic,600,600italic%7CNoto+Serif:400,400italic,700,700italic%7CUbuntu+Mono:400

Hello.
----

"#
    );

    let webfonts =
        "Open+Sans:300,300italic,400,400italic,600,600italic%7CNoto+Serif:400,400italic,700,700italic%7CUbuntu+Mono:400";
    let html = convert(&format!("= My Document\n:webfonts: {webfonts}\n\nHello."));
    assert!(html.contains(&format!(
        "<link rel=\"stylesheet\" href=\"https://fonts.googleapis.com/css?family={webfonts}\">"
    )));
}

non_normative!(
    r#"
== Known limitations

`asciidoc-html5` applies both the default stylesheet and custom stylesheets,
embedding, linking, copying (`copycss`), or disabling them as described in
xref:stylesheet-modes.adoc[Stylesheet Modes]. One Asciidoctor stylesheet feature
is *not planned*: fetching a _remote_ stylesheet (an `http`/`https` URL) to
embed it. Neither the library nor the `adoc` CLI reads over the network, so a
remote stylesheet can only be linked.

To pin down exactly which markup the renderer supports today, see the
xref:ROOT:index.adoc[introduction].
"#
);
