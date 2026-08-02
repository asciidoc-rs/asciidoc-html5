//! Coverage of the AsciiDoc language description's *Italic* page.
//!
//! Italic maps to the AsciiDoc *emphasis* type. A single pair of underscores is
//! the constrained form; a double pair is the unconstrained form used for
//! bounded characters. This crate renders both through `convert` and nests the
//! combined monospace/bold/italic marks in the documented order, so every
//! example is verified. Only the title, byline comment, introduction, and the
//! two section headings are tracked as non-normative.

use crate::{
    convert,
    tests::{assert_html::assert_css, sdd::*},
};

track_file!("ref/asciidoc-lang/docs/modules/text/pages/italic.adoc");

non_normative!(
    r#"
= Italic
// content written and moved upstream from Antora by @graphitefriction

Text is often italicized in order to stress a word or phrase, quote a speaker, or introduce a term.
Italic text slants slightly to the right, and depending on the font, may have cursive swashes and flourishes.

The italic presentation of text maps to the formatted text type known as *emphasis* in the AsciiDoc language.

== Italic syntax

"#
);

// A single pair of underscores emphasizes a whole word or phrase (constrained);
// a double pair emphasizes bounded characters (unconstrained). Both render as
// `<em>`.
#[test]
fn italic_syntax() {
    verifies!(
        r#"
You can emphasize (aka italicize) a word or phrase by enclosing it in a single pair of underscores (e.g., `+_word_+`) (constrained).
You can emphasize bounded characters (i.e., characters within a word) by enclosing them in a pair of double underscores (e.g., `+char__act__ers+`) (unconstrained).

.Italic inline formatting
[#ex-italic]
----
An italic _word_, and an italic _phrase of text_.

Italic c__hara__cter__s__ within a word.
----

You don't need to use double underscores when an entire word or phrase marked as italic is directly followed by a common punctuation mark, such as `;`, `"`, and `!`.

The result of <<ex-italic>> is rendered below.

====
An italic _word_, and an italic _phrase of text_.

Italic c__hara__cter__s__ within a word.
====

"#
    );

    let constrained = convert("An italic _word_, and an italic _phrase of text_.\n");
    assert!(constrained.contains("An italic <em>word</em>, and an italic <em>phrase of text</em>."));
    assert_css(&constrained, "em", 2);

    let unconstrained = convert("Italic c__hara__cter__s__ within a word.\n");
    assert!(unconstrained.contains("Italic c<em>hara</em>cter<em>s</em> within a word."));
    assert_css(&unconstrained, "em", 2);

    // The single-underscore (constrained) form still applies when the italic
    // word is directly followed by each common punctuation mark the page names:
    // `;`, `"`, and `!`.
    let punctuation = convert("An _word_; an _word_\" and an _word_!\n");
    assert!(punctuation.contains("An <em>word</em>; an <em>word</em>\" and an <em>word</em>!"));
    assert_css(&punctuation, "em", 3);
}

non_normative!(
    r#"
== Mixing italic with other formatting

"#
);

// Combined with monospace and bold, italic is always the innermost mark:
// `<code><strong><em>…</em></strong></code>`.
#[test]
fn mixing_italic_with_other_formatting() {
    verifies!(
        r#"
You can add multiple emphasis styles to italic text as long as the syntax is placed in the correct order.

.Order of inline formatting syntax
[#ex-mix]
----
`*_monospace bold italic phrase_*` & ``**__char__**``acter``**__s__**``
----

xref:monospace.adoc[Monospace syntax] (`++`++`) must be the outermost formatting pair.
xref:bold.adoc[Bold syntax] (`+*+`) must be outside the italics formatting pair.
Italic syntax is always the innermost formatting pair.

The result of <<ex-mix>> is rendered below.

====
`*_monospace bold italic phrase_*` & ``**__char__**``acter``**__s__**``
====
"#
    );

    let out = convert("`*_monospace bold italic phrase_*` & ``**__char__**``acter``**__s__**``\n");
    assert!(out.contains("<code><strong><em>monospace bold italic phrase</em></strong></code>"));
    assert!(out.contains(
        "<code><strong><em>char</em></strong></code>acter<code><strong><em>s</em></strong></code>"
    ));
}
