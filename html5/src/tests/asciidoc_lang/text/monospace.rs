//! Coverage of the AsciiDoc language description's *Monospace* page.
//!
//! Monospace maps to the AsciiDoc *monospaced* type. A single pair of backticks
//! is the constrained form; a double pair is the unconstrained form for bounded
//! characters. Because monospace is subject to normal substitutions, its
//! content still runs through the other inline steps, so the examples exercise
//! attribute-free substitutions and nested bold/italic. Every example renders
//! through `convert`, so each is verified; the title, introduction, and the
//! section headings are tracked as non-normative.

use crate::{
    convert,
    tests::{assert_html::assert_css, sdd::*},
};

track_file!("ref/asciidoc-lang/docs/modules/text/pages/monospace.adoc");

non_normative!(
    r#"
= Monospace

In AsciiDoc, a span of text enclosed in a single pair of backticks (`{backtick}`) is displayed using a fixed-width (i.e., monospaced) font.
Monospace text formatting is typically used to represent text shown in computer terminals or code editors (often referred to as a codespan).

The monospace presentation of text maps to the formatted text type known as *monospaced* in the AsciiDoc language.

== Constrained

"#
);

// A single pair of backticks renders a constrained monospace span as `<code>`.
// Because monospace is not literal, the enclosed text is still interpreted
// (here the superscript and italic marks and the smart-quote replacements).
#[test]
fn constrained() {
    verifies!(
        r#"
Here's an example:

.Constrained monospace syntax
[#ex-constrain]
----
include::example$text.adoc[tag=mono]
----

The result of <<ex-constrain>> is rendered below.

====
include::example$text.adoc[tag=mono]
====

"#
    );

    let out = convert(
        "\"`Wait!`\" Indigo plucked a small vial from her desk's top drawer\nand held it toward us.\nThe vial's label read: `E=mc^2^`; the `E` represents _energy_,\nbut also pure _genius!_\n",
    );
    assert!(out.contains("<code>E=mc<sup>2</sup></code>"));
    assert!(out.contains("the <code>E</code> represents <em>energy</em>"));
}

non_normative!(
    r#"
== Unconstrained

"#
);

// When the text is bounded by word characters, a double pair of backticks is
// required for the monospace formatting to apply.
#[test]
fn unconstrained() {
    verifies!(
        r#"
As with other types of text formatting, if the text is bounded by word characters on either side, it must be enclosed in a double pair of backtick characters (`{backtick}{backtick}`) in order for the formatting to be applied.

Here's an example:

----
The command will re``link`` all packages.
----

"#
    );

    let out = convert("The command will re``link`` all packages.\n");
    assert!(out.contains("The command will re<code>link</code> all packages."));
    assert_css(&out, "code", 1);
}

non_normative!(
    r#"
== Mixed Formatting

"#
);

// Monospace can combine with bold and italic when the marks nest from
// outermost (monospace) to innermost (italic).
#[test]
fn mixed_formatting() {
    verifies!(
        r#"
Monospaced text can also be formatted in bold or italic or both, as long as the markup pairs are entered in the right order.
The monospace markup must be the outermost formatting mark, then the bold marks, then the italic marks.

.Order of inline formatting syntax
[#ex-mix]
----
`*_monospaced bold italic_*`
----

The result of <<ex-mix>> is rendered below.

====
`*_monospaced bold italic_*`
====

"#
    );

    let out = convert("`*_monospaced bold italic_*`\n");
    assert!(out.contains("<code><strong><em>monospaced bold italic</em></strong></code>"));
}

non_normative!(
    r#"
== Literal Monospace

To learn how to make monospace text that's not otherwise formatted, see xref:literal-monospace.adoc[].
"#
);
