//! Coverage of the AsciiDoc language description's *Bold* page.
//!
//! Bold maps to the AsciiDoc *strong* type. A single pair of asterisks is the
//! constrained form; a double pair is the unconstrained form used for bounded
//! characters. This crate renders both through `convert`, and it nests
//! monospace/bold/italic in the documented outer-to-inner order, so every
//! example on the page is verified. Only the title, the byline comment, the
//! introduction, and the two section headings are tracked as non-normative.

use crate::{
    convert,
    tests::{assert_html::assert_css, sdd::*},
};

track_file!("ref/asciidoc-lang/docs/modules/text/pages/bold.adoc");

non_normative!(
    r#"
= Bold
// content written and moved upstream from Antora by @graphitefriction

Text that is marked up as bold will stand out against the regular, surrounding text due to the application of a thicker and/or darker font.
Bold is useful when the text needs to catch the attention of a site visitor quickly scanning a page.

The bold presentation of text maps to the formatted text type known as *strong* in the AsciiDoc language.

== Bold syntax

"#
);

// A single pair of asterisks marks a whole word or phrase (constrained); a
// double pair marks bounded characters within a word (unconstrained). Both
// render as `<strong>`.
#[test]
fn bold_syntax() {
    verifies!(
        r#"
You can mark a word or phrase as bold by enclosing it in a single pair of asterisks (e.g., `+*word*+`) (constrained).
You can mark bounded characters (i.e., characters within a word) as bold by enclosing them in a pair of double asterisks (e.g., `+char**act**ers+`) (unconstrained).

.Bold inline formatting
[#ex-bold]
----
A bold *word*, and a bold *phrase of text*.

Bold c**hara**cter**s** within a word.
----

You don't need to use double asterisks when an entire word or phrase marked as bold is directly followed by a common punctuation mark, such as `;`, `"`, and `!`.

The results of <<ex-bold>> are displayed below.

====
A bold *word*, and a bold *phrase of text*.

Bold c**hara**cter**s** within a word.
====

"#
    );

    let constrained = convert("A bold *word*, and a bold *phrase of text*.\n");
    assert!(constrained
        .contains("A bold <strong>word</strong>, and a bold <strong>phrase of text</strong>."));
    assert_css(&constrained, "strong", 2);

    let unconstrained = convert("Bold c**hara**cter**s** within a word.\n");
    assert!(
        unconstrained.contains("Bold c<strong>hara</strong>cter<strong>s</strong> within a word.")
    );
    assert_css(&unconstrained, "strong", 2);
}

non_normative!(
    r#"
== Mixing bold with other formatting

"#
);

// When monospace, bold, and italic are combined, the marks nest from outermost
// (monospace) to innermost (italic):
// `<code><strong><em>…</em></strong></code>`.
#[test]
fn mixing_bold_with_other_formatting() {
    verifies!(
        r#"
You can add multiple emphasis styles to bold text as long as the syntax is placed in the correct order.

.Order of inline formatting syntax
[#ex-mix]
----
`*_monospace bold italic phrase_*` & ``**__char__**``acter``**__s__**``
----

xref:monospace.adoc[Monospace syntax] (`++`++`) must be the outermost formatting pair (i.e., outside the bold formatting pair).
xref:italic.adoc[Italic syntax] (`+_+`) is always the innermost formatting pair.

The results of <<ex-mix>> are displayed below.

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
