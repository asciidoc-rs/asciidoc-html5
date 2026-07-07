use crate::{convert, tests::sdd::*};

track_file!("docs/modules/generate-html/pages/default-stylesheet.adoc");

// This crate's "Default Stylesheet" page. It documents that a converted
// document embeds Asciidoctor's default stylesheet (and the web-font `<link>`
// it relies on) into the standalone HTML5 `<head>`, and how the `linkcss`,
// `webfonts`, and `stylesheet` attributes change that. Every claim is verified
// against `asciidoc_html5::convert`.
//
// The page is tracked from the library crate only. The `adoc` binary is a thin
// wrapper over `convert`, so the stylesheet it produces is identical to the
// library's, and it exposes no attribute options of its own — the one `adoc`
// invocation shown here (`adoc my-document.adoc`) exercises the same skeleton
// already covered by the CLI crate's other page tests. Tracking this page from
// both crates would only duplicate the full-page reproduction with no added
// coverage.
//
// The prose is non-normative documentation; the docinfo and `copycss`
// limitations describe absent features, so they carry no rule to verify.

non_normative!(
    r#"
= The Default Stylesheet
:navtitle: Default Stylesheet
:description: How asciidoc-html5 embeds Asciidoctor's default stylesheet and how to control the web fonts.

When you convert a document, `asciidoc-html5` produces a standalone HTML5
document with Asciidoctor's default stylesheet embedded in its `<head>`, so the
result looks presentable without any extra files or setup. This is the same
stylesheet Asciidoctor's `html5` backend embeds, included verbatim, so a
document converted with `adoc` looks the same as one converted with Asciidoctor.

[NOTE]
====
The prose on this page is non-normative documentation. The `adoc` and API
invocations it shows are normative: they are verified against the
implementation, so the documented behavior is guaranteed.
====

"#
);

// The features a stylesheet is required for each have a companion CSS class in
// the embedded default stylesheet: `.text-center` for the built-in role,
// `list-style-type` for the `loweralpha` marker, the `grid-all` table rules,
// and the `toc2` sidebar layout.
#[test]
fn the_embedded_stylesheet_backs_the_documented_features() {
    verifies!(
        r#"
== Why embed a stylesheet?

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

Embedding the default stylesheet means these features work out of the box, and
it serves as a reference for the styles any custom stylesheet must provide.

"#
    );

    let html = convert("= Doc\n\nBody.");
    assert!(html.contains(".text-center{text-align:center!important}"));
    assert!(html.contains("loweralpha{list-style-type:lower-alpha}"));
    assert!(html.contains("grid-all"));
    assert!(html.contains("toc2"));
}

// The converter adds a Google Fonts `<link>` naming the three families the
// stylesheet prefers.
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

// Converting a document embeds the stylesheet automatically: the `<head>` holds
// the web-font `<link>` followed by a `<style>` element with the stylesheet.
#[test]
fn converting_embeds_the_stylesheet_in_the_head() {
    verifies!(
        r#"
== Applying the stylesheet

There is nothing special to do: converting a document embeds the stylesheet
automatically. The `<head>` of the output holds the web-font `<link>` followed
by a `<style>` element containing the stylesheet.

[,rust]
----
let html = asciidoc_html5::convert("= My Document\n\nHello.");
assert!(html.contains("<style>"));
----

Converting a file from the command line produces the same self-contained result:

 $ adoc my-document.adoc

"#
    );

    let html = convert("= My Document\n\nHello.");
    assert!(html.contains("<style>"));

    // Ordering: the web-font <link> precedes the <style>, both in the head.
    let head = &html[..html.find("</head>").expect("head")];
    let fonts = head.find("fonts.googleapis.com").expect("web-font link");
    let style = head.find("<style>").expect("style");
    assert!(fonts < style);
}

// Setting `linkcss` replaces the inline `<style>` with a `<link>` to
// `./asciidoctor.css`.
#[test]
fn linkcss_links_the_stylesheet() {
    verifies!(
        r##"
If you would rather link to the stylesheet than embed it, set the `linkcss`
attribute. The `<style>` element is then replaced by a `<link>` to
_asciidoctor.css_:

[,rust]
----
let html = asciidoc_html5::convert("= My Document\n:linkcss:\n\nHello.");
assert!(html.contains(r#"<link rel="stylesheet" href="./asciidoctor.css">"#));
----

"##
    );

    let html = convert("= My Document\n:linkcss:\n\nHello.");
    assert!(html.contains(r#"<link rel="stylesheet" href="./asciidoctor.css">"#));
    assert!(!html.contains("<style>"));
}

non_normative!(
    r#"
You are then responsible for placing _asciidoctor.css_ next to the output so the
browser can find it. Unlike Asciidoctor, `asciidoc-html5` does not write that
file for you (there is no `copycss` step, tracked in
https://github.com/asciidoc-rs/asciidoc-html5/issues/39[issue #39]); it only
produces the HTML.

[NOTE]
====
Asciidoctor's API links to the stylesheet by default (instead of embedding it)
because of its default _safe mode_. `asciidoc-html5` has no safe mode: it always
embeds the default stylesheet unless you ask for `linkcss`. Modeling safe mode is
tracked in
https://github.com/asciidoc-rs/asciidoc-html5/issues/37[issue #37].
====

"#
);

// Unsetting `webfonts` in the document header drops the Google Fonts `<link>`
// while keeping the embedded stylesheet.
#[test]
fn unsetting_webfonts_drops_the_font_link() {
    verifies!(
        r#"
== Disable or change the web fonts

Unset the `webfonts` attribute in the document header to drop the Google Fonts
`<link>`. The stylesheet stays embedded, and the browser falls back to the
generic font families it names (for example, `sans-serif`):

[,asciidoc]
----
= My Document
:webfonts!:

Hello.
----

The `adoc` command has no options for setting attributes, so control the web
fonts (and the other attributes on this page) from the document header. Passing
attributes from outside the document is tracked in
https://github.com/asciidoc-rs/asciidoc-html5/issues/38[issue #38].

"#
    );

    let html = convert("= My Document\n:webfonts!:\n\nHello.");
    assert!(!html.contains("<link rel=\"stylesheet\" href=\"https://fonts.googleapis.com"));
    assert!(html.contains("<style>"));
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

`asciidoc-html5` embeds only the default stylesheet. A few of Asciidoctor's
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

    // Explicitly unsetting the stylesheet drops the embedded default.
    let unset = convert("= Doc\n:stylesheet!:\n\nBody.");
    assert!(!unset.contains("<style>"));
}

non_normative!(
    r#"
* *docinfo.* Injecting auxiliary styles through a docinfo file is not supported,
tracked in https://github.com/asciidoc-rs/asciidoc-html5/issues/40[issue #40].
* *copycss.* The library never writes files, so with `linkcss` you must supply
_asciidoctor.css_ yourself; writing it out is tracked in
https://github.com/asciidoc-rs/asciidoc-html5/issues/39[issue #39].

To pin down exactly which markup the renderer supports today, see the
xref:ROOT:index.adoc[introduction].
"#
);
