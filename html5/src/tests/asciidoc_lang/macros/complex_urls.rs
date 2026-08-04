//! Coverage of the AsciiDoc language description's *Troubleshooting Complex
//! URLs* page.
//!
//! A URL containing characters the inline parser treats as formatting markup
//! (repeating underscores, carets) can be preserved with one of three
//! passthrough workarounds: assigning it to an attribute, wrapping it in
//! `pass:macros[…]`, or using the double-plus `link:++…++[]` form. Each renders
//! the same bare link, verified through `convert`.
//!
//! The page's body is pulled in with `include::partial$ts-url-format.adoc`; the
//! tests reproduce the relevant examples inline.

use crate::{convert, tests::sdd::*};

track_file!("ref/asciidoc-lang/docs/modules/macros/pages/complex-urls.adoc");

// The expected bare-link output shared by all three workarounds.
const BARE_LINK: &str = r#"<a href="https://asciidoctor.org/now_this__link_works.html" class="bare">https://asciidoctor.org/now_this__link_works.html</a>"#;

non_normative!(
    r#"
= Troubleshooting Complex URLs

A URL may not display correctly when it contains characters such as underscores (`+_+`) or carets (`+^+`).
"#
);

// Solution A: assigning the URL to a document attribute preserves it, because
// inline formatting markup is substituted before attribute references are
// expanded — so the URL stays hidden while the rest of the text is formatted.
#[test]
fn assign_to_attribute() {
    verifies!(
        r#"
include::partial$ts-url-format.adoc[tag=sb]
"#
    );

    let doc = convert(
        "= Document Title\n\
         :link-with-underscores: https://asciidoctor.org/now_this__link_works.html\n\n\
         This URL has repeating underscores {link-with-underscores}.",
    );
    assert!(doc.contains(BARE_LINK));
}

// Solution B: wrapping the URL in `pass:macros[…]` applies only the macros
// substitution (which resolves links) to the URL, preserving its literal text.
#[test]
fn pass_macro() {
    let doc = convert(
        "This URL has repeating underscores pass:macros[https://asciidoctor.org/now_this__link_works.html].",
    );
    assert!(doc.contains(BARE_LINK));
}

// Solution B, variant: the double-plus passthrough (`++…++`) around the URL
// keeps it literal, but then the processor no longer recognizes it as a URL, so
// the explicit `link:` prefix is required.
#[test]
fn double_plus_inline_macro() {
    let doc = convert(
        "This URL has repeating underscores link:++https://asciidoctor.org/now_this__link_works.html++[].",
    );
    assert!(doc.contains(BARE_LINK));
}
