//! Coverage of the AsciiDoc language description's *Equations and Formulas
//! (STEM)* page.
//!
//! STEM expressions are carried through the converter as passthroughs and
//! wrapped in the math delimiters a client-side processor (MathJax) reads: this
//! crate emits `\$ … \$` for AsciiMath and `\( … \)` / `\[ … \]` for LaTeX,
//! selects the notation from the `stem` attribute (or an explicit
//! `asciimath` / `latexmath` name), preserves AsciiMath block newlines by the
//! documented rules, and configures TeX equation numbering from `eqnums`. Each
//! is verified against `convert_with`.
//!
//! What the converter cannot observe is left non-normative: how MathJax renders
//! LaTeX block newlines (the converter passes LaTeX through verbatim), the
//! DocBook converter's AsciiMath-only support (a non-target backend), and
//! `\label` / `\eqref` equation cross-referencing (a MathJax runtime feature).
//! The page's include directives (`include::example$stem.adoc[...]`) are
//! reproduced verbatim for alignment; each verifying test drives the expanded
//! example content those tags name.

use crate::{convert_with, tests::sdd::*, Options};

track_file!("ref/asciidoc-lang/docs/modules/stem/pages/index.adoc");

/// Converts `source` to a standalone document (the default Secure safe mode
/// honors a document-set `:stem:`), so the MathJax `<head>` config and CDN
/// loader are present alongside the rendered body.
fn convert_stem(source: &str) -> String {
    convert_with(source, &Options::new().standalone(true))
}

// The page title, aliases, notation/URL attributes, and the introductory prose:
// STEM expressions are passthroughs handed to a converter-side processor. The
// concrete behavior is verified in the sections below.
non_normative!(
    r#"
= Equations and Formulas (STEM)
:page-aliases: stem.adoc
:stem: asciimath
:url-mathjax: https://www.mathjax.org
:url-asciimath: https://docs.mathjax.org/en/latest/input/asciimath.html
:url-latexmath: https://docs.mathjax.org/en/latest/input/tex/index.html
:url-mathjax-docs: https://meta.math.stackexchange.com/questions/5020/mathjax-basic-tutorial-and-quick-reference

If you need to include Science, Technology, Engineering and Math (STEM) expressions in your document, the AsciiDoc language supports embedding math-mode macros from {url-latexmath}[LaTeX] and/or {url-asciimath}[AsciiMath] notation as block or inline elements.
These elements act as passthroughs to preserve the expressions as entered.
The expressions are then passed on to the converter to be processed and rendered for display using a STEM provider (e.g., MathJax).

"#
);

// Setting `stem` (with no value, or `asciimath`) in the header activates STEM
// support and selects AsciiMath as the default notation: the page loads MathJax
// and inline expressions are wrapped in the AsciiMath delimiters (`\$ … \$`).
#[test]
fn activating_stem_defaults_to_asciimath() {
    verifies!(
        r#"
== Activating STEM support

To activate equation and formula support, set the `stem` attribute in the document's header (or by passing the attribute to the command line or API).

.Setting the stem attribute
[source]
----
include::example$stem.adoc[tag=base-co]
----
<.> The default notation value, `asciimath`, is assigned implicitly.

By default, AsciiDoc's stem integration assumes all equations are AsciiMath if not specified explicitly.
"#
    );

    // The `base-co` example sets `:stem:` with no value; an inline expression
    // then renders in the default AsciiMath notation, and MathJax is loaded.
    let html = convert_stem(
        "= My Diabolical Mathematical Opus\nJamie Moriarty\n:stem:\n\nstem:[sqrt(4) = 2]\n",
    );

    assert!(html.contains("\\$sqrt(4) = 2\\$"));
    assert!(html.contains(
        "<script src=\"https://cdnjs.cloudflare.com/ajax/libs/mathjax/2.7.9/MathJax.js?config=TeX-MML-AM_CHTML\"></script>"
    ));
}

// The HTML converter's support for both AsciiMath and LaTeX is exercised
// throughout this module; the DocBook converter's AsciiMath-only behavior
// targets a backend this crate does not implement, so this sentence is
// non-normative.
non_normative!(
    r#"
The HTML converter supports STEM content written in {url-asciimath}[AsciiMath^] and {url-latexmath}[TeX and LaTeX^] math notation.
The DocBook converter only processes AsciiMath notation, leaving LaTeX to be processed by a separate tool in the DocBook toolchain.

"#
);

// Assigning `latexmath` to the `stem` attribute makes LaTeX the default
// notation: an inline expression is then wrapped in the LaTeX inline delimiters
// (`\( … \)`) instead of the AsciiMath ones.
#[test]
fn stem_latexmath_makes_latex_the_default() {
    verifies!(
        r#"
If you want to use the LaTeX notation by default, assign `latexmath` to the stem attribute.

.Assigning an alternative notation to the stem attribute
[source]
----
include::example$stem.adoc[tag=base-alt]
----

"#
    );

    // The `base-alt` example sets `:stem: latexmath`; an inline expression then
    // renders in the LaTeX notation.
    let html = convert_stem(
        "= My Diabolical Mathematical Opus\nJamie Moriarty\n:stem: latexmath\n\nstem:[C = x]\n",
    );

    assert!(html.contains("\\(C = x\\)"));
    assert!(!html.contains("\\$C = x\\$"));
}

// The tip that both notations may coexist (verified under "Mixing STEM
// notations"), and the framing that stem content is inline or block:
// descriptive prose.
non_normative!(
    r#"
TIP: You can use both notations in the same document.
The value of the `stem` attribute merely sets the default notation.
To set the notation explicitly for a given block or inline span, just use `asciimath` or `latexmath` in place of `stem` as explained in <<mixing-notations>>.

Stem content can be displayed inline with other content or as discrete blocks.
No substitutions are applied to the content within a stem macro or block.

"#
);

// The inline `stem` macro is an implicit passthrough: the text between its
// square brackets is emitted verbatim inside the notation's inline delimiters.
#[test]
fn inline_stem_macro_is_a_passthrough() {
    verifies!(
        r#"
[#inline]
== Inline STEM content

The best way to mark up an inline formula is to use the `stem` macro.
The text between the square brackets of the inline stem macro acts as an implicit passthrough.

.Inline stem macro syntax
[source#ex-inline]
----
include::example$stem.adoc[tag=in-co]
----
<.> The inline stem macro contains only one colon (`:`).
<.> Place the expression within the square brackets (`[ ]`) of the macro.

The result of <<ex-inline>> is displayed below.

====
include::example$stem.adoc[tag=in]
====

"#
    );

    // The `in` example: an inline stem expression and one embedded in prose.
    let html = convert_stem(
        "= Doc\n:stem:\n\nstem:[sqrt(4) = 2]\n\nWater (stem:[H_2O]) is a critical component.\n",
    );

    assert!(html.contains("\\$sqrt(4) = 2\\$"));
    assert!(html.contains("Water (\\$H_2O\\$) is a critical component."));
}

// The only character that must be escaped inside an inline stem macro is the
// closing square bracket (`\]`); every other AsciiDoc construct is passed
// through untouched because the macro is an implicit passthrough.
#[test]
fn inline_stem_escapes_only_the_right_bracket() {
    verifies!(
        r#"
If the inline stem equation contains a right square bracket, you must escape this character using a backslash.

.Inline stem macro with a right square bracket
[source#ex-square-bracket]
----
include::example$stem.adoc[tag=in-sb]
----

The result of <<ex-square-bracket>> is displayed below.

====
include::example$stem.adoc[tag=in-sb]
====

Since the inline stem macro is an implicit passthrough, the closing square bracket it the only character you have to escape.
You don't have to worry about escaping other AsciiDoc syntax, such as attribute references.
All such syntax is passed through to the STEM processor as is.

"#
    );

    // The `in-sb` example: each escaped `\]` yields a literal `]` in the
    // passed-through expression.
    let html = convert_stem(
        "= Doc\n:stem:\n\nA matrix can be written as stem:[[[a,b\\],[c,d\\]\\]((n),(k))].\n",
    );

    assert!(html.contains("\\$[[a,b],[c,d]]((n),(k))\\$"));
}

// A block formula is a delimited passthrough block carrying the `stem` style;
// the converter wraps its content in the notation's block delimiters inside a
// `stemblock`.
#[test]
fn block_stem_content_wraps_in_stemblock() {
    verifies!(
        r#"
[#block]
== Block STEM content

Block formulas are marked up by assigning the `stem` style to a delimited passthrough block.

.Delimited stem block syntax
[source#ex-block]
----
include::example$stem.adoc[tag=bl-co]
----
<.> Assign the stem style to the passthrough block.
<.> A passthrough block is delimited by a line of four consecutive plus signs (`pass:[++++]`).

The result <<ex-block>> is rendered beautifully in the browser thanks to MathJax!

====
include::example$stem.adoc[tag=bl]
====

"#
    );

    // The `bl` example: a `[stem]` passthrough block.
    let html = convert_stem("= Doc\n:stem:\n\n[stem]\n++++\nsqrt(4) = 2\n++++\n");

    assert!(html.contains("<div class=\"stemblock\">"));
    assert!(html.contains("\\$sqrt(4) = 2\\$"));
}

// The tip that the converter supplies the MathJax delimiters automatically is
// borne out by every block example above (no author-written delimiters), so it
// needs no separate assertion.
non_normative!(
    r#"
TIP: You don't need to add special delimiters around the expression as the {url-mathjax-docs}[MathJax documentation^] suggests.
The AsciiDoc processor handles that for you automatically!

"#
);

// The intro and the collapsible listings illustrate the *input* forms for
// newlines in an AsciiMath block; the converter behavior they demonstrate is
// verified against the summary rule below.
non_normative!(
    r#"
=== Newlines in AsciiMath blocks

Newlines in an AsciiMath block are only preserved in certain circumstances.
The following examples illustrate how newlines are handled.

[%collapsible]
.Single newline not preserved
====
[listing]
----
x
y
----
====

[%collapsible]
.Single newline preserved if escaped
====
[listing]
----
x\
y
----
====

[%collapsible]
.Sequential newlines preserved if escaped
====
[listing]
----
x\
\
y
----
====

[%collapsible]
.Paragraph break preserved
====
[listing]
----
x

y
----
====

[%collapsible]
.Sequential newlines between paragraph break preserved
====
[listing]
----
x


y
----
====

"#
);

// The AsciiMath newline rule the converter implements: a paragraph break splits
// the expression into two delimited spans, and each subsequent blank line
// becomes a `<br>`.
#[test]
fn asciimath_paragraph_break_splits_the_expression() {
    verifies!(
        r#"
The first preserved newline splits the expression into two.
Subsequent newlines get translated into a `<br>` element.

"#
    );

    // A single paragraph break splits `x` and `y` into two AsciiMath spans.
    let split = convert_stem("= Doc\n:stem:\n\n[asciimath]\n++++\nx\n\ny\n++++\n");
    assert!(split.contains("\\$x\\$\n<br>\n\\$y\\$"));

    // Each additional blank line adds another `<br>` between the spans.
    let multi = convert_stem("= Doc\n:stem:\n\n[asciimath]\n++++\nx\n\n\ny\n++++\n");
    assert!(multi.contains("\\$x\\$\n<br>\n<br>\n\\$y\\$"));
}

// LaTeX block newline handling is a MathJax rendering concern: the converter
// passes LaTeX block content through verbatim (no delimiter-splitting like
// AsciiMath), so how `\\`, `~`, and `\\[1em]` affect line breaks is decided at
// render time, not in the converter's output. The whole section is
// non-normative here.
non_normative!(
    r#"
=== Newlines in LaTeX blocks

Newlines in a LaTeX block are only preserved in certain circumstances.
The following examples illustrate how newlines are handled.

[%collapsible]
.Single newline not preserved
====
[listing]
----
x
y
----
====

[%collapsible]
.Single newline preserved if escaped
====
[listing]
----
x\\
y
----
====

[%collapsible]
.Sequential newlines preserved if escaped and prefixed by null character
====
[listing]
----
x\\
~\\
y
----
====

[%collapsible]
.Paragraph break not preserved
====
[listing]
----
x

y
----
====

[%collapsible]
.Paragraph break preserved if separated by newline spacer
====
[listing]
----
x
\\[1em]
y
----
====

The first preserved newline splits the expression into two.
Subsequent newlines get translated into a `<br>` element.

"#
);

// A notation may be named explicitly (`asciimath` / `latexmath`) in place of
// `stem`, so both notations can appear in one document: an inline `latexmath`
// macro emits LaTeX inline delimiters, and per-block notation names select each
// block's delimiters independently of the document default.
#[test]
fn notations_can_be_mixed_by_name() {
    verifies!(
        r#"
[#mixing-notations]
== Mixing STEM notations

You can use multiple notations for STEM content within the same document by using the notation's name instead of the keyword `stem`.

For example, if you want to write an inline equation using the LaTeX notation, name the macro `latexmath`.

.Inline latexmath macro syntax
[source#ex-latexmath]
----
include::example$stem.adoc[tag=multi-l]
----

The result of <<ex-latexmath>> is displayed below.

====
include::example$stem.adoc[tag=multi-l]
====

The name that maps to the notation you want to use can also be applied to block STEM content.

.Using both asciimath and latexmath notations in a single document
[source]
----
include::example$stem.adoc[tag=multi-a]
----

Here's how the body of this example will be shown:

=====
include::example$stem.adoc[tag=multi-a-render]
=====

"#
    );

    // The `multi-l` example: an inline `latexmath` macro renders in LaTeX
    // notation regardless of the document default.
    let inline =
        convert_stem("= Doc\n:stem:\n\nlatexmath:[C = \\alpha + \\beta Y^{\\gamma} + \\epsilon]\n");
    assert!(inline.contains("\\(C = \\alpha + \\beta Y^{\\gamma} + \\epsilon\\)"));

    // The `multi-a` example: with `:stem: latexmath` as the default, a `[stem]`
    // block renders as LaTeX while an explicit `[asciimath]` block renders as
    // AsciiMath.
    let blocks = convert_stem(
        "= Doc\n:stem: latexmath\n\n[stem]\n++++\n\\lim_{n \\to \\infty}\\frac{n}{\\sqrt[n]{n!}} = {\\large e}\n++++\n\n[asciimath]\n++++\nsqrt(4) = 2\n++++\n",
    );
    assert!(blocks.contains("\\[\\lim_{n \\to \\infty}\\frac{n}{\\sqrt[n]{n!}} = {\\large e}\\]"));
    assert!(blocks.contains("\\$sqrt(4) = 2\\$"));
}

// Setting `eqnums` configures TeX equation numbering: an empty value (or `AMS`)
// makes the converter emit `autoNumber: "AMS"`, and `all` emits
// `autoNumber: "all"`. Which expressions actually receive a number is decided
// by MathJax at render time.
#[test]
fn eqnums_configures_tex_equation_numbering() {
    verifies!(
        r#"
== Equation numbering

When writing expresions in LaTeX, you can configure the STEM processor to add numbers to block equations.
To do so, set the `eqnums` attribute on the document to empty (or `AMS`).
Let's look at an example.

[source]
----
= Document Title
:stem: latexmath
:eqnums:

[stem]
++++
\begin{equation}
x = y^2
\end{equation}
++++
----

The automatic numbering is only applied when the expression is contained within an `\{equation}` container (in fact, to all such containers in a stem block).

If you want all expressions to be numbered, even those not designated as an equation, set the value of the `eqnums` attribute to `all`.
Let's look at an example.

[source]
----
= Document Title
:stem: latexmath
:eqnums: all

[stem]
++++
x = y^2
++++
----

In this case, the `\{equation}` container is not required.

"#
    );

    // A bare `:eqnums:` maps to the AMS numbering scheme.
    let ams = convert_stem(
        "= Document Title\n:stem: latexmath\n:eqnums:\n\n[stem]\n++++\n\\begin{equation}\nx = y^2\n\\end{equation}\n++++\n",
    );
    assert!(ams.contains("TeX: { equationNumbers: { autoNumber: \"AMS\" } }"));

    // `:eqnums: all` numbers every expression.
    let all = convert_stem(
        "= Document Title\n:stem: latexmath\n:eqnums: all\n\n[stem]\n++++\nx = y^2\n++++\n",
    );
    assert!(all.contains("TeX: { equationNumbers: { autoNumber: \"all\" } }"));
}

// Referencing an equation relies on MathJax's `\label` / `\eqref` machinery:
// the converter passes the LaTeX (and the `\eqref` inline expression) through
// verbatim, and the cross-reference is resolved when MathJax renders the page —
// nothing the converter's output can be checked against. This section is
// non-normative.
non_normative!(
    r#"
== Reference equations

When writing expresions in LaTeX, you can reference an equation in a stem block using an inline stem macro.
First, you need to add a label (aka ID) to the (ideally numbered) equation in a stem block using `\label` .

[source]
----
= Document Title
:stem: latexmath
:eqnums:

[stem]
++++
\begin{equation}
\label{hypotenuse}
c = \sqrt{a^2 + b^2}
\end{equation}
++++
----

Next, we reference that equation using `\eqref`.

[source]
----
We can calculate the hypotenuse using stem:[\eqref{hypotenuse}].
----

You can also use `\eqref` within any LaTeX stem block.
"#
);
