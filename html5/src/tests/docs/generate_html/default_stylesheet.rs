use crate::{convert, convert_with, tests::sdd::*, Options, SafeMode};

track_file!("docs/modules/generate-html/pages/default-stylesheet.adoc");

// This crate's "Default Stylesheet" page. It documents that a converted
// document carries Asciidoctor's default stylesheet (and the web-font `<link>`
// it relies on) in the standalone HTML5 `<head>`, that the safe mode decides
// whether it is linked or embedded, and how the `linkcss`, `webfonts`, and
// `stylesheet` attributes change that. Every claim is verified against
// `asciidoc_html5`.
//
// The page is tracked from the library crate only. The `adoc` binary is a thin
// wrapper over `convert_with`, so the stylesheet it produces is identical to
// the library's; the one `adoc` invocation shown here (`adoc my-document.adoc`)
// exercises the same skeleton already covered by the CLI crate's other page
// tests. Tracking this page from both crates would only duplicate the full-page
// reproduction with no added coverage.
//
// The prose is non-normative documentation; the docinfo and `copycss`
// limitations describe absent features, so they carry no rule to verify.

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
xref:ROOT:safe-modes.adoc[safe mode]. Under the API default (`secure`) the
converter links to _asciidoctor.css_; the `adoc` command, and any safe mode
below `secure`, embeds the stylesheet inline instead. See <<applying>>.

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

// Applying the stylesheet: the default safe mode (`secure`) links it; a lower
// safe mode embeds it; and `linkcss` overrides the safe-mode default either
// way.
#[test]
fn applying_links_by_default_and_a_lower_safe_mode_embeds() {
    verifies!(
        r##"
[#applying]
== Applying the stylesheet

There is nothing special to do: converting a document applies the default
stylesheet automatically. Under the default safe mode (`secure`), the `<head>`
links to _asciidoctor.css_:

[,rust]
----
let html = asciidoc_html5::convert("= My Document\n\nHello.");
assert!(html.contains(r#"<link rel="stylesheet" href="./asciidoctor.css">"#));
----

"##
    );

    let html = convert("= My Document\n\nHello.");
    assert!(html.contains(r#"<link rel="stylesheet" href="./asciidoctor.css">"#));
    assert!(!html.contains("<style>"));

    non_normative!(
        r#"
You are then responsible for placing _asciidoctor.css_ next to the output so the
browser can find it. Unlike Asciidoctor, `asciidoc-html5` does not write that
file for you (there is no `copycss` step, tracked in
https://github.com/asciidoc-rs/asciidoc-html5/issues/39[issue #39]); it only
produces the HTML.

"#
    );

    verifies!(
        r#"
To embed the stylesheet inline instead -- so the output is self-contained -- run
the document under a xref:ROOT:safe-modes.adoc[safe mode] below `secure`. The
`adoc` command does this by default (it runs `unsafe`), so converting a file from
the command line produces a self-contained result:

 $ adoc my-document.adoc

Through the API, pass the safe mode explicitly:

[,rust]
----
use asciidoc_html5::{convert_with, Options, SafeMode};

let html = convert_with(
    "= My Document\n\nHello.",
    &Options::new().safe_mode(SafeMode::Server),
);
assert!(html.contains("<style>"));
----

"#
    );

    // The API path shown on the page; the `adoc` default (unsafe) embeds too.
    let embedded = convert_with(
        "= My Document\n\nHello.",
        &Options::new().safe_mode(SafeMode::Server),
    );
    assert!(embedded.contains("<style>"));
    assert!(!embedded.contains("./asciidoctor.css"));

    verifies!(
        r#"
The `linkcss` attribute overrides the safe-mode default: set it to link even
under a low safe mode, or unset it from the API (`Options::unset("linkcss")`) to
embed under `secure`. Under `secure`, a document cannot unset `linkcss` itself.

"#
    );

    // Set `linkcss` to link even under a low (unsafe) safe mode.
    let forced_link = convert_with(
        "= Doc\n\nBody.",
        &Options::new().set("linkcss").safe_mode(SafeMode::Unsafe),
    );
    assert!(forced_link.contains("<link rel=\"stylesheet\" href=\"./asciidoctor.css\">"));

    // Unset `linkcss` from the API to embed under `secure`.
    let forced_embed = convert_with("= Doc\n\nBody.", &Options::new().unset("linkcss"));
    assert!(forced_embed.contains("<style>"));

    // A document cannot unset `linkcss` under `secure`.
    let locked = convert("= Doc\n:linkcss!:\n\nBody.");
    assert!(locked.contains("<link rel=\"stylesheet\" href=\"./asciidoctor.css\">"));
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

// The stylesheet limitations that are checkable: a custom `stylesheet` value
// has no effect (the file is not read, so its contents are absent), and
// explicitly unsetting the stylesheet drops the default one.
#[test]
fn custom_and_unset_stylesheet_behaviors() {
    verifies!(
        r#"
== Known limitations

`asciidoc-html5` applies only the default stylesheet. A few of Asciidoctor's
stylesheet features are not available:

* *Custom stylesheets.* Setting `stylesheet` to your own CSS file has no effect:
the library converts a string to a string and cannot read an external file.
Explicitly unsetting the stylesheet (`:stylesheet!:`) does drop the default one.
Supporting custom stylesheets is tracked in
https://github.com/asciidoc-rs/asciidoc-html5/issues/36[issue #36].
"#
    );

    // A custom stylesheet file is not read, so neither its name nor the default
    // stylesheet appears in the output.
    let custom = convert("= Doc\n:stylesheet: my-theme.css\n\nBody.");
    assert!(!custom.contains("my-theme.css"));
    assert!(!custom.contains("<style>"));
    assert!(!custom.contains("asciidoctor.css"));

    // Explicitly unsetting the stylesheet drops the default (linked or embedded).
    let unset = convert("= Doc\n:stylesheet!:\n\nBody.");
    assert!(!unset.contains("<style>"));
    assert!(!unset.contains("asciidoctor.css"));
}

non_normative!(
    r#"
* *docinfo.* Injecting auxiliary styles through a docinfo file is not supported,
tracked in https://github.com/asciidoc-rs/asciidoc-html5/issues/40[issue #40].
* *copycss.* The library never writes files, so with a linked stylesheet you
must supply _asciidoctor.css_ yourself; writing it out is tracked in
https://github.com/asciidoc-rs/asciidoc-html5/issues/39[issue #39].

To pin down exactly which markup the renderer supports today, see the
xref:ROOT:index.adoc[introduction].
"#
);
