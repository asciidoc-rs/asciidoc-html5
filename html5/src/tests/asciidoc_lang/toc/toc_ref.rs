//! Coverage of the AsciiDoc language description's *TOC Attributes Reference*
//! page.
//!
//! This page is a cross-backend summary table for the `toc`, `toclevels`,
//! `toc-title`, and `toc-class` attributes. The concrete html-backend behaviors
//! it tabulates are verified in detail on the other `toc` module pages – the
//! default depth of `toclevels` on the *Adjust the TOC Depth* page, the default
//! `toc-title` on the *Customize the TOC Title* page, and the accepted `toc`
//! placements on the *Position the TOC* page. Here the html-backend defaults
//! for `toc` (not set by default; `auto` when the value is unspecified) and
//! `toc-class` (defaults to `toc`) are verified through `convert`; the
//! html-embeddable, pdf, and docbook rows describe converters this crate does
//! not implement and are non-normative.

use crate::{
    convert_with,
    tests::{
        assert_html::{assert_css, assert_xpath},
        sdd::*,
    },
    Options,
};

track_file!("ref/asciidoc-lang/docs/modules/toc/pages/toc-ref.adoc");

// Renders a standalone document, so the TOC and its class are present in the
// output.
fn convert(source: &str) -> String {
    convert_with(source, &Options::new().standalone(true))
}

// A minimal document with two nested sections, used to exercise the attribute
// defaults the table documents.
const DOC: &str = "\
= Document Title

== Section One

=== Section Two
";

// The page title, the en-dash attribute entry, and the table's opening – the
// commented-out column spec, the `%autowidth` option, and the header row.
non_normative!(
    r#"
= TOC Attributes Reference
:endash: &#8211;

//[cols="1,1,2,2,1"]
[%autowidth]
|===
|Attribute |Values |Example Syntax |Notes |Backends

"#
);

// The html-backend `toc` row: not set by default, and `auto` when the value is
// left unspecified. Without `:toc:` there is no TOC; with an empty `:toc:` the
// TOC appears in its default (auto) location, the document header. The accepted
// values are verified on the *Position the TOC* page.
#[test]
fn toc_html_row_defaults() {
    verifies!(
        r#"
.4+|`toc`
|`auto`, `left`, `right`, `macro`, `preamble`
|`:toc: left`
|Not set by default.
Defaults to `auto` if value is unspecified.
|html

"#
    );

    // Not set by default: no `toc` attribute means no TOC.
    let plain = convert(DOC);
    assert_css(&plain, r#"div#toc"#, 0);

    // Defaults to `auto` when the value is unspecified: an empty `:toc:` places
    // the TOC in the document header.
    let auto = convert("= Document Title\n:toc:\n\n== Section One\n\n=== Section Two\n");
    assert_xpath(&auto, r#"//div[@id="header"]//div[@id="toc"]"#, 1);
}

// The html-embeddable, pdf, and docbook `toc` rows. The embeddable placements
// are verified on the *Position the TOC* page; the pdf and docbook converters
// are not implemented by this crate.
non_normative!(
    r#"
|`auto`, `macro`, `preamble`
|`:toc: macro`
|Not set by default.
Defaults to `auto` if value is unspecified.
|html (embeddable)

|`auto`
|`:toc:`
|Not set by default.
When the title page is enabled in PDF output, the table of contents is placed directly after the title page.
|pdf

|`auto`
|`:toc:`
|Not set by default.
The placement and styling of the table of contents is determined by the DocBook toolchain configuration.
|docbook

"#
);

// The `toclevels` row. Its default value of `2` is verified on the *Adjust the
// TOC Depth* page.
non_normative!(
    r#"
|`toclevels`
|0{endash}5
|`:toclevels: 4`
|Default value is `2`.
|html, pdf

"#
);

// The `toc-title` row. Its default value _Table of Contents_ is verified on the
// *Customize the TOC Title* page.
non_normative!(
    r#"
|`toc-title`
|user-defined
|`:toc-title: Contents`
|Default value is _Table of Contents_.
|html, pdf

"#
);

// The `toc-class` row: the TOC div's class defaults to `toc`, and a
// `toc-class` value overrides it. Both are verified against the standalone
// output.
#[test]
fn toc_class_row_default_and_override() {
    verifies!(
        r#"
|`toc-class`
|valid CSS class name
|`:toc-class: floating-toc`
|Default value is _toc_.
|html
|===
"#
    );

    // Default value is `toc`.
    let default = convert("= Document Title\n:toc:\n\n== Section One\n\n=== Section Two\n");
    assert_css(&default, r#"div#toc.toc"#, 1);

    // A `toc-class` value overrides the default class.
    let custom = convert(
        "= Document Title\n:toc:\n:toc-class: floating-toc\n\n== Section One\n\n=== Section Two\n",
    );
    assert_css(&custom, r#"div#toc.floating-toc"#, 1);
}
