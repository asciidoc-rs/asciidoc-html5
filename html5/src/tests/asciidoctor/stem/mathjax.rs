//! Coverage of Asciidoctor's *MathJax and HTML* page — the client-side STEM
//! integration this crate models.
//!
//! When the `stem` attribute is set, a standalone HTML document loads MathJax
//! from the CDN and carries a `text/x-mathjax-config` stanza teaching it the
//! custom delimiters the converter wraps expressions in. Each is verified
//! against `convert_with`. The interactivity of the rendered expression is a
//! runtime MathJax behavior with no counterpart in the converter's output, so
//! that prose — and the cross-reference to the image-based Mathematical
//! alternative — is tracked as non-normative.

use crate::{convert_with, tests::sdd::*, Options};

track_file!("ref/asciidoctor/docs/modules/stem/pages/mathjax.adoc");

/// Converts `source` to a standalone document (the default Secure safe mode
/// honors a document-set `:stem:`). Standalone output surfaces the MathJax
/// `<head>`-config stanza and the CDN loader script, which embedded output
/// omits.
fn convert_stem(source: &str) -> String {
    convert_with(source, &Options::new().standalone(true))
}

// The source comment, the page title, its URL attribute definitions, the `stem`
// activation, and the descriptive lead: definitional prose whose concrete
// behavior is verified in the sections below.
non_normative!(
    r#"
// TODO: document how to change eqnums
= MathJax and HTML
:url-mathjax: https://www.mathjax.org
:url-asciimath: https://docs.mathjax.org/en/latest/input/asciimath.html
:url-latexmath: https://docs.mathjax.org/en/latest/input/tex/index.html
:stem:

{url-mathjax}[MathJax^] is a JavaScript library for displaying interactive LaTeX, AsciiMath, and MathML expressions in the browser.
The Asciidoctor HTML converter integrates with MathJax to provide STEM processing for {url-asciimath}[AsciiMath^] and {url-latexmath}[TeX and LaTeX^] notation used in an AsciiDoc document.
This integration offers simple, out-of-the-box support for STEM content when converting to HTML.

"#
);

// Setting `stem` on a standalone document configures the page to load MathJax
// from the cdnjs CDN — no installation required. The page's illustrative
// `[,html]` listing shows the loader with the older
// `?config=TeX-MML-AM_HTMLorMML` bundle; Asciidoctor 2.0.26 (the parity oracle)
// — and this crate — emit the `?config=TeX-MML-AM_CHTML` bundle, which likewise
// supports both TeX and AsciiMath. The host, path, and pinned version match the
// listing.
#[test]
fn a_stem_document_loads_mathjax_from_the_cdn() {
    verifies!(
        r#"
== How it works

If you convert your document to HTML using the HTML converter, and the `stem` document attribute is set on your document, the HTML will be configured to automatically load MathJax into the page from a CDN.
There's nothing you have to install.
Specifically, it loads the MathJax bundle that includes support for both TeX and AsciiMath, as shown here:

[,html]
----
<script src="https://cdnjs.cloudflare.com/ajax/libs/mathjax/2.7.9/MathJax.js?config=TeX-MML-AM_HTMLorMML"></script>
----

After the page loads, MathJax scans for expressions inside special delimiters and transforms them into a displayable format.
You may notice a brief flash of the expression in source form while this is happening.

"#
    );

    let html = convert_stem("= Doc\n:stem:\n\nstem:[x]\n");

    // The MathJax loader is emitted from the same host, path, and pinned version
    // the page's listing shows (the `?config=` bundle is the converter's
    // `TeX-MML-AM_CHTML`, not the page's illustrative `TeX-MML-AM_HTMLorMML`).
    assert!(html.contains(
        "<script src=\"https://cdnjs.cloudflare.com/ajax/libs/mathjax/2.7.9/MathJax.js?config=TeX-MML-AM_CHTML\"></script>"
    ));
}

// An AsciiMath `[stem]` block is emitted verbatim inside a `stemblock`, wrapped
// in the AsciiMath display delimiters (`\$ … \$`) MathJax renders in the
// browser. The expression's own text passes through untouched.
#[test]
fn an_asciimath_stem_block_is_emitted_for_mathjax() {
    verifies!(
        r#"
Here's an example of a STEM expression written in AsciiMath and rendered by MathJax.

[stem]
++++
e = sum_(n=0)^oo 1 / n!
++++

"#
    );

    let html = convert_stem("= Doc\n:stem:\n\n[stem]\n++++\ne = sum_(n=0)^oo 1 / n!\n++++\n");

    assert!(html.contains("<div class=\"stemblock\">"));
    assert!(html.contains("\\$e = sum_(n=0)^oo 1 / n!\\$"));
}

// The rendered expression's right-click interactivity (and viewing the original
// AsciiMath input) is a runtime MathJax behavior, not part of the converter's
// output, so it is non-normative here.
non_normative!(
    r#"
If you right click on the rendered expression, you'll notice its interactive.
One of the options even allows you to view the original AsciiMath input.

"#
);

// The converter emits a `text/x-mathjax-config` stanza that teaches MathJax the
// custom delimiters it wraps expressions in — chosen to avoid clashing with
// other AsciiDoc syntax — plus the other STEM settings.
#[test]
fn a_custom_delimiter_config_stanza_is_emitted() {
    verifies!(
        r#"
== Custom delimiters

The delimiters that the HTML converter places around the expressions to tell MathJax what to process are not the same as the ones MathJax looks for by default.
These custom delimiters were chosen to avoid conflicts with other syntax in the AsciiDoc language.

The converter configures MathJax to look for the custom delimiters it emits.
If you look at the HTML source, you'll see a text/x-mathjax-config stanza that defines them, as well as some other settings.

"#
    );

    let html = convert_stem("= Doc\n:stem:\n\nstem:[x]\n");

    // The config stanza is present and defines the LaTeX and AsciiMath
    // delimiters the converter emits around expressions.
    assert!(html.contains("<script type=\"text/x-mathjax-config\">"));
    assert!(html.contains(r#"inlineMath: [["\\(", "\\)"]]"#));
    assert!(html.contains(r#"delimiters: [["\\$", "\\$"]]"#));
}

// The scope note (HTML-only integration) and the cross-reference to the
// image-based Asciidoctor Mathematical alternative — a non-goal for this crate
// (see the module doc) — are descriptive.
non_normative!(
    r#"
== Alternatives

The MathJax integration is currently only designed for HTML output.
When converting to other backend formats, additional libraries are needed.
A good alternative is xref:mathematical.adoc[].
"#
);
