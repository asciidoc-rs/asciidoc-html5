//! Coverage of Asciidoctor's *STEM Processing* overview page.
//!
//! This page surveys how STEM nodes are handed to a processing library and
//! tabulates the three integrations Asciidoctor offers (MathJax, Asciidoctor
//! Mathematical, and the AsciiMath gem) with their backends and load points. It
//! makes no crate-specific claim beyond that survey: the one integration this
//! crate models — client-side MathJax for the HTML backend — has its concrete
//! behavior verified on the `mathjax.adoc` page, and the other two are
//! non-goals (see the module doc). The whole overview is therefore tracked as
//! non-normative.

use crate::tests::sdd::*;

track_file!("ref/asciidoctor/docs/modules/stem/pages/index.adoc");

// Survey prose plus the integrations matrix. The concrete client-side MathJax
// behavior this crate implements is verified on the `mathjax.adoc` page; the
// image-based Mathematical and DocBook-only AsciiMath-gem integrations the
// table lists are non-goals, so the whole overview is reproduced as
// non-normative.
non_normative!(
    r#"
= STEM Processing

AsciiDoc defines syntax elements for adding xref:asciidoc:stem:stem.adoc[science, technology, engineering, and math (STEM) expressions] to an AsciiDoc document.
Much like AsciiDoc itself, this STEM notation allows the graphical notation that scientists, mathematicians, and engineers use to describe equations, formulas, etc. be expressed in a plain text format.
It's up to the converter to interpret and transform the source of these expressions into a displayable format.
That's where STEM processing comes in.

On this page, we look at how STEM expressions are handled when converting AsciiDoc and survey the integrations Asciidoctor offers to process them for various output formats.

== How it works

When a converter comes across a STEM node in an AsciiDoc document, it passes the source of the expression to a STEM processing library, such as MathJax.
That library either displays the rendered expression itself (e.g., MathJax), turns it into an image the converter can link to or embed in the output file (e.g., Asciidoctor Mathematical), or translates it into another format that something further down the pipeline can process (e.g., AsciiMath).

The STEM integrations are activated when the xref:asciidoc:stem:stem.adoc[stem document attribute] is set on the document and, if required, the relevant library is installed.

== STEM integrations

[%autowidth]
|===
|Library Name |Supported Versions |Backends |Loaded By

|MathJax
|2.7.9
|html
|client

|Asciidoctor Mathematical
|0.3.5
|any
|processor

|AsciiMath (Ruby)
|1.0.x or 2.0.x
|docbook
|processor
|===

You can explore these integrations in depth on the xref:mathjax.adoc[], xref:mathematical.adoc[], and xref:asciimath-gem.adoc[] pages.

If you're planning to convert to DocBook to leverage the DocBook toolchain, be sure to consider xref:docbook.adoc[] when deciding which STEM notation to use in your AsciiDoc document.
"#
);
